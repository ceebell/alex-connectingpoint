use crate::{
    db::{drafts, events::duplicate},
    error::AppError,
    models::{customer::Actor, outbound::SendInput},
    state::AppState,
};
use mongodb::{
    bson::{DateTime, Document, doc, oid::ObjectId},
    options::ReturnDocument,
};
use serde_json::{Value, json};

const COLLECTION: &str = "outbound_messages";

async fn push(
    state: &AppState,
    user: &str,
    text: &str,
    request_id: &str,
) -> Result<Option<String>, &'static str> {
    let response = state
        .http
        .post(format!(
            "{}/v2/bot/message/push",
            state.config.line_api_base
        ))
        .bearer_auth(&state.config.channel_access_token)
        .header("X-Line-Retry-Key", request_id)
        .json(&json!({"to":user,"messages":[{"type":"text","text":text}]}))
        .send()
        .await
        .map_err(|_| "LINE_UNAVAILABLE")?;
    match response.status().as_u16() {
        200 => {
            let body: Value = response.json().await.unwrap_or(Value::Null);
            Ok(body["sentMessages"][0]["id"].as_str().map(String::from))
        }
        // The retry key was already accepted: the message went out earlier.
        409 => Ok(None),
        // Rate limit or the plan's monthly message quota.
        429 => Err("LINE_RATE_LIMITED"),
        status if status >= 500 => Err("LINE_UNAVAILABLE"),
        _ => Err("LINE_SEND_FAILED"),
    }
}

pub async fn send(
    state: &AppState,
    inquiry_id: ObjectId,
    input: SendInput,
    actor: &Actor,
) -> Result<Document, AppError> {
    let input = input.validate()?;
    let inquiry = drafts::inquiry(state, inquiry_id).await?;
    let user = inquiry
        .get_str("line_user_id")
        .map_err(|_| AppError::ReplyUnavailable)?
        .to_string();
    let conversation = inquiry
        .get_object_id("conversation_id")
        .map_err(|_| AppError::ReplyUnavailable)?;
    let mut filter = state.scope();
    filter.insert("_id", conversation);
    let key = state
        .collection("conversations")
        .find_one(filter)
        .await?
        .and_then(|c| c.get_str("conversation_key").ok().map(String::from))
        .unwrap_or_default();
    // Push to a user ID only for 1:1 chats; a group member must not get a DM.
    if !key.starts_with("[\"user\",") {
        return Err(AppError::ReplyUnavailable);
    }
    let text_hash = drafts::hash(&json!(input.text))?;
    let mut identity = state.scope();
    identity.insert("request_id", &input.request_id);
    let now = DateTime::now();
    match state
        .collection(COLLECTION)
        .find_one(identity.clone())
        .await?
    {
        Some(existing) => {
            if existing.get_object_id("inquiry_id").ok() != Some(inquiry_id)
                || existing.get_str("text_hash").ok() != Some(text_hash.as_str())
            {
                return Err(AppError::RecordConflict);
            }
            if existing.get_str("status").ok() == Some("DONE") {
                return Ok(existing);
            }
            // Retry a failed send, or one abandoned by a crashed request.
            let mut claim = identity.clone();
            claim.insert(
                "$or",
                vec![
                    doc! {"status":"FAILED"},
                    doc! {"status":"SENDING","updated_at":{"$lt":DateTime::from_millis(now.timestamp_millis()-60_000)}},
                ],
            );
            state
                .collection(COLLECTION)
                .find_one_and_update(
                    claim,
                    doc! {"$set":{"status":"SENDING","delivery_status":"SENDING","updated_at":now}},
                )
                .await?
                .ok_or(AppError::RecordConflict)?;
        }
        None => {
            let mut record = identity.clone();
            record.extend(doc! {"_id":ObjectId::new(),"inquiry_id":inquiry_id,"conversation_id":conversation,
                "line_user_id":&user,"event_id":format!("alex-{}",input.request_id),"event_at":now,
                "source":"alex_push","sender_role":"admin","direction":"outbound","sender_name":&actor.name,
                "sent_by_id":&actor.id,"text_hash":&text_hash,"status":"SENDING","delivery_status":"SENDING",
                "payload":{"type":"message","message":{"type":"text","text":&input.text}},
                "created_at":now,"updated_at":now});
            match state.collection(COLLECTION).insert_one(record).await {
                Ok(_) => (),
                Err(error) if duplicate(&error) => return Err(AppError::RecordConflict),
                Err(error) => return Err(error.into()),
            }
        }
    }
    let result = push(state, &user, &input.text, &input.request_id).await;
    let now = DateTime::now();
    let update = match &result {
        Ok(id) => {
            doc! {"$set":{"status":"DONE","delivery_status":"SENT","line_message_id":id.as_deref(),"sent_at":now,"updated_at":now},
            "$unset":{"error_code":""}}
        }
        Err(code) => {
            doc! {"$set":{"status":"FAILED","delivery_status":"FAILED","error_code":*code,"updated_at":now}}
        }
    };
    let record = state
        .collection(COLLECTION)
        .find_one_and_update(identity, update)
        .return_document(ReturnDocument::After)
        .await?
        .ok_or(AppError::Data)?;
    match result {
        Ok(_) => {
            let mut filter = state.scope();
            filter.insert("_id", inquiry_id);
            state
                .collection("inquiries")
                .update_one(filter, doc! {"$set":{"updated_at":now}})
                .await?;
            Ok(record)
        }
        Err(code) => Err(AppError::LineSend(code)),
    }
}
