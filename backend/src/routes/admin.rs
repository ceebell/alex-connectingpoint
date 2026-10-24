use crate::{
    db::{inquiries, media as media_db},
    error::AppError,
    state::AppState,
};
use axum::{
    Json,
    body::Body,
    extract::{Path, Query, Request, State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use futures_util::TryStreamExt;
use hmac::{Hmac, Mac};
use mongodb::bson::{DateTime, Document, doc, oid::ObjectId};
use serde::Deserialize;
use sha2::Sha256;

pub async fn authorize(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;
    let mut expected =
        Hmac::<Sha256>::new_from_slice(b"inquiry-admin-auth").map_err(|_| AppError::Data)?;
    expected.update(state.config.admin_token.as_bytes());
    let mut candidate =
        Hmac::<Sha256>::new_from_slice(b"inquiry-admin-auth").map_err(|_| AppError::Data)?;
    candidate.update(token.as_bytes());
    candidate
        .verify_slice(&expected.finalize().into_bytes())
        .map_err(|_| AppError::Unauthorized)?;
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-store"),
    );
    Ok(response)
}

fn object_id(value: &str) -> Result<ObjectId, AppError> {
    ObjectId::parse_str(value).map_err(|_| AppError::BadRequest)
}

#[derive(Default, Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
    pub before: Option<String>,
}

async fn page(
    state: &AppState,
    collection: &str,
    query: ListQuery,
) -> Result<Vec<Document>, AppError> {
    let mut filter = state.scope();
    if let Some(status) = query.status {
        filter.insert("status", status);
    }
    if let Some(before) = query.before {
        filter.insert("_id", doc! {"$lt":object_id(&before)?});
    }
    Ok(state
        .collection(collection)
        .find(filter)
        .sort(doc! {"_id":-1})
        .limit(100)
        .projection(doc! {"lease_token":0})
        .await?
        .try_collect()
        .await?)
}

pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Document>>, AppError> {
    Ok(Json(page(&state, "inquiries", query).await?))
}

pub async fn events(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Document>>, AppError> {
    let mut events = page(&state, "line_events", query).await?;
    // Suppress unsent content immediately, even while the purge worker is waiting.
    crate::db::events::redact_withdrawn(&state, &mut events).await?;
    Ok(Json(events))
}

#[derive(Deserialize)]
pub struct ContextQuery {
    pub after: Option<String>,
}

pub async fn context(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<ContextQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let inquiry_id = object_id(&id)?;
    let mut filter = state.scope();
    filter.insert("_id", inquiry_id);
    let inquiry = state
        .collection("inquiries")
        .find_one(filter)
        .await?
        .ok_or(AppError::NotFound)?;
    let mut filter = state.scope();
    filter.extend(doc! {"inquiry_id":inquiry_id, "status":"DONE", "withdrawn":{"$ne":true}});
    // Use an opaque event reference as a stable (event_at, event_id) cursor.
    if let Some(after) = query.after {
        let mut cursor_filter = state.scope();
        cursor_filter.extend(doc! {"_id":object_id(&after)?, "inquiry_id":inquiry_id});
        let cursor = state
            .collection("line_events")
            .find_one(cursor_filter)
            .await?
            .ok_or(AppError::NotFound)?;
        let at = *cursor
            .get_datetime("event_at")
            .map_err(|_| AppError::Data)?;
        let event_id = cursor.get_str("event_id").map_err(|_| AppError::Data)?;
        filter.insert(
            "$or",
            vec![
                doc! {"event_at":{"$gt":at}},
                doc! {"event_at":at, "event_id":{"$gt":event_id}},
            ],
        );
    }
    let mut messages: Vec<Document> = state
        .collection("line_events")
        .find(filter)
        .sort(doc! {"event_at":1, "event_id":1})
        .limit(201)
        .projection(doc! {"lease_token":0})
        .await?
        .try_collect()
        .await?;
    let has_more = messages.len() > 200;
    messages.truncate(200);
    let next = if has_more {
        messages
            .last()
            .and_then(|m| m.get_object_id("_id").ok())
            .map(|id| id.to_hex())
    } else {
        None
    };
    let mut visible = Vec::new();
    for message in messages {
        if let Ok(id) = message.get_str("message_id")
            && media_db::withdrawn(&state, id).await?
        {
            continue;
        }
        visible.push(message);
    }
    Ok(Json(
        serde_json::json!({"inquiry":inquiry, "batches":inquiries::batches(visible, state.config.burst_ms), "next":next}),
    ))
}

pub async fn create(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<Document>), AppError> {
    let conversation = object_id(&id)?;
    let mut filter = state.scope();
    filter.insert("_id", conversation);
    let conversation_doc = state
        .collection("conversations")
        .find_one(filter)
        .await?
        .ok_or(AppError::NotFound)?;
    let mut inquiry = state.scope();
    let now = DateTime::now();
    inquiry.extend(doc! {"_id":ObjectId::new(), "conversation_id":conversation,
        "line_user_id":conversation_doc.get_str("line_user_id").ok(), "status":"RECEIVED", "mode":"ADMIN_ASSIST",
        "created_at":now, "updated_at":now});
    state
        .collection("inquiries")
        .insert_one(inquiry.clone())
        .await?;
    Ok((StatusCode::CREATED, Json(inquiry)))
}

#[derive(Deserialize)]
pub struct Assignment {
    pub inquiry_id: String,
}

pub async fn assign(
    State(state): State<AppState>,
    Path(id): Path<String>,
    body: Result<Json<Assignment>, JsonRejection>,
) -> Result<Json<serde_json::Value>, AppError> {
    let Json(body) = body.map_err(|_| AppError::BadRequest)?;
    let inquiry_id = object_id(&body.inquiry_id)?;
    let mut filter = state.scope();
    filter.insert("_id", object_id(&id)?);
    let event = state
        .collection("line_events")
        .find_one(filter.clone())
        .await?
        .ok_or(AppError::NotFound)?;
    if event.get_str("status").ok() != Some("NEEDS_ADMIN") {
        return Err(AppError::Conflict);
    }
    let mut target = state.scope();
    target.extend(doc! {"_id":inquiry_id,
        "conversation_id":event.get_object_id("conversation_id").map_err(|_| AppError::Conflict)?, "status":{"$ne":"CLOSED"}});
    if state
        .collection("inquiries")
        .find_one(target)
        .await?
        .is_none()
    {
        return Err(AppError::NotFound);
    }
    filter.insert("status", "NEEDS_ADMIN");
    let result = state.collection("line_events").update_one(filter, doc! {"$set":{
        "inquiry_id":inquiry_id, "status":"PENDING", "available_at":DateTime::now(), "updated_at":DateTime::now()}}).await?;
    if result.modified_count == 0 {
        return Err(AppError::Conflict);
    }
    Ok(Json(serde_json::json!({"status":"ok"})))
}

pub async fn retry(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut filter = state.scope();
    filter.extend(doc! {"_id":object_id(&id)?, "status":"FAILED"});
    let result = state.collection("line_events").update_one(filter, doc! {"$set":{
        "status":"PENDING", "attempts":0, "available_at":DateTime::now(), "updated_at":DateTime::now()}}).await?;
    if result.modified_count == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(serde_json::json!({"status":"ok"})))
}

pub async fn status(State(state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let mut counts = serde_json::Map::new();
    for status in [
        "PENDING",
        "PROCESSING",
        "FAILED",
        "NEEDS_ADMIN",
        "DONE",
        "IGNORED",
    ] {
        let mut filter = state.scope();
        filter.insert("status", status);
        counts.insert(
            status.into(),
            state
                .collection("line_events")
                .count_documents(filter)
                .await?
                .into(),
        );
    }
    Ok(Json(serde_json::json!({"events":counts})))
}

pub async fn media(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let (mime, bytes) = media_db::read(&state, object_id(&id)?).await?;
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        mime.parse().map_err(|_| AppError::Data)?,
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        header::HeaderValue::from_static("nosniff"),
    );
    Ok((headers, Body::from(bytes)).into_response())
}
