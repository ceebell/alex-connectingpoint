use crate::{
    db::{drafts, events},
    error::AppError,
    models::oa_sync::{OaBatch, OaHeartbeat},
    state::AppState,
};
use futures_util::TryStreamExt;
use mongodb::bson::{DateTime, Document, doc, oid::ObjectId};
use serde_json::{Value, json};

// Customer webhook events, OA Manager captures and replies sent from Alex.
pub const MESSAGE_COLLECTIONS: [&str; 3] = ["line_events", "oa_messages", "outbound_messages"];

pub fn source_type(record: &Document) -> &'static str {
    match record.get_str("source").ok() {
        Some("oa_manager_browser") => "oa_message",
        Some("alex_push") => "admin_message",
        _ => "line_event",
    }
}

pub fn sort_messages(records: &mut [Document]) {
    records.sort_by(|a, b| {
        let time = |d: &Document| {
            d.get_datetime("event_at")
                .map(|v| v.timestamp_millis())
                .unwrap_or(0)
        };
        time(a).cmp(&time(b)).then_with(|| {
            a.get_str("event_id")
                .unwrap_or("")
                .cmp(b.get_str("event_id").unwrap_or(""))
        })
    });
}

pub async fn messages(
    state: &AppState,
    inquiry: ObjectId,
    after: Option<ObjectId>,
) -> Result<Vec<Document>, AppError> {
    let mut filter = state.scope();
    filter.extend(doc! {"inquiry_id":inquiry,"status":"DONE","withdrawn":{"$ne":true}});
    if let Some(id) = after {
        let mut cursor_filter = state.scope();
        cursor_filter.extend(doc! {"_id":id,"inquiry_id":inquiry});
        let mut cursor = None;
        for name in MESSAGE_COLLECTIONS {
            cursor = state
                .collection(name)
                .find_one(cursor_filter.clone())
                .await?;
            if cursor.is_some() {
                break;
            }
        }
        let cursor = cursor.ok_or(AppError::NotFound)?;
        let at = *cursor
            .get_datetime("event_at")
            .map_err(|_| AppError::Data)?;
        let key = cursor.get_str("event_id").map_err(|_| AppError::Data)?;
        filter.insert(
            "$or",
            vec![
                doc! {"event_at":{"$gt":at}},
                doc! {"event_at":at,"event_id":{"$gt":key}},
            ],
        );
    }
    let mut records = Vec::new();
    for name in MESSAGE_COLLECTIONS {
        records.extend(
            state
                .collection(name)
                .find(filter.clone())
                .sort(doc! {"event_at":1,"event_id":1})
                .limit(201)
                .projection(doc! {"lease_token":0,"content_hash":0})
                .await?
                .try_collect::<Vec<_>>()
                .await?,
        );
    }
    sort_messages(&mut records);
    records.truncate(201);
    Ok(records)
}

pub async fn ingest(state: &AppState, batch: OaBatch) -> Result<Value, AppError> {
    batch.validate()?;
    if batch.bot_id != state.config.destination {
        return Err(AppError::BadRequest);
    }
    let mut filter = state.scope();
    filter.extend(doc! {"message_id":{"$in":&batch.anchor_message_ids},"status":"DONE","inquiry_id":{"$exists":true},"payload.source.type":"user"});
    let anchors: Vec<Document> = state
        .collection("line_events")
        .find(filter)
        .limit(21)
        .await?
        .try_collect()
        .await?;
    let first = anchors.first().ok_or(AppError::OaUnlinked)?;
    let inquiry = first
        .get_object_id("inquiry_id")
        .map_err(|_| AppError::Data)?;
    let conversation = first
        .get_object_id("conversation_id")
        .map_err(|_| AppError::Data)?;
    if anchors.iter().any(|a| {
        a.get_object_id("inquiry_id").ok() != Some(inquiry)
            || a.get_object_id("conversation_id").ok() != Some(conversation)
    }) {
        return Err(AppError::OaUnlinked);
    }
    let inquiry_doc = drafts::inquiry(state, inquiry).await?;
    let anchor_at = anchors
        .iter()
        .filter_map(|a| a.get_datetime("event_at").ok())
        .map(|t| t.timestamp_millis())
        .max()
        .ok_or(AppError::Data)?;
    if batch.messages.iter().any(|m| m.sent_at_ms < anchor_at) {
        return Err(AppError::OaUnlinked);
    }
    let mut inserted = 0;
    for message in &batch.messages {
        let hash = drafts::hash(
            &json!({"chat_id":batch.chat_id,"inquiry_id":inquiry.to_hex(),"text":message.text,"at":message.sent_at_ms}),
        )?;
        let mut identity = state.scope();
        identity.extend(doc! {"bot_id":&batch.bot_id,"external_message_id":&message.id});
        if let Some(existing) = state
            .collection("oa_messages")
            .find_one(identity.clone())
            .await?
        {
            if existing.get_str("content_hash").ok() != Some(&hash) {
                return Err(AppError::OaMessageConflict);
            }
            continue;
        }
        let now = DateTime::now();
        let mut record = identity.clone();
        record.extend(doc! {"_id":ObjectId::new(),"chat_id":&batch.chat_id,"event_id":format!("oa-{}",message.id),
            "inquiry_id":inquiry,"conversation_id":conversation,"line_user_id":inquiry_doc.get_str("line_user_id").ok(),
            "event_at":DateTime::from_millis(message.sent_at_ms),"created_at":now,"updated_at":now,"captured_at":now,
            "source":"oa_manager_browser","sender_role":"admin","direction":"outbound","sender_name":"แอดมิน",
            "status":"DONE","content_hash":&hash,"verified_by_line_webhook":false,
            "payload":{"type":"message","message":{"type":"text","text":&message.text}}});
        match state.collection("oa_messages").insert_one(record).await {
            Ok(_) => inserted += 1,
            Err(error) if events::duplicate(&error) => {
                let existing = state
                    .collection("oa_messages")
                    .find_one(identity)
                    .await?
                    .ok_or(AppError::Data)?;
                if existing.get_str("content_hash").ok() != Some(&hash) {
                    return Err(AppError::OaMessageConflict);
                }
            }
            Err(error) => return Err(error.into()),
        }
    }
    Ok(json!({"inserted":inserted,"duplicates":batch.messages.len()-inserted}))
}

pub async fn heartbeat(state: &AppState, input: OaHeartbeat) -> Result<(), AppError> {
    input.validate()?;
    let now = DateTime::now();
    let mut filter = state.scope();
    filter.insert("name", "line_oa");
    let mut initial = filter.clone();
    initial.extend(doc! {"_id":ObjectId::new(),"created_at":now});
    state.collection("connector_statuses").update_one(filter,doc! {"$setOnInsert":initial,"$set":{
        "updated_at":now,"state":input.state,"synced":input.synced as i64,"pending":input.pending as i64}}).upsert(true).await?;
    Ok(())
}

pub async fn status(state: &AppState) -> Result<Value, AppError> {
    let mut filter = state.scope();
    filter.insert("name", "line_oa");
    let Some(record) = state
        .collection("connector_statuses")
        .find_one(filter)
        .await?
    else {
        return Ok(json!({"state":"NOT_CONFIGURED"}));
    };
    let at = record
        .get_datetime("updated_at")
        .map_err(|_| AppError::Data)?;
    let state_name = if DateTime::now().timestamp_millis() - at.timestamp_millis() > 90_000 {
        "OFFLINE"
    } else {
        record.get_str("state").unwrap_or("OFFLINE")
    };
    Ok(
        json!({"state":state_name,"updated_at":at,"synced":record.get_i64("synced").unwrap_or(0),"pending":record.get_i64("pending").unwrap_or(0)}),
    )
}
