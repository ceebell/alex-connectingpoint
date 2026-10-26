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

const PROFILE_TTL_MS: i64 = 24 * 60 * 60 * 1000;

fn https_url(value: Option<&str>) -> Option<String> {
    value
        .filter(|u| u.starts_with("https://") && u.len() <= 500 && !u.contains(char::is_whitespace))
        .map(String::from)
}

// Cache the LINE display name and picture via the official Get profile API.
// Best effort: failures never block message processing and retry on a later message.
pub async fn refresh_profile(state: &AppState, user_id: &str) -> Result<(), AppError> {
    let mut filter = state.scope();
    filter.insert("line_user_id", user_id);
    let Some(user) = state
        .collection("line_users")
        .find_one(filter.clone())
        .await?
    else {
        return Ok(());
    };
    let now = DateTime::now();
    if user
        .get_datetime("profile_checked_at")
        .is_ok_and(|at| now.timestamp_millis() - at.timestamp_millis() < PROFILE_TTL_MS)
    {
        return Ok(());
    }
    let response = state
        .http
        .get(format!(
            "{}/v2/bot/profile/{user_id}",
            state.config.line_api_base
        ))
        .bearer_auth(&state.config.channel_access_token)
        .send()
        .await?;
    let update = match response.status().as_u16() {
        200 => {
            let body: serde_json::Value = response.json().await?;
            let name: Option<String> = body["displayName"]
                .as_str()
                .map(|n| n.chars().take(100).collect());
            doc! {"$set":{"display_name":name,"picture_url":https_url(body["pictureUrl"].as_str()),
            "profile_status":"OK","profile_checked_at":now}}
        }
        // Not a friend of the OA, blocked it, or the account was deleted.
        404 => {
            doc! {"$set":{"profile_status":"UNAVAILABLE","profile_checked_at":now},"$unset":{"picture_url":""}}
        }
        _ => return Err(AppError::Media("LINE profile unavailable")),
    };
    state
        .collection("line_users")
        .update_one(filter, update)
        .await?;
    Ok(())
}

// Display names and pictures keyed by LINE user ID for inquiry lists.
pub async fn profiles(
    state: &AppState,
    user_ids: Vec<String>,
) -> Result<std::collections::HashMap<String, Document>, AppError> {
    let mut filter = state.scope();
    filter.insert("line_user_id", doc! {"$in":user_ids});
    let records: Vec<Document> = state
        .collection("line_users")
        .find(filter)
        .projection(doc! {"line_user_id":1,"display_name":1,"picture_url":1,"profile_checked_at":1})
        .await?
        .try_collect()
        .await?;
    Ok(records
        .into_iter()
        .filter_map(|r| Some((r.get_str("line_user_id").ok()?.to_string(), r)))
        .collect())
}
