use crate::{error::AppError, models::line::LineEvent, state::AppState};
use mongodb::{
    bson::{self, DateTime, Document, doc, oid::ObjectId},
    options::ReturnDocument,
};

pub fn duplicate(error: &mongodb::error::Error) -> bool {
    matches!(error.kind.as_ref(), mongodb::error::ErrorKind::Write(mongodb::error::WriteFailure::WriteError(e)) if e.code == 11000)
}

pub async fn persist(state: &AppState, event: &LineEvent) -> Result<bool, AppError> {
    let now = DateTime::now();
    let mut filter = state.scope();
    filter.insert("event_id", &event.webhook_event_id);
    let mut record = filter.clone();
    record.extend(doc! {"_id": ObjectId::new(), "payload": bson::to_document(event).map_err(|_| AppError::Data)?,
        "message_id": event.message.as_ref().and_then(|m| m["id"].as_str()),
        "event_at": DateTime::from_millis(event.timestamp), "received_at": now,
        "created_at": now, "updated_at": now, "available_at": now,
        "status": "PENDING", "attempts": 0_i32});
    let inserted = match state
        .collection("line_events")
        .update_one(filter, doc! {"$setOnInsert": record})
        .upsert(true)
        .await
    {
        Ok(result) => result.upserted_id.is_some(),
        Err(error) if duplicate(&error) => false,
        Err(error) => return Err(error.into()),
    };
    // Repeat on redelivery to repair a crash between event and user writes before ACK.
    super::users::observe(state, event).await?;
    Ok(inserted)
}

pub async fn redact_withdrawn(state: &AppState, records: &mut [Document]) -> Result<(), AppError> {
    for record in records {
        if record.get_bool("withdrawn").unwrap_or(false)
            || (if let Ok(id) = record.get_str("message_id") {
                super::media::withdrawn(state, id).await?
            } else {
                false
            })
        {
            record.remove("payload");
            record.remove("media_id");
            record.insert("withdrawn", true);
        }
    }
    Ok(())
}

pub async fn claim(state: &AppState) -> Result<Option<Document>, AppError> {
    let now = DateTime::now();
    let mut filter = state.scope();
    filter
        .extend(doc! {"status": {"$in": ["PENDING", "PROCESSING"]}, "available_at": {"$lte": now}});
    Ok(state
        .collection("line_events")
        .find_one_and_update(
            filter,
            doc! {
                "$set": {"status":"PROCESSING", "lease_token":ObjectId::new(), "updated_at":now,
                    "available_at":DateTime::from_millis(now.timestamp_millis() + 120_000)},
                "$inc":{"attempts":1}
            },
        )
        .sort(doc! {"received_at":1, "event_id":1})
        .return_document(ReturnDocument::After)
        .await?)
}

pub fn lease_filter(state: &AppState, event: &Document) -> Result<Document, AppError> {
    let mut filter = state.scope();
    filter.extend(
        doc! {"_id": event.get_object_id("_id").map_err(|_| AppError::Data)?,
        "lease_token": event.get_object_id("lease_token").map_err(|_| AppError::Data)?},
    );
    Ok(filter)
}

pub async fn finish(state: &AppState, event: &Document, status: &str) -> Result<(), AppError> {
    state.collection("line_events").update_one(lease_filter(state, event)?,
        doc! {"$set":{"status":status, "updated_at":DateTime::now()}, "$unset":{"lease_token":"", "error_code":""}}).await?;
    Ok(())
}

pub async fn renew(state: &AppState, event: &Document) -> Result<bool, AppError> {
    let result = state.collection("line_events").update_one(lease_filter(state, event)?, doc! {
        "$set":{"available_at":DateTime::from_millis(DateTime::now().timestamp_millis() + 120_000)}
    }).await?;
    Ok(result.matched_count == 1)
}

pub async fn retry(state: &AppState, event: &Document, code: &str) -> Result<(), AppError> {
    let attempts = event.get_i32("attempts").unwrap_or(1);
    let delay = (1_i64 << attempts.min(8)) * 1000;
    let status = if attempts >= 8 { "FAILED" } else { "PENDING" };
    state.collection("line_events").update_one(lease_filter(state, event)?, doc! {
        "$set":{"status":status, "error_code":code, "updated_at":DateTime::now(),
            "available_at":DateTime::from_millis(DateTime::now().timestamp_millis() + delay)},
        "$unset":{"lease_token":""}
    }).await?;
    tracing::warn!(event_ref = %event.get_object_id("_id").map_err(|_| AppError::Data)?, attempts, status, code, "inquiry event needs attention");
    Ok(())
}
