use crate::{error::AppError, state::AppState};
use futures_util::TryStreamExt;
use mongodb::bson::{DateTime, Document, doc, oid::ObjectId};
use std::collections::HashMap;

fn key(message: &Document) -> (i64, &str) {
    (
        message
            .get_datetime("event_at")
            .map(|v| v.timestamp_millis())
            .unwrap_or(0),
        message.get_str("event_id").unwrap_or(""),
    )
}

fn read_key(conversation: &Document) -> (i64, &str) {
    (
        conversation
            .get_datetime("last_read_event_at")
            .map(|v| v.timestamp_millis())
            .unwrap_or(-1),
        conversation.get_str("last_read_event_id").unwrap_or(""),
    )
}

fn preview(message: &Document) -> String {
    let payload = message.get_document("payload").ok();
    let content = payload.and_then(|p| p.get_document("message").ok());
    let text = match content.and_then(|m| m.get_str("type").ok()) {
        Some("text") => content
            .and_then(|m| m.get_str("text").ok())
            .unwrap_or("ข้อความ"),
        Some("image") => "ส่งรูปภาพ",
        Some("sticker") => "ส่งสติกเกอร์",
        Some("video") => "ส่งวิดีโอ",
        Some("audio") => "ส่งข้อความเสียง",
        Some("file") => "ส่งไฟล์",
        Some("location") => "ส่งตำแหน่งที่ตั้ง",
        _ => payload
            .and_then(|p| p.get_document("postback").ok())
            .and_then(|p| p.get_str("data").ok())
            .unwrap_or("ข้อความ"),
    };
    let text = text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(240)
        .collect::<String>();
    if message.get_str("sender_role").ok() == Some("admin") {
        format!("คุณ: {text}")
    } else {
        text
    }
}

// Batch the list metadata; never return read tokens or hidden/withdrawn content.
pub async fn decorate(state: &AppState, inquiries: &mut [Document]) -> Result<(), AppError> {
    let ids: Vec<ObjectId> = inquiries
        .iter()
        .filter_map(|i| i.get_object_id("_id").ok())
        .collect();
    if ids.is_empty() {
        return Ok(());
    }
    let mut latest: HashMap<ObjectId, Document> = HashMap::new();
    let mut incoming: HashMap<ObjectId, Document> = HashMap::new();
    for collection in super::oa_sync::MESSAGE_COLLECTIONS {
        let mut filter = state.scope();
        filter.extend(doc! {"inquiry_id":{"$in":&ids},"status":"DONE","withdrawn":{"$ne":true},"payload.type":{"$in":["message","postback"]}});
        let mut pipeline = vec![doc! {"$match":filter}];
        if collection == "line_events" {
            // Suppress unsends immediately, before the purge worker updates the event.
            pipeline.push(doc! {"$lookup":{"from":"withdrawn_messages","let":{"message":"$message_id"},"pipeline":[
                {"$match":{"shop_id":&state.config.shop_id,"channel_id":&state.config.channel_id,"$expr":{"$eq":["$message_id","$$message"]}}},
                {"$limit":1},{"$project":{"_id":1}}
            ],"as":"_withdrawn"}});
            pipeline.push(doc! {"$match":{"_withdrawn.0":{"$exists":false}}});
        }
        pipeline.push(doc! {"$sort":{"event_at":-1,"event_id":-1}});
        pipeline.push(doc! {"$group":{"_id":"$inquiry_id","message":{"$first":"$$ROOT"}}});
        let mut records = state.collection(collection).aggregate(pipeline).await?;
        while let Some(record) = records.try_next().await? {
            let id = record.get_object_id("_id").map_err(|_| AppError::Data)?;
            let message = record
                .get_document("message")
                .map_err(|_| AppError::Data)?
                .clone();
            if collection == "line_events" {
                incoming.insert(id, message.clone());
            }
            if latest.get(&id).is_none_or(|old| key(old) < key(&message)) {
                latest.insert(id, message);
            }
        }
    }
    let conversations: Vec<ObjectId> = inquiries
        .iter()
        .filter_map(|i| i.get_object_id("conversation_id").ok())
        .collect();
    let mut filter = state.scope();
    filter.insert("_id", doc! {"$in":conversations});
    let records: Vec<Document> = state
        .collection("conversations")
        .find(filter)
        .projection(doc! {"last_read_event_at":1,"last_read_event_id":1,"oa_unread":1,"oa_through_event_at":1,"oa_through_event_id":1})
        .await?
        .try_collect()
        .await?;
    let reads: HashMap<ObjectId, Document> = records
        .into_iter()
        .filter_map(|r| Some((r.get_object_id("_id").ok()?, r)))
        .collect();
    for inquiry in inquiries {
        let id = inquiry.get_object_id("_id").map_err(|_| AppError::Data)?;
        if let Some(message) = latest.get(&id) {
            inquiry.insert("last_message_preview", preview(message));
        }
        let read = inquiry
            .get_object_id("conversation_id")
            .ok()
            .and_then(|c| reads.get(&c));
        let mut unread = incoming
            .get(&id)
            .is_some_and(|m| read.is_none_or(|r| key(m) > read_key(r)));
        let mut source = "alex";
        if state.config.oa_connector_enabled
            && let Some(read) = read
            && let Ok(oa_unread) = read.get_bool("oa_unread")
            && let Ok(through) = read.get_datetime("oa_through_event_at")
        {
            let boundary = (
                through.timestamp_millis(),
                read.get_str("oa_through_event_id").unwrap_or(""),
            );
            if incoming.get(&id).is_none_or(|m| key(m) <= boundary) {
                unread = oa_unread;
                source = "line_oa";
            }
        }
        if let Some(message_id) = incoming.get(&id).and_then(|m| m.get_object_id("_id").ok()) {
            inquiry.insert("last_incoming_message_id", message_id);
        }
        inquiry.insert("unread", unread);
        inquiry.insert("unread_source", source);
    }
    Ok(())
}

pub async fn mark_read(
    state: &AppState,
    inquiry_id: ObjectId,
    message_id: ObjectId,
) -> Result<&'static str, AppError> {
    let started = DateTime::now();
    let inquiry = super::drafts::inquiry(state, inquiry_id).await?;
    let conversation_id = inquiry
        .get_object_id("conversation_id")
        .map_err(|_| AppError::Data)?;
    let mut filter = state.scope();
    filter.extend(doc! {"_id":message_id,"inquiry_id":inquiry_id,"status":"DONE","withdrawn":{"$ne":true},"payload.type":{"$in":["message","postback"]}});
    let message = state
        .collection("line_events")
        .find_one(filter)
        .await?
        .ok_or(AppError::NotFound)?;
    if let Ok(id) = message.get_str("message_id")
        && super::media::withdrawn(state, id).await?
    {
        return Err(AppError::NotFound);
    }
    let mut conversation_filter = state.scope();
    conversation_filter.insert("_id", conversation_id);
    let conversation = state
        .collection("conversations")
        .find_one(conversation_filter.clone())
        .await?
        .ok_or(AppError::NotFound)?;
    if key(&message) <= read_key(&conversation)
        && !(state.config.oa_connector_enabled
            && conversation.get_bool("oa_unread").unwrap_or(false))
    {
        return Ok("ALREADY_READ");
    }
    let token = message
        .get_document("payload")
        .ok()
        .and_then(|p| p.get_document("message").ok())
        .and_then(|m| m.get_str("markAsReadToken").ok())
        .filter(|v| !v.is_empty());
    let line_status = if let Some(token) = token {
        let response = state
            .http
            .post(format!(
                "{}/v2/bot/chat/markAsRead",
                state.config.line_api_base
            ))
            .bearer_auth(&state.config.channel_access_token)
            .json(&serde_json::json!({"markAsReadToken":token}))
            .send()
            .await
            .map_err(|_| AppError::LineSend("LINE_READ_FAILED"))?;
        if !response.status().is_success() {
            return Err(AppError::LineSend("LINE_READ_FAILED"));
        }
        "SYNCED"
    } else {
        // Older webhook events and postbacks may not have a LINE read token.
        "TOKEN_UNAVAILABLE"
    };
    let at = *message
        .get_datetime("event_at")
        .map_err(|_| AppError::Data)?;
    let event_id = message.get_str("event_id").map_err(|_| AppError::Data)?;
    conversation_filter.insert(
        "$or",
        vec![
            doc! {"last_read_event_at":{"$exists":false}},
            doc! {"last_read_event_at":{"$lt":at}},
            doc! {"last_read_event_at":at,"last_read_event_id":{"$lt":event_id}},
        ],
    );
    state.collection("conversations").update_one(conversation_filter, doc! {"$set":{
        "last_read_event_at":at,"last_read_event_id":event_id,"last_read_at":DateTime::now(),"updated_at":DateTime::now()
    }}).await?;
    if line_status == "SYNCED" && state.config.oa_connector_enabled {
        let mut filter = state.scope();
        filter.extend(doc! {"_id":conversation_id,"$and":[
            {"$or":[{"oa_observed_at":{"$exists":false}},{"oa_observed_at":{"$lte":started}}]},
            {"$or":[{"oa_through_event_at":{"$exists":false}},{"oa_through_event_at":{"$lt":at}},
                {"oa_through_event_at":at,"oa_through_event_id":{"$lte":event_id}}]}
        ]});
        state
            .collection("conversations")
            .update_one(
                filter,
                doc! {"$set":{
                    "oa_unread":false,"oa_local_read_at":DateTime::now(),"oa_through_event_at":at,
                    "oa_through_event_id":event_id,"updated_at":DateTime::now()
                }},
            )
            .await?;
    }
    Ok(line_status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn previews_label_media_and_keep_admin_replies_distinct() {
        for (kind, label) in [
            ("image", "ส่งรูปภาพ"),
            ("sticker", "ส่งสติกเกอร์"),
            ("video", "ส่งวิดีโอ"),
            ("audio", "ส่งข้อความเสียง"),
            ("file", "ส่งไฟล์"),
            ("location", "ส่งตำแหน่งที่ตั้ง"),
        ] {
            assert_eq!(
                preview(
                    &doc! {"payload":{"type":"message","message":{"type":kind,"markAsReadToken":"private-fixture"}}}
                ),
                label
            );
        }
        assert_eq!(
            preview(
                &doc! {"sender_role":"admin","payload":{"type":"message","message":{"type":"text","text":"hello\n  world"}}}
            ),
            "คุณ: hello world"
        );
        let text = "ก".repeat(1000);
        assert_eq!(
            preview(&doc! {"payload":{"message":{"type":"text","text":text}}})
                .chars()
                .count(),
            240
        );
    }
}
