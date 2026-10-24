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
    "status":"RECEIVED", "mode":"ADMIN_ASSIST", "created_at":now, "updated_at":now});
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
    Ok(())
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
