use crate::{
    db::{customers, drafts, events::duplicate},
    error::AppError,
    models::{
        customer::Actor,
        customer_notes::{CreateInput, NoteContent, NoteImage},
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

fn scope(state: &AppState, user: &str) -> Document {
    let mut filter = state.scope();
    filter.insert("line_user_id", user);
    filter
}

fn public(mut record: Document) -> Document {
    record.remove("content_hash");
    record.remove("request_id");
    record
}

pub async fn list(
    state: &AppState,
    user: &str,
    before: Option<ObjectId>,
) -> Result<(Vec<Document>, Option<String>), AppError> {
    customers::ensure_user(state, user).await?;
    let mut filter = scope(state, user);
    if let Some(before) = before {
        let cursor = get(state, user, before).await?;
        let at = *cursor
            .get_datetime("created_at")
            .map_err(|_| AppError::Data)?;
        filter.insert(
            "$or",
            vec![
                doc! {"created_at":{"$lt":at}},
                doc! {"created_at":at,"_id":{"$lt":before}},
            ],
        );
    }
    let mut records: Vec<Document> = state
        .collection("customer_notes")
        .find(filter)
        .sort(doc! {"created_at":-1,"_id":-1})
        .projection(doc! {"history":0,"content_hash":0,"request_id":0})
        .limit(51)
        .await?
        .try_collect()
        .await?;
    let next = if records.len() > 50 {
        records.truncate(50);
        Some(
            records[49]
                .get_object_id("_id")
                .map_err(|_| AppError::Data)?
                .to_hex(),
        )
    } else {
        None
    };
    Ok((records, next))
}

pub async fn get(state: &AppState, user: &str, id: ObjectId) -> Result<Document, AppError> {
    let mut filter = scope(state, user);
    filter.insert("_id", id);
    state
        .collection("customer_notes")
        .find_one(filter)
        .await?
        .map(public)
        .ok_or(AppError::NotFound)
}

fn image_filter(state: &AppState, user: &str, image: ObjectId) -> Document {
    doc! {"_id":image,"metadata.shop_id":&state.config.shop_id,"metadata.channel_id":&state.config.channel_id,"metadata.line_user_id":user,"metadata.source":"customer_note"}
}

async fn cleanup(state: &AppState, ids: &[ObjectId]) {
    for id in ids {
        let _ = state
            .db
            .gridfs_bucket(None)
            .delete(Bson::ObjectId(*id))
            .await;
    }
}

async fn store_images(
    state: &AppState,
    user: &str,
    note: ObjectId,
    images: Vec<NoteImage>,
) -> Result<(Vec<Document>, Vec<ObjectId>), AppError> {
    let mut stored = Vec::new();
    let mut created = Vec::new();
    let bucket = state.db.gridfs_bucket(None);
    for image in images {
        let result: Result<Document, AppError> = async {
            let (id, name) = match image {
                NoteImage::Existing(id) => {
                    let mut filter = image_filter(state, user, id);
                    filter.insert("metadata.note_id", note);
                    let file = bucket.find_one(filter).await?.ok_or(AppError::NotFound)?;
                    let metadata = file.metadata.ok_or(AppError::Data)?;
                    (id, metadata.get_str("name").map_err(|_| AppError::Data)?.to_string())
                }
                NoteImage::New { name, bytes, mime } => {
                    let now = DateTime::now();
                    let mut metadata = scope(state, user);
                    // Customer notes are permanent records; LINE media expiry does not apply.
                    metadata.extend(doc! {"source":"customer_note","note_id":note,"name":&name,"content_type":mime,"created_at":now,"updated_at":now});
                    let mut upload = bucket.open_upload_stream("customer-note-image").metadata(metadata).await?;
                    let id = upload.id().as_object_id().ok_or(AppError::Data)?;
                    if let Err(error) = upload.write_all(&bytes).await { let _ = upload.abort().await; return Err(error.into()); }
                    created.push(id);
                    upload.close().await?;
                    (id, name)
                }
            };
            Ok(doc! {"id":id.to_hex(),"name":name,"url":format!("/api/line/users/{user}/note-images/{id}")})
        }.await;
        match result {
            Ok(image) => stored.push(image),
            Err(error) => {
                cleanup(state, &created).await;
                return Err(error);
            }
        }
    }
    Ok((stored, created))
}

fn revision(
    version: i64,
    text: &str,
    images: &[Document],
    actor: &Actor,
    now: DateTime,
) -> Document {
    doc! {"version":version,"text":text,"images":images.to_vec(),"updated_by":&actor.name,"updated_by_id":&actor.id,"updated_at":now}
}

pub async fn create(
    state: &AppState,
    user: &str,
    input: &CreateInput,
    content: NoteContent,
    actor: &Actor,
) -> Result<Document, AppError> {
    customers::ensure_user(state, user).await?;
    let hash = drafts::hash(&serde_json::json!({"text":&content.text,"images":&input.images}))?;
    let mut identity = scope(state, user);
    identity.insert("request_id", input.request_id.to_ascii_lowercase());
    if let Some(existing) = state
        .collection("customer_notes")
        .find_one(identity.clone())
        .await?
    {
        return if existing.get_str("content_hash").ok() == Some(hash.as_str()) {
            Ok(public(existing))
        } else {
            Err(AppError::RecordConflict)
        };
    }
    let id = ObjectId::new();
    let (images, created) = store_images(state, user, id, content.images).await?;
    let now = DateTime::now();
    let mut record = identity.clone();
    record.extend(doc! {"_id":id,"version":1_i64,"text":&content.text,"images":&images,"content_hash":&hash,"created_at":now,"updated_at":now,"created_by":&actor.name,"created_by_id":&actor.id,"updated_by":&actor.name,"updated_by_id":&actor.id,"history":[revision(1,&content.text,&images,actor,now)]});
    match state
        .collection("customer_notes")
        .insert_one(record.clone())
        .await
    {
        Ok(_) => Ok(public(record)),
        Err(error) => {
            if !duplicate(&error) {
                return Err(error.into());
            }
            cleanup(state, &created).await;
            let existing = state
                .collection("customer_notes")
                .find_one(identity)
                .await?
                .ok_or(AppError::Data)?;
            if existing.get_str("content_hash").ok() == Some(hash.as_str()) {
                Ok(public(existing))
            } else {
                Err(AppError::RecordConflict)
            }
        }
    }
}

pub async fn edit(
    state: &AppState,
    user: &str,
    id: ObjectId,
    expected: i64,
    content: NoteContent,
    actor: &Actor,
) -> Result<Document, AppError> {
    let existing = get(state, user, id).await?;
    if existing.get_i64("version").ok() != Some(expected) {
        return Err(AppError::RecordConflict);
    }
    let next = expected.checked_add(1).ok_or(AppError::Data)?;
    let (images, created) = store_images(state, user, id, content.images).await?;
    let now = DateTime::now();
    let mut filter = scope(state, user);
    filter.extend(doc! {"_id":id,"version":expected});
    let update = doc! {"$set":{"version":next,"text":&content.text,"images":&images,"updated_at":now,"updated_by":&actor.name,"updated_by_id":&actor.id},"$push":{"history":revision(next,&content.text,&images,actor,now)}};
    match state
        .collection("customer_notes")
        .find_one_and_update(filter, update)
        .return_document(ReturnDocument::After)
        .await
    {
        Ok(Some(record)) => Ok(public(record)),
        Ok(None) => {
            cleanup(state, &created).await;
            Err(AppError::RecordConflict)
        }
        // A lost acknowledgement may already have committed the revision; retain its images.
        Err(error) => Err(error.into()),
    }
}

pub async fn read_image(
    state: &AppState,
    user: &str,
    id: ObjectId,
) -> Result<(String, Vec<u8>), AppError> {
    let bucket = state.db.gridfs_bucket(None);
    let file = bucket
        .find_one(image_filter(state, user, id))
        .await?
        .ok_or(AppError::NotFound)?;
    let metadata = file.metadata.ok_or(AppError::Data)?;
    let mime = metadata
        .get_str("content_type")
        .map_err(|_| AppError::Data)?
        .to_string();
    let mut stream = bucket.open_download_stream(Bson::ObjectId(id)).await?;
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).await?;
    Ok((mime, bytes))
}
