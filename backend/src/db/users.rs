use crate::{db::events::duplicate, error::AppError, models::line::LineEvent, state::AppState};
use futures_util::TryStreamExt;
use mongodb::bson::{DateTime, Document, doc, oid::ObjectId};

pub async fn observe(state: &AppState, event: &LineEvent) -> Result<(), AppError> {
    let Some(user_id) = event
        .source
        .as_ref()
        .filter(|s| matches!(s.kind.as_str(), "user" | "group" | "room"))
        .and_then(|s| s.user_id.as_deref())
        .filter(|id| !id.is_empty())
    else {
        return Ok(());
    };
    let now = DateTime::now();
    let at = DateTime::from_millis(event.timestamp);
    let mut filter = state.scope();
    filter.insert("line_user_id", user_id);
    let mut initial = filter.clone();
    initial.extend(doc! {"_id":ObjectId::new(), "created_at":now});
    // Min/max make redeliveries and concurrent, out-of-order events harmless.
    let update = doc! {
        "$setOnInsert":initial,
        "$min":{"first_seen_at":at},
        "$max":{"last_seen_at":at, "updated_at":now}
    };
    match state
        .collection("line_users")
        .update_one(filter.clone(), update.clone())
        .upsert(true)
        .await
    {
        Ok(_) => (),
        Err(error) if duplicate(&error) => {
            state
                .collection("line_users")
                .update_one(filter, update)
                .await?;
        }
        Err(error) => return Err(error.into()),
    }
    Ok(())
}

pub async fn list(state: &AppState, before: Option<ObjectId>) -> Result<Vec<Document>, AppError> {
    let mut filter = state.scope();
    if let Some(before) = before {
        filter.insert("_id", doc! {"$lt":before});
    }
    Ok(state
        .collection("line_users")
        .find(filter)
        .sort(doc! {"_id":-1})
        .limit(101)
        .await?
        .try_collect()
        .await?)
}

fn message_filter(state: &AppState, user_id: &str) -> Document {
    let mut filter = state.scope();
    // Query the original source so messages received before user registration also appear.
    filter.extend(doc! {"payload.source.userId":user_id,
    "payload.type":{"$in":["message", "postback"]}});
    filter
}

pub async fn messages(
    state: &AppState,
    user_id: &str,
    after: Option<ObjectId>,
) -> Result<Vec<Document>, AppError> {
    let mut filter = message_filter(state, user_id);
    if let Some(after) = after {
        let mut cursor_filter = message_filter(state, user_id);
        cursor_filter.insert("_id", after);
        let cursor = state
            .collection("line_events")
            .find_one(cursor_filter)
            .await?
            .ok_or(AppError::NotFound)?;
        let at = *cursor
            .get_datetime("event_at")
            .map_err(|_| AppError::Data)?;
        let event_id = cursor.get_str("event_id").map_err(|_| AppError::Data)?;
        filter.insert(
            "$or",
            vec![
                doc! {"event_at":{"$gt":at}},
                doc! {"event_at":at, "event_id":{"$gt":event_id}},
            ],
        );
    }
    Ok(state
        .collection("line_events")
        .find(filter)
        .sort(doc! {"event_at":1, "event_id":1})
        .limit(201)
        .projection(doc! {"lease_token":0})
        .await?
        .try_collect()
        .await?)
}
