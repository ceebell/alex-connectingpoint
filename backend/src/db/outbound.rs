use crate::{
    db::{drafts, events::duplicate, media},
    error::AppError,
    models::{
        customer::Actor,
        outbound::{Outgoing, SendInput, UploadInput},
    },
    state::AppState,
};
use hmac::{Hmac, Mac};
use mongodb::{
    bson::{self, DateTime, Document, doc, oid::ObjectId},
    options::ReturnDocument,
};
use serde_json::{Value, json};
use sha2::Sha256;

const COLLECTION: &str = "outbound_messages";
const MEDIA: &str = "outbound_media";
pub const MEDIA_PATH: &str = "/api/line/outbound-media";

async fn push(
    state: &AppState,
    user: &str,
    message: &Value,
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
        .json(&json!({"to":user,"messages":[message]}))
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

// The 1:1 chat's LINE user and conversation; groups and rooms are never pushed to.
async fn recipient(state: &AppState, inquiry: &Document) -> Result<(String, ObjectId), AppError> {
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
    if !key.starts_with("[\"user\",") {
        return Err(AppError::ReplyUnavailable);
    }
    Ok((user, conversation))
}

// LINE API message and the copy kept for the chat view.
async fn render(
    state: &AppState,
    inquiry_id: ObjectId,
    outgoing: &Outgoing,
) -> Result<(Value, Document), AppError> {
    Ok(match outgoing {
        Outgoing::Text(text) => (
            json!({"type":"text","text":text}),
            doc! {"type":"text","text":text},
        ),
        Outgoing::Sticker {
            package_id,
            sticker_id,
        } => (
            json!({"type":"sticker","packageId":package_id,"stickerId":sticker_id}),
            doc! {"type":"sticker","packageId":package_id,"stickerId":sticker_id},
        ),
        Outgoing::Image(id) => {
            let base = state
                .config
                .public_base_url
                .as_ref()
                .ok_or(AppError::ImageUnavailable)?;
            let mut filter = state.scope();
            filter.extend(
                doc! {"_id":id,"inquiry_id":inquiry_id,"expires_at":{"$gt":DateTime::now()}},
            );
            let record = state
                .collection(MEDIA)
                .find_one(filter)
                .await?
                .ok_or(AppError::NotFound)?;
            let token = record.get_str("token").map_err(|_| AppError::Data)?;
            let original = format!("{MEDIA_PATH}/{token}/original");
            let preview = format!("{MEDIA_PATH}/{token}/preview");
            (
                json!({"type":"image","originalContentUrl":format!("{base}{original}"),"previewImageUrl":format!("{base}{preview}")}),
                doc! {"type":"image","outbound_media_id":id,"original_url":original,"preview_url":preview},
            )
        }
    })
}

pub async fn send(
    state: &AppState,
    inquiry_id: ObjectId,
    input: SendInput,
    actor: &Actor,
) -> Result<Document, AppError> {
    let (request_id, outgoing) = input.validate()?;
    let inquiry = drafts::inquiry(state, inquiry_id).await?;
    let (user, conversation) = recipient(state, &inquiry).await?;
    let (message, stored) = render(state, inquiry_id, &outgoing).await?;
    let content_hash = drafts::hash(&message)?;
    let mut identity = state.scope();
    identity.insert("request_id", &request_id);
    let now = DateTime::now();
    match state
        .collection(COLLECTION)
        .find_one(identity.clone())
        .await?
    {
        Some(existing) => {
            if existing.get_object_id("inquiry_id").ok() != Some(inquiry_id)
                || existing.get_str("text_hash").ok() != Some(content_hash.as_str())
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
                "line_user_id":&user,"event_id":format!("alex-{request_id}"),"event_at":now,
                "source":"alex_push","sender_role":"admin","direction":"outbound","sender_name":&actor.name,
                "sent_by_id":&actor.id,"text_hash":&content_hash,"status":"SENDING","delivery_status":"SENDING",
                "payload":{"type":"message","message":stored},
                "created_at":now,"updated_at":now});
            match state.collection(COLLECTION).insert_one(record).await {
                Ok(_) => (),
                Err(error) if duplicate(&error) => return Err(AppError::RecordConflict),
                Err(error) => return Err(error.into()),
            }
        }
    }
    let result = push(state, &user, &message, &request_id).await;
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

// Unguessable capability token: LINE fetches images by URL without credentials.
fn token(state: &AppState, id: ObjectId) -> Result<String, AppError> {
    let mut mac = Hmac::<Sha256>::new_from_slice(state.config.admin_token.as_bytes())
        .map_err(|_| AppError::Data)?;
    mac.update(
        format!(
            "outbound-media:{}:{:?}",
            id.to_hex(),
            std::time::SystemTime::now()
        )
        .as_bytes(),
    );
    Ok(mac
        .finalize()
        .into_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

pub async fn upload(
    state: &AppState,
    inquiry_id: ObjectId,
    input: UploadInput,
    actor: &Actor,
) -> Result<Document, AppError> {
    if state.config.public_base_url.is_none() {
        return Err(AppError::ImageUnavailable);
    }
    let (original, preview) = input.validate()?;
    let inquiry = drafts::inquiry(state, inquiry_id).await?;
    recipient(state, &inquiry).await?;
    let id = ObjectId::new();
    let token = token(state, id)?;
    let mut files = Vec::new();
    for (variant, image) in [("original", &original), ("preview", &preview)] {
        let mut metadata = state.scope();
        metadata.extend(doc! {"kind":"outbound","inquiry_id":inquiry_id,"outbound_media_id":id});
        files.push(
            media::store(
                state,
                metadata,
                &format!("outbound-{token}-{variant}"),
                image.mime,
                &image.bytes,
            )
            .await?,
        );
    }
    let now = DateTime::now();
    let mut record = state.scope();
    record.extend(doc! {"_id":id,"inquiry_id":inquiry_id,"token":&token,
        "original_file_id":files[0],"preview_file_id":files[1],
        "original_url":format!("{MEDIA_PATH}/{token}/original"),"preview_url":format!("{MEDIA_PATH}/{token}/preview"),
        "uploaded_by":&actor.name,"uploaded_by_id":&actor.id,"created_at":now,"updated_at":now,
        "expires_at":DateTime::from_millis(now.timestamp_millis() + state.config.media_retention_days * 86_400_000)});
    state.collection(MEDIA).insert_one(record.clone()).await?;
    let mut view = doc! {"_id":id,"preview_url":record.get_str("preview_url").map_err(|_| AppError::Data)?,
    "original_url":record.get_str("original_url").map_err(|_| AppError::Data)?};
    view.insert(
        "expires_at",
        bson::Bson::DateTime(
            *record
                .get_datetime("expires_at")
                .map_err(|_| AppError::Data)?,
        ),
    );
    Ok(view)
}

pub async fn read(
    state: &AppState,
    token: &str,
    variant: &str,
) -> Result<(String, Vec<u8>), AppError> {
    if token.len() != 64 || !token.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(AppError::NotFound);
    }
    let field = match variant {
        "original" => "original_file_id",
        "preview" => "preview_file_id",
        _ => return Err(AppError::NotFound),
    };
    let mut filter = state.scope();
    filter.extend(doc! {"token":token,"expires_at":{"$gt":DateTime::now()}});
    let record = state
        .collection(MEDIA)
        .find_one(filter)
        .await?
        .ok_or(AppError::NotFound)?;
    media::read(
        state,
        record.get_object_id(field).map_err(|_| AppError::Data)?,
    )
    .await
}
