use crate::{
    db::{drafts, events::duplicate, media},
    error::AppError,
    state::AppState,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use futures_util::TryStreamExt;
use mongodb::bson::{DateTime, Document, doc, oid::ObjectId};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UploadInput {
    pub name: String,
    pub image_base64: String,
}

fn view(record: &Document) -> Result<Value, AppError> {
    let id = record.get_object_id("_id").map_err(|_| AppError::Data)?;
    let media = record
        .get_object_id("media_id")
        .map_err(|_| AppError::Data)?;
    Ok(
        json!({"id":id.to_hex(),"name":record.get_str("name").unwrap_or("รูปแนบ"),"media_id":media.to_hex(),"url":format!("/api/media/{media}"),"created_at":record.get_datetime("created_at").ok()}),
    )
}

pub async fn list(state: &AppState, inquiry: ObjectId) -> Result<Vec<Value>, AppError> {
    let mut filter = state.scope();
    filter.extend(doc! {"inquiry_id":inquiry,"expires_at":{"$gt":DateTime::now()}});
    let records: Vec<Document> = state
        .collection("inquiry_attachments")
        .find(filter)
        .sort(doc! {"created_at":1})
        .limit(50)
        .await?
        .try_collect()
        .await?;
    records.iter().map(view).collect()
}

// Removes an admin upload and its stored file. The saved summary drops it from its selected
// images so it never points at a missing file; like an upload, this changes the context hash.
pub async fn remove(
    state: &AppState,
    inquiry_id: ObjectId,
    attachment_id: ObjectId,
) -> Result<Value, AppError> {
    let inquiry = drafts::inquiry(state, inquiry_id).await?;
    if inquiry.get_str("status").ok() == Some("CLOSED") {
        return Err(AppError::DraftConflict);
    }
    let mut filter = state.scope();
    filter.extend(doc! {"_id":attachment_id,"inquiry_id":inquiry_id});
    let record = state
        .collection("inquiry_attachments")
        .find_one_and_delete(filter)
        .await?
        .ok_or(AppError::NotFound)?;
    if let Ok(media_id) = record.get_object_id("media_id") {
        let _ = state.db.gridfs_bucket(None).delete(media_id.into()).await;
    }
    let mut scope = state.scope();
    scope.insert("_id", inquiry_id);
    state
        .collection("inquiries")
        .update_one(
            scope,
            doc! {"$pull":{"working_summary.image_event_ids":attachment_id.to_hex()}},
        )
        .await?;
    let snapshot = drafts::snapshot(state, &inquiry).await?;
    Ok(json!({"attachments":list(state, inquiry_id).await?,"context_hash":snapshot.hash}))
}

pub async fn upload(
    state: &AppState,
    inquiry_id: ObjectId,
    input: UploadInput,
) -> Result<Value, AppError> {
    if input.name.chars().count() > 200
        || input.name.chars().any(char::is_control)
        || input.image_base64.len() > 12 * 1024 * 1024
    {
        return Err(AppError::Validation("Invalid attachment"));
    }
    let bytes = STANDARD
        .decode(&input.image_base64)
        .map_err(|_| AppError::BadRequest)?;
    let mime = if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        "image/jpeg"
    } else if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        "image/png"
    } else {
        return Err(AppError::Validation("Attachment must be JPEG or PNG"));
    };
    if bytes.len() > 8 * 1024 * 1024 {
        return Err(AppError::Validation("Attachment exceeds 8 MB"));
    }
    let inquiry = drafts::inquiry(state, inquiry_id).await?;
    if inquiry.get_str("status").ok() == Some("CLOSED") {
        return Err(AppError::DraftConflict);
    }
    let hash = format!("{:x}", Sha256::digest(&bytes));
    let mut identity = state.scope();
    identity.extend(doc! {"inquiry_id":inquiry_id,"content_hash":hash});
    let existing = state
        .collection("inquiry_attachments")
        .find_one(identity.clone())
        .await?;
    let record = if let Some(record) = existing {
        record
    } else {
        let mut filter = state.scope();
        filter.insert("inquiry_id", inquiry_id);
        if state
            .collection("inquiry_attachments")
            .count_documents(filter)
            .await?
            >= 50
        {
            return Err(AppError::Validation("Inquiry attachment limit reached"));
        }
        let now = DateTime::now();
        let id = ObjectId::new();
        let message_id = format!("attachment-{id}");
        let mut metadata = state.scope();
        metadata.extend(doc! {"inquiry_id":inquiry_id,"event_id":id});
        let media_id = media::store(state, metadata, &message_id, mime, &bytes).await?;
        let mut record = identity.clone();
        record.extend(doc! {"_id":id,"media_id":media_id,"name":input.name,"message_id":&message_id,
            "source":"admin_attachment","sender_role":"admin","status":"DONE","event_id":&message_id,"event_at":now,
            "payload":{"type":"admin_attachment","message":{"id":&message_id,"type":"image"}},
            "created_at":now,"updated_at":now,"expires_at":DateTime::from_millis(now.timestamp_millis()+state.config.media_retention_days*86_400_000)});
        match state
            .collection("inquiry_attachments")
            .insert_one(record.clone())
            .await
        {
            Ok(_) => record,
            Err(error) => {
                let _ = state.db.gridfs_bucket(None).delete(media_id.into()).await;
                if duplicate(&error) {
                    state
                        .collection("inquiry_attachments")
                        .find_one(identity)
                        .await?
                        .ok_or(AppError::Data)?
                } else {
                    return Err(error.into());
                }
            }
        }
    };
    let snapshot = drafts::snapshot(state, &inquiry).await?;
    Ok(json!({"attachment":view(&record)?,"context_hash":snapshot.hash}))
}
