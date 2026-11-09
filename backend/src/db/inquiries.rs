use crate::{db::events::duplicate, error::AppError, models::line::LineEvent, state::AppState};
use futures_util::TryStreamExt;
use mongodb::{
    bson::{DateTime, Document, doc, oid::ObjectId},
    options::ReturnDocument,
};

pub async fn link(
    state: &AppState,
    event: &LineEvent,
) -> Result<Option<(ObjectId, Option<ObjectId>)>, AppError> {
    let Some(key) = event.conversation_key() else {
        return Ok(None);
    };
    let now = DateTime::now();
    let mut filter = state.scope();
    filter.insert("conversation_key", key);
    let mut initial = filter.clone();
    initial.extend(
        doc! {"_id":ObjectId::new(), "default_inquiry_id":ObjectId::new(),
        "line_user_id":event.source.as_ref().and_then(|s| s.user_id.as_deref()),
        "created_at":now, "updated_at":now},
    );
    let conversation = match state
        .collection("conversations")
        .find_one_and_update(filter.clone(), doc! {"$setOnInsert":initial})
        .upsert(true)
        .return_document(ReturnDocument::After)
        .await
    {
        Ok(Some(value)) => value,
        Err(error) if duplicate(&error) => state
            .collection("conversations")
            .find_one(filter)
            .await?
            .ok_or(AppError::Data)?,
        Ok(None) => return Err(AppError::Data),
        Err(error) => return Err(error.into()),
    };
    let conversation_id = conversation
        .get_object_id("_id")
        .map_err(|_| AppError::Data)?;
    let default_id = conversation
        .get_object_id("default_inquiry_id")
        .map_err(|_| AppError::Data)?;
    let mut initial = state.scope();
    initial.extend(doc! {"_id":default_id, "conversation_id":conversation_id,
    "line_user_id":event.source.as_ref().and_then(|s| s.user_id.as_deref()),
    "status":"RECEIVED", "mode":"ADMIN_ASSIST", "created_at":now, "updated_at":now,
    "last_activity_at":DateTime::from_millis(event.timestamp)});
    let mut default_filter = state.scope();
    default_filter.insert("_id", default_id);
    match state
        .collection("inquiries")
        .update_one(default_filter, doc! {"$setOnInsert":initial})
        .upsert(true)
        .await
    {
        Ok(_) => (),
        Err(error) if duplicate(&error) => (),
        Err(error) => return Err(error.into()),
    }
    let mut filter = state.scope();
    filter.extend(doc! {"conversation_id":conversation_id, "status":{"$ne":"CLOSED"}});
    let open: Vec<Document> = state
        .collection("inquiries")
        .find(filter)
        .limit(2)
        .await?
        .try_collect()
        .await?;
    let inquiry = if open.len() == 1 {
        Some(open[0].get_object_id("_id").map_err(|_| AppError::Data)?)
    } else {
        None
    };
    Ok(Some((conversation_id, inquiry)))
}

pub async fn record_activity(
    state: &AppState,
    inquiry: ObjectId,
    at: DateTime,
) -> Result<(), AppError> {
    let mut filter = state.scope();
    filter.insert("_id", inquiry);
    state
        .collection("inquiries")
        .update_one(
            filter,
            doc! {"$max":{"last_activity_at":at,"updated_at":DateTime::now()}},
        )
        .await?;
    Ok(())
}

pub async fn list(
    state: &AppState,
    status: Option<&str>,
    before: Option<ObjectId>,
    before_at: Option<i64>,
) -> Result<Vec<Document>, AppError> {
    if before.is_none() && before_at.is_some() {
        return Err(AppError::BadRequest);
    }
    let mut filter = state.scope();
    if let Some(status) = status {
        filter.insert("status", status);
    }
    if let Some(id) = before {
        let mut cursor = state.scope();
        cursor.insert("_id", id);
        let cursor = state
            .collection("inquiries")
            .find_one(cursor)
            .await?
            .ok_or(AppError::NotFound)?;
        let at = match before_at {
            Some(at) => DateTime::from_millis(at),
            None => *cursor
                .get_datetime("last_activity_at")
                .or_else(|_| cursor.get_datetime("created_at"))
                .map_err(|_| AppError::Data)?,
        };
        filter.insert(
            "$or",
            vec![
                doc! {"last_activity_at":{"$lt":at}},
                doc! {"last_activity_at":at,"_id":{"$lt":id}},
            ],
        );
    }
    Ok(state
        .collection("inquiries")
        .find(filter)
        .sort(doc! {"last_activity_at":-1,"_id":-1})
        .limit(100)
        .projection(doc! {"lease_token":0})
        .await?
        .try_collect()
        .await?)
}

// Repair older inbox cards once, using message timestamps rather than draft edit times.
pub async fn backfill_activity(state: &AppState) -> Result<(), AppError> {
    let mut filter = state.scope();
    filter.insert("last_activity_at", doc! {"$exists":false});
    let records: Vec<Document> = state
        .collection("inquiries")
        .find(filter.clone())
        .projection(doc! {"_id":1})
        .await?
        .try_collect()
        .await?;
    let ids: Vec<ObjectId> = records
        .iter()
        .map(|record| record.get_object_id("_id").map_err(|_| AppError::Data))
        .collect::<Result<_, _>>()?;
    for chunk in ids.chunks(500) {
        for collection in super::oa_sync::MESSAGE_COLLECTIONS {
            let mut messages = state.scope();
            messages.extend(doc! {"inquiry_id":{"$in":chunk.to_vec()},"status":"DONE",
            "withdrawn":{"$ne":true},"event_at":{"$type":"date"}});
            let mut latest = state
                .collection(collection)
                .aggregate(vec![
                    doc! {"$match":messages},
                    doc! {"$group":{"_id":"$inquiry_id","at":{"$max":"$event_at"}}},
                ])
                .await?;
            while let Some(record) = latest.try_next().await? {
                record_activity(
                    state,
                    record.get_object_id("_id").map_err(|_| AppError::Data)?,
                    *record.get_datetime("at").map_err(|_| AppError::Data)?,
                )
                .await?;
            }
        }
    }
    // This filter preserves activity recorded concurrently during the repair.
    state
        .collection("inquiries")
        .update_many(
            filter,
            vec![doc! {"$set":{"last_activity_at":"$created_at"}}],
        )
        .await?;
    Ok(())
}

pub async fn schedule(
    state: &AppState,
    event: &Document,
    inquiry: ObjectId,
) -> Result<(), AppError> {
    let received = event
        .get_datetime("received_at")
        .map_err(|_| AppError::Data)?
        .timestamp_millis();
    let now = DateTime::now();
    let mut filter = state.scope();
    filter.insert("inquiry_id", inquiry);
    let mut initial = filter.clone();
    initial.extend(doc! {"_id":ObjectId::new(), "created_at":now});
    let update = doc! {"$setOnInsert":initial,
    "$set":{"status":"PENDING", "updated_at":now},
    "$max":{"ready_at":DateTime::from_millis(received + state.config.burst_ms)},
    "$addToSet":{"event_ids":event.get_object_id("_id").map_err(|_| AppError::Data)?}};
    match state
        .collection("analysis_jobs")
        .update_one(filter.clone(), update.clone())
        .upsert(true)
        .await
    {
        Ok(_) => (),
        Err(error) if duplicate(&error) => {
            state
                .collection("analysis_jobs")
                .update_one(filter, update)
                .await?;
        }
        Err(error) => return Err(error.into()),
    }
    record_activity(
        state,
        inquiry,
        *event.get_datetime("event_at").map_err(|_| AppError::Data)?,
    )
    .await
}

pub fn batches(events: Vec<Document>, burst_ms: i64) -> Vec<Vec<Document>> {
    let mut batches: Vec<Vec<Document>> = Vec::new();
    let mut previous: Option<i64> = None;
    for event in events {
        let at = event
            .get_datetime("event_at")
            .map(|t| t.timestamp_millis())
            .unwrap_or(0);
        if previous.is_none_or(|p| at.saturating_sub(p) > burst_ms) {
            batches.push(Vec::new());
        }
        if let Some(batch) = batches.last_mut() {
            batch.push(event);
        }
        previous = Some(at);
    }
    batches
}
