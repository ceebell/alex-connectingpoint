use crate::{
    db,
    error::AppError,
    models::oa_inbox::{OaInboxBatch, RefreshResult},
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
    http::HeaderMap,
};
use mongodb::bson::oid::ObjectId;
use serde_json::{Value, json};

fn authorize(state: &AppState, headers: &HeaderMap) -> Result<(), AppError> {
    if !state.config.oa_connector_enabled {
        return Err(AppError::NotFound);
    }
    super::admin::require_token(state, headers)
}

pub async fn ingest(
    State(state): State<AppState>,
    headers: HeaderMap,
    input: Result<Json<OaInboxBatch>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    authorize(&state, &headers)?;
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    Ok(Json(db::oa_inbox::ingest(&state, input).await?))
}

pub async fn pending(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
    authorize(&state, &headers)?;
    Ok(Json(db::oa_inbox::pending(&state).await?))
}

pub async fn finish(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    input: Result<Json<RefreshResult>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    authorize(&state, &headers)?;
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    let id = ObjectId::parse_str(id).map_err(|_| AppError::BadRequest)?;
    db::oa_inbox::finish(&state, id, input).await?;
    Ok(Json(json!({"status":"ok"})))
}

pub async fn refresh(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    Ok(Json(db::oa_inbox::request_refresh(&state).await?))
}

pub async fn status(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, AppError> {
    let id = ObjectId::parse_str(id).map_err(|_| AppError::BadRequest)?;
    Ok(Json(db::oa_inbox::refresh_status(&state, id).await?))
}
