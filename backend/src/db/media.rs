use crate::{error::AppError, state::AppState};
use futures_util::{StreamExt, TryStreamExt, io::AsyncWriteExt};
use mongodb::bson::{self, Bson, DateTime, Document, doc, oid::ObjectId};

pub async fn download(
    state: &AppState,
    event: &Document,
    message_id: &str,
) -> Result<ObjectId, AppError> {
    // Only LINE's fixed content endpoint is used; user-provided URLs are never fetched.
    if !message_id.bytes().all(|c| c.is_ascii_digit()) || message_id.is_empty() {
        return Err(AppError::Media("Invalid LINE message ID"));
    }
    let event_id = event.get_object_id("_id").map_err(|_| AppError::Data)?;
    let bucket = state.db.gridfs_bucket(None);
    let mut existing = state.scope();
    existing.insert("event_id", event_id);
    let metadata_filter = doc! {"metadata.shop_id":&state.config.shop_id,
    "metadata.channel_id":&state.config.channel_id, "metadata.event_id":event_id};
    if let Some(file) = bucket.find_one(metadata_filter).await? {
        return file.id.as_object_id().ok_or(AppError::Data);
    }
    let response = state
        .http
        .get(format!(
            "https://api-data.line.me/v2/bot/message/{message_id}/content"
        ))
        .bearer_auth(&state.config.channel_access_token)
        .send()
        .await?
        .error_for_status()?;
    let mime = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|s| s.to_str().ok())
        .unwrap_or("application/octet-stream")
        .split(';')
        .next()
        .unwrap_or("")
        .to_string();
    if !matches!(
        mime.as_str(),
        "image/jpeg" | "image/png" | "image/gif" | "image/webp"
    ) {
        return Err(AppError::Media("Unsupported image format"));
    }
    if response
        .content_length()
        .is_some_and(|n| n > state.config.media_max_bytes as u64)
    {
        return Err(AppError::Media("Image exceeds configured size limit"));
    }
    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if bytes.len().saturating_add(chunk.len()) > state.config.media_max_bytes {
            return Err(AppError::Media("Image exceeds configured size limit"));
        }
        bytes.extend_from_slice(&chunk);
    }
    store(state, existing, message_id, &mime, &bytes).await
}

pub async fn store(
    state: &AppState,
    mut metadata: Document,
    message_id: &str,
    mime: &str,
    bytes: &[u8],
) -> Result<ObjectId, AppError> {
    let now = DateTime::now();
    metadata.extend(doc! {"message_id":message_id, "content_type":mime,
        "created_at":now, "updated_at":now,
        "expires_at":DateTime::from_millis(now.timestamp_millis() + state.config.media_retention_days * 86_400_000)});
    let bucket = state.db.gridfs_bucket(None);
    let mut upload = bucket
        .open_upload_stream("line-image")
        .metadata(metadata)
        .await?;
    let id = upload.id().as_object_id().ok_or(AppError::Data)?;
    if let Err(error) = upload.write_all(bytes).await {
        let _ = upload.abort().await;
        return Err(error.into());
    }
    upload.close().await?;
    Ok(id)
}

pub async fn withdrawn(state: &AppState, message_id: &str) -> Result<bool, AppError> {
    let mut filter = state.scope();
    filter.insert("message_id", message_id);
    Ok(state
        .collection("withdrawn_messages")
        .find_one(filter)
        .await?
        .is_some())
}

pub async fn tombstone(state: &AppState, message_id: &str) -> Result<(), AppError> {
    let mut filter = state.scope();
    filter.insert("message_id", message_id);
    let mut record = filter.clone();
    let now = DateTime::now();
    record.extend(doc! {"_id":ObjectId::new(), "created_at":now, "updated_at":now});
    match state
        .collection("withdrawn_messages")
        .update_one(filter, doc! {"$setOnInsert":record})
        .upsert(true)
        .await
    {
        Ok(_) => Ok(()),
        Err(error) if super::events::duplicate(&error) => Ok(()),
        Err(error) => Err(error.into()),
    }
}

pub async fn purge_message(state: &AppState, message_id: &str) -> Result<(), AppError> {
    let mut filter = state.scope();
    filter.insert("message_id", message_id);
    let mut fresh = filter.clone();
    fresh.insert("withdrawn", doc! {"$ne":true});
    let events: Vec<Document> = state
        .collection("line_events")
        .find(fresh)
        .await?
        .try_collect()
        .await?;
    for event in &events {
        if let Ok(inquiry) = event.get_object_id("inquiry_id") {
            super::inquiries::schedule(state, event, inquiry).await?;
        }
    }
    state
        .collection("line_events")
        .update_many(
            filter,
            doc! {
                "$set":{"withdrawn":true, "updated_at":DateTime::now(), "media_status":"WITHDRAWN"},
                "$unset":{"payload.message":"", "media_id":""}
            },
        )
        .await?;
    delete_files(state, doc! {"metadata.message_id":message_id}).await
}

async fn delete_files(state: &AppState, mut filter: Document) -> Result<(), AppError> {
    filter.extend(doc! {"metadata.shop_id":&state.config.shop_id, "metadata.channel_id":&state.config.channel_id});
    let bucket = state.db.gridfs_bucket(None);
    let files: Vec<_> = bucket.find(filter).await?.try_collect().await?;
    for file in files {
        // Concurrent cleanup may have already deleted the same file.
        if let Err(error) = bucket.delete(file.id).await
            && !matches!(
                error.kind.as_ref(),
                mongodb::error::ErrorKind::GridFs(
                    mongodb::error::GridFsErrorKind::FileNotFound { .. }
                )
            )
        {
            return Err(error.into());
        }
    }
    Ok(())
}

pub async fn cleanup(state: &AppState) -> Result<(), AppError> {
    delete_files(state, doc! {"metadata.expires_at":{"$lte":DateTime::now()}}).await?;
    // Reconcile unsend races, including a download finishing after its unsend event.
    let tombstones: Vec<Document> = state
        .collection("withdrawn_messages")
        .find(state.scope())
        .await?
        .try_collect()
        .await?;
    for item in tombstones {
        purge_message(
            state,
            item.get_str("message_id").map_err(|_| AppError::Data)?,
        )
        .await?;
    }
    Ok(())
}

pub async fn read(state: &AppState, id: ObjectId) -> Result<(String, Vec<u8>), AppError> {
    use futures_util::io::AsyncReadExt;
    let bucket = state.db.gridfs_bucket(None);
    let filter = doc! {"_id":id, "metadata.shop_id":&state.config.shop_id,
    "metadata.channel_id":&state.config.channel_id, "metadata.expires_at":{"$gt":DateTime::now()}};
    let file = bucket.find_one(filter).await?.ok_or(AppError::NotFound)?;
    let metadata = file.metadata.ok_or(AppError::Data)?;
    if withdrawn(
        state,
        metadata.get_str("message_id").map_err(|_| AppError::Data)?,
    )
    .await?
    {
        return Err(AppError::NotFound);
    }
    let mime = metadata
        .get_str("content_type")
        .map_err(|_| AppError::Data)?
        .to_string();
    let mut stream = bucket.open_download_stream(Bson::ObjectId(id)).await?;
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).await?;
    Ok((mime, bytes))
}

pub fn payload(event: &Document) -> Result<crate::models::line::LineEvent, AppError> {
    bson::from_document(
        event
            .get_document("payload")
            .map_err(|_| AppError::Data)?
            .clone(),
    )
    .map_err(|_| AppError::Data)
}
