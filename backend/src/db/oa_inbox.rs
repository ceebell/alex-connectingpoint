use crate::{
    error::AppError,
    models::oa_inbox::{OaInboxBatch, RefreshResult},
    state::AppState,
};
use futures_util::TryStreamExt;
use mongodb::bson::{DateTime, Document, doc, oid::ObjectId};
use serde_json::{Value, json};

// Names and read flags come from the OA inbox, linked only by exact webhook IDs.
// A snapshot is never allowed to clear a newer message or overwrite a later Alex read.
pub async fn ingest(state: &AppState, batch: OaInboxBatch) -> Result<Value, AppError> {
    batch.validate()?;
    if batch.bot_id != state.config.destination {
        return Err(AppError::BadRequest);
    }
    let started = DateTime::from_millis(batch.started_at_ms);
    let mut updated = 0;
    let mut unlinked = 0;
    for chat in batch.chats {
        let mut filter = state.scope();
        filter.extend(
            doc! {"message_id":{"$in":&chat.anchor_message_ids},"status":"DONE",
            "conversation_id":{"$exists":true},"payload.source.type":"user",
            "event_at":{"$lt":started},"created_at":{"$lt":started}},
        );
        let anchors: Vec<Document> = state
            .collection("line_events")
            .find(filter)
            .sort(doc! {"event_at":-1,"event_id":-1})
            .limit(21)
            .await?
            .try_collect()
            .await?;
        let Some(anchor) = anchors.first() else {
            unlinked += 1;
            continue;
        };
        let conversation = anchor
            .get_object_id("conversation_id")
            .map_err(|_| AppError::Data)?;
        let user_id = anchor
            .get_document("payload")
            .ok()
            .and_then(|p| p.get_document("source").ok())
            .and_then(|s| s.get_str("userId").ok())
            .ok_or(AppError::Data)?;
        if anchors
            .iter()
            .any(|a| a.get_object_id("conversation_id").ok() != Some(conversation))
        {
            unlinked += 1;
            continue;
        }
        let now = DateTime::now();
        let mut user_filter = state.scope();
        user_filter.extend(doc! {"line_user_id":user_id,"$or":[
            {"oa_name_observed_at":{"$exists":false}},{"oa_name_observed_at":{"$lt":started}}
        ]});
        state.collection("line_users").update_one(user_filter, doc! {"$set":{
            "oa_display_name":chat.display_name.trim(),"oa_name_observed_at":started,"updated_at":now
        }}).await?;
        let mut conversation_filter = state.scope();
        conversation_filter.extend(doc! {"_id":conversation,"$and":[
            {"$or":[{"oa_observed_at":{"$exists":false}},{"oa_observed_at":{"$lt":started}}]},
            {"$or":[{"oa_local_read_at":{"$exists":false}},{"oa_local_read_at":{"$lt":started}}]}
        ]});
        let at = *anchor
            .get_datetime("event_at")
            .map_err(|_| AppError::Data)?;
        let event_id = anchor.get_str("event_id").map_err(|_| AppError::Data)?;
        state
            .collection("conversations")
            .update_one(
                conversation_filter,
                doc! {"$set":{
                    "oa_chat_id":chat.chat_id,"oa_unread":chat.unread,"oa_observed_at":started,
                    "oa_through_event_at":at,"oa_through_event_id":event_id,"updated_at":now
                }},
            )
            .await?;
        updated += 1;
    }
    Ok(json!({"updated":updated,"unlinked":unlinked}))
}

pub async fn request_refresh(state: &AppState) -> Result<Value, AppError> {
    if !state.config.oa_connector_enabled {
        return Ok(json!({"state":"NOT_CONFIGURED"}));
    }
    let now = DateTime::now();
    let id = ObjectId::new();
    let mut record = state.scope();
    record.extend(doc! {"_id":id,"state":"PENDING","created_at":now,"updated_at":now});
    state
        .collection("oa_sync_requests")
        .insert_one(record)
        .await?;
    Ok(json!({"state":"PENDING","request_id":id.to_hex()}))
}

pub async fn pending(state: &AppState) -> Result<Value, AppError> {
    let mut filter = state.scope();
    filter.extend(doc! {"state":"PENDING","created_at":{"$gt":DateTime::from_millis(DateTime::now().timestamp_millis()-60_000)}});
    let records: Vec<Document> = state
        .collection("oa_sync_requests")
        .find(filter)
        .sort(doc! {"created_at":1})
        .limit(100)
        .projection(doc! {"_id":1})
        .await?
        .try_collect()
        .await?;
    let ids: Vec<String> = records
        .iter()
        .filter_map(|r| r.get_object_id("_id").ok().map(|id| id.to_hex()))
        .collect();
    Ok(json!({"requests":ids}))
}

pub async fn refresh_status(state: &AppState, id: ObjectId) -> Result<Value, AppError> {
    let mut filter = state.scope();
    filter.insert("_id", id);
    let record = state
        .collection("oa_sync_requests")
        .find_one(filter)
        .await?
        .ok_or(AppError::NotFound)?;
    let name = if record.get_str("state").ok() == Some("PENDING")
        && record
            .get_datetime("created_at")
            .is_ok_and(|at| DateTime::now().timestamp_millis() - at.timestamp_millis() > 60_000)
    {
        "OFFLINE"
    } else {
        record.get_str("state").unwrap_or("OFFLINE")
    };
    Ok(json!({"state":name}))
}

pub async fn finish(state: &AppState, id: ObjectId, input: RefreshResult) -> Result<(), AppError> {
    input.validate()?;
    let mut filter = state.scope();
    filter.extend(doc! {"_id":id,"state":"PENDING"});
    if state
        .collection("oa_sync_requests")
        .update_one(
            filter,
            doc! {"$set":{"state":input.state,"updated_at":DateTime::now()}},
        )
        .await?
        .matched_count
        == 0
    {
        return Err(AppError::NotFound);
    }
    Ok(())
}
