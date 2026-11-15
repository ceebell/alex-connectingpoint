use crate::{
    db::{drafts, events::duplicate, outbound},
    error::AppError,
    models::{
        customer::Actor,
        outbound::{UploadedImage, image_type},
        quick_replies::Content,
    },
    state::AppState,
};
use futures_util::{
    TryStreamExt,
    io::{AsyncReadExt, AsyncWriteExt},
};
use mongodb::{
    bson::{Bson, DateTime, Document, doc, oid::ObjectId},
    options::ReturnDocument,
};
use sha2::{Digest, Sha256};

const COLLECTION: &str = "quick_replies";

fn filter(state: &AppState, id: ObjectId) -> Document {
    let mut filter = state.scope();
    filter.extend(doc! {"_id":id,"deleted_at":{"$exists":false}});
    filter
}
fn public(mut record: Document) -> Result<Document, AppError> {
    let id = record.get_object_id("_id").map_err(|_| AppError::Data)?;
    if record.get_str("kind").ok() == Some("image") {
        record.insert(
            "preview_url",
            format!("/api/admin/quick-replies/{id}/image/preview"),
        );
    }
    for key in [
        "original_file_id",
        "preview_file_id",
        "content_hash",
        "request_id",
    ] {
        record.remove(key);
    }
    Ok(record)
}
async fn record(state: &AppState, id: ObjectId) -> Result<Document, AppError> {
    state
        .collection(COLLECTION)
        .find_one(filter(state, id))
        .await?
        .ok_or(AppError::NotFound)
}
pub async fn list(
    state: &AppState,
    before: Option<ObjectId>,
) -> Result<(Vec<Document>, Option<String>), AppError> {
    let mut filter = state.scope();
    filter.insert("deleted_at", doc! {"$exists":false});
    if let Some(before) = before {
        filter.insert("_id", doc! {"$lt":before});
    }
    let mut rows: Vec<Document> = state
        .collection(COLLECTION)
        .find(filter)
        .sort(doc! {"_id":-1})
        .limit(81)
        .await?
        .try_collect()
        .await?;
    let more = rows.len() > 80;
    rows.truncate(80);
    let next = if more {
        rows.last()
            .and_then(|row| row.get_object_id("_id").ok())
            .map(|id| id.to_hex())
    } else {
        None
    };
    Ok((
        rows.into_iter().map(public).collect::<Result<_, _>>()?,
        next,
    ))
}
async fn cleanup(state: &AppState, files: &[ObjectId]) {
    for id in files {
        let _ = state
            .db
            .gridfs_bucket(None)
            .delete(Bson::ObjectId(*id))
            .await;
    }
}
async fn store_images(
    state: &AppState,
    reply: ObjectId,
    images: Option<(UploadedImage, UploadedImage)>,
) -> Result<Vec<ObjectId>, AppError> {
    let Some((original, preview)) = images else {
        return Ok(Vec::new());
    };
    let mut files = Vec::new();
    for (variant, image) in [("original", original), ("preview", preview)] {
        let result: Result<ObjectId, AppError> = async {
            let now = DateTime::now(); let mut metadata = state.scope();
            // Reusable images must not expire with temporary LINE media.
            metadata.extend(doc! {"source":"quick_reply","quick_reply_id":reply,"variant":variant,"content_type":image.mime,"created_at":now,"updated_at":now});
            let mut upload = state.db.gridfs_bucket(None).open_upload_stream("quick-reply-image").metadata(metadata).await?;
            let id = upload.id().as_object_id().ok_or(AppError::Data)?;
            if let Err(error) = upload.write_all(&image.bytes).await { let _ = upload.abort().await; return Err(error.into()); }
            files.push(id); upload.close().await?; Ok(id)
        }.await;
        if let Err(error) = result {
            cleanup(state, &files).await;
            return Err(error);
        }
    }
    Ok(files)
}
fn files(record: &Document) -> Vec<ObjectId> {
    ["original_file_id", "preview_file_id"]
        .iter()
        .filter_map(|key| record.get_object_id(key).ok())
        .collect()
}
fn fingerprint(content: &Content) -> Result<String, AppError> {
    let images = content.image.as_ref().map(|(original, preview)| {
        (
            format!("{:x}", Sha256::digest(&original.bytes)),
            format!("{:x}", Sha256::digest(&preview.bytes)),
        )
    });
    drafts::hash(&serde_json::json!([
        &content.title,
        &content.kind,
        &content.text,
        images
    ]))
}
pub async fn create(
    state: &AppState,
    request_id: &str,
    content: Content,
    actor: &Actor,
) -> Result<Document, AppError> {
    let hash = fingerprint(&content)?;
    let mut identity = state.scope();
    identity.extend(doc! {"request_id":request_id,"created_by_id":&actor.id});
    if let Some(existing) = state
        .collection(COLLECTION)
        .find_one(identity.clone())
        .await?
    {
        return if existing.get_str("content_hash").ok() == Some(&hash)
            && !existing.contains_key("deleted_at")
        {
            public(existing)
        } else {
            Err(AppError::RecordConflict)
        };
    }
    let id = ObjectId::new();
    let stored = store_images(state, id, content.image).await?;
    let now = DateTime::now();
    let mut row = identity.clone();
    row.extend(doc! {"_id":id,"title":content.title,"kind":content.kind,"text":content.text,"version":1_i64,"content_hash":&hash,
        "created_by":&actor.name,"updated_by":&actor.name,"updated_by_id":&actor.id,"created_at":now,"updated_at":now});
    if stored.len() == 2 {
        row.extend(doc! {"original_file_id":stored[0],"preview_file_id":stored[1]});
    }
    match state.collection(COLLECTION).insert_one(row.clone()).await {
        Ok(_) => public(row),
        Err(error) if duplicate(&error) => {
            cleanup(state, &stored).await;
            let existing = state
                .collection(COLLECTION)
                .find_one(identity)
                .await?
                .ok_or(AppError::Data)?;
            if existing.get_str("content_hash").ok() == Some(&hash)
                && !existing.contains_key("deleted_at")
            {
                public(existing)
            } else {
                Err(AppError::RecordConflict)
            }
        }
        // Keep files on an ambiguous acknowledgement: the insert may have committed.
        Err(error) => Err(error.into()),
    }
}
pub async fn edit(
    state: &AppState,
    id: ObjectId,
    version: i64,
    content: Content,
    actor: &Actor,
) -> Result<Document, AppError> {
    let existing = record(state, id).await?;
    if existing.get_i64("version").ok() != Some(version) {
        return Err(AppError::RecordConflict);
    }
    if existing.get_str("kind").ok() != Some(content.kind.as_str()) {
        return Err(AppError::Validation("Quick reply type cannot change"));
    }
    let next = version.checked_add(1).ok_or(AppError::Data)?;
    let stored = store_images(state, id, content.image).await?;
    let mut set = doc! {"title":content.title,"text":content.text,"version":next,"updated_at":DateTime::now(),"updated_by":&actor.name,"updated_by_id":&actor.id};
    if stored.len() == 2 {
        set.extend(doc! {"original_file_id":stored[0],"preview_file_id":stored[1]});
    }
    let mut filter = filter(state, id);
    filter.insert("version", version);
    match state
        .collection(COLLECTION)
        .find_one_and_update(filter, doc! {"$set":set})
        .return_document(ReturnDocument::After)
        .await
    {
        Ok(Some(row)) => {
            if !stored.is_empty() {
                cleanup(state, &files(&existing)).await;
            }
            public(row)
        }
        Ok(None) => {
            cleanup(state, &stored).await;
            Err(AppError::RecordConflict)
        }
        Err(error) => Err(error.into()),
    }
}
pub async fn delete(
    state: &AppState,
    id: ObjectId,
    version: i64,
    actor: &Actor,
) -> Result<(), AppError> {
    let mut query = filter(state, id);
    query.insert("version", version);
    let now = DateTime::now();
    let old = state.collection(COLLECTION).find_one_and_update(query, doc! {"$set":{"deleted_at":now,"updated_at":now,"updated_by":&actor.name,"updated_by_id":&actor.id}}).await?;
    let Some(old) = old else {
        record(state, id).await?;
        return Err(AppError::RecordConflict);
    };
    cleanup(state, &files(&old)).await;
    Ok(())
}
async fn read_file(
    state: &AppState,
    row: &Document,
    variant: &str,
) -> Result<UploadedImage, AppError> {
    let field = match variant {
        "original" => "original_file_id",
        "preview" => "preview_file_id",
        _ => return Err(AppError::NotFound),
    };
    let id = row.get_object_id(field).map_err(|_| AppError::NotFound)?;
    let bucket = state.db.gridfs_bucket(None);
    let metadata = doc! {"_id":id,"metadata.shop_id":&state.config.shop_id,"metadata.channel_id":&state.config.channel_id,"metadata.quick_reply_id":row.get_object_id("_id").map_err(|_| AppError::Data)?,"metadata.source":"quick_reply"};
    bucket.find_one(metadata).await?.ok_or(AppError::NotFound)?;
    let mut stream = bucket.open_download_stream(Bson::ObjectId(id)).await?;
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).await?;
    let mime = image_type(&bytes).ok_or(AppError::Data)?;
    Ok(UploadedImage { bytes, mime })
}
pub async fn image(
    state: &AppState,
    id: ObjectId,
    variant: &str,
) -> Result<UploadedImage, AppError> {
    read_file(state, &record(state, id).await?, variant).await
}
pub async fn prepare_image(
    state: &AppState,
    inquiry: ObjectId,
    id: ObjectId,
    version: i64,
    actor: &Actor,
) -> Result<Document, AppError> {
    // Check the scoped conversation before reading any reusable image bytes.
    drafts::inquiry(state, inquiry).await?;
    let row = record(state, id).await?;
    if row.get_i64("version").ok() != Some(version) {
        return Err(AppError::RecordConflict);
    }
    let original = read_file(state, &row, "original").await?;
    let preview = read_file(state, &row, "preview").await?;
    outbound::upload_images(state, inquiry, (original, preview), actor).await
}
