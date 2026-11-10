use crate::{
    db::{inquiries, media as media_db},
    error::AppError,
    state::AppState,
};
use axum::{
    Json,
    body::Body,
    extract::{Path, Query, Request, State, rejection::JsonRejection},
    http::{HeaderMap, HeaderValue, StatusCode, header},
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
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    if super::session::valid_cookie(&state, request.headers()) {
        if !matches!(
            *request.method(),
            axum::http::Method::GET | axum::http::Method::HEAD
        ) {
            super::session::require_browser_header(request.headers())?;
        }
        if crate::db::team::enabled(&state) {
            let identity = super::session::identity(&state, request.headers())
                .ok_or(AppError::Unauthorized)?;
            if !identity.is_line() {
                return Err(AppError::Forbidden);
            }
            let access = crate::db::team::access(&state, &identity.id, &identity.name).await?;
            let path = request
                .extensions()
                .get::<axum::extract::MatchedPath>()
                .ok_or(AppError::Data)?
                .as_str();
            let mutation = !matches!(
                *request.method(),
                axum::http::Method::GET | axum::http::Method::HEAD
            );
            if (path.starts_with("/api/admin/team")
                || path.ends_with("/assign")
                || path.ends_with("/retry"))
                && access.role != crate::models::team::Role::Owner
            {
                return Err(AppError::Forbidden);
            }
            if mutation
                && (path.ends_with("/checkout/submit")
                    || path.ends_with("/checkout/resolve")
                    || path.ends_with("/order"))
                && !access.can_submit()
            {
                return Err(AppError::Forbidden);
            }
            request.extensions_mut().insert(access);
        }
        let mut response = next.run(request).await;
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
        return Ok(response);
    }
    require_token(&state, request.headers())?;
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-store"),
    );
    Ok(response)
}

pub fn require_token(state: &AppState, headers: &HeaderMap) -> Result<(), AppError> {
    let token = headers
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
    Ok(())
}

fn object_id(value: &str) -> Result<ObjectId, AppError> {
    ObjectId::parse_str(value).map_err(|_| AppError::BadRequest)
}

#[derive(Default, Deserialize)]
pub struct ListQuery {
    pub id: Option<String>,
    pub status: Option<String>,
    pub before: Option<String>,
    pub before_at: Option<String>,
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
    let before = query.before.as_deref().map(object_id).transpose()?;
    let before_at = query
        .before_at
        .as_deref()
        .map(|at| at.parse::<i64>().map_err(|_| AppError::BadRequest))
        .transpose()?;
    let mut inquiries = if let Some(id) = query.id.as_deref() {
        inquiries::by_id(&state, object_id(id)?).await?
    } else {
        inquiries::list(&state, query.status.as_deref(), before, before_at).await?
    };
    let ids: Vec<String> = inquiries
        .iter()
        .filter_map(|i| i.get_str("line_user_id").ok().map(String::from))
        .collect();
    let aliases = crate::db::customers::aliases(&state, ids.clone()).await?;
    let profiles = crate::db::users::profiles(&state, ids).await?;
    // Users seen before profile caching get it fetched in the background once.
    let missing: Vec<String> = profiles
        .iter()
        .filter(|(_, p)| p.get_datetime("profile_checked_at").is_err())
        .map(|(id, _)| id.clone())
        .take(20)
        .collect();
    if !missing.is_empty() {
        let background = state.clone();
        tokio::spawn(async move {
            for id in missing {
                if crate::db::users::refresh_profile(&background, &id)
                    .await
                    .is_err()
                {
                    tracing::warn!("LINE profile refresh failed");
                }
            }
        });
    }
    for inquiry in &mut inquiries {
        if let Some(profile) = inquiry
            .get_str("line_user_id")
            .ok()
            .and_then(|id| profiles.get(id))
        {
            let alias = inquiry
                .get_str("line_user_id")
                .ok()
                .and_then(|id| aliases.get(id));
            let custom_name = alias.and_then(|record| record.get_str("custom_name").ok());
            let line_name = profile.get_str("display_name").ok();
            inquiry.insert(
                "customer",
                doc! {"display_name":custom_name.or(line_name),"line_display_name":line_name,"custom_name":custom_name,"name_version":alias.and_then(|record|record.get_i64("version").ok()).unwrap_or(0),"picture_url":profile.get_str("picture_url").ok()},
            );
        }
    }
    crate::db::inbox::decorate(&state, &mut inquiries).await?;
    Ok(Json(inquiries))
}

pub async fn events(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Document>>, AppError> {
    let mut events = page(&state, "line_events", query).await?;
    // Suppress unsent content immediately, even while the purge worker is waiting.
    crate::db::events::redact_withdrawn(&state, &mut events).await?;
    crate::db::events::hide_read_tokens(&mut events);
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
    let after = query.after.as_deref().map(object_id).transpose()?;
    let mut messages = crate::db::oa_sync::messages(&state, inquiry_id, after).await?;
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
    crate::db::events::hide_read_tokens(&mut visible);
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
        "created_at":now, "updated_at":now,"last_activity_at":now});
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
