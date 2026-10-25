use crate::{
    db::oa_sync,
    error::AppError,
    models::oa_sync::{OaBatch, OaHeartbeat},
    state::AppState,
};
use axum::{Json, extract::State, http::HeaderMap};
use serde_json::{Value, json};

pub async fn ingest(
    State(state): State<AppState>,
    headers: HeaderMap,
    input: Result<Json<OaBatch>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<Value>, AppError> {
    super::admin::require_token(&state, &headers)?;
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    Ok(Json(oa_sync::ingest(&state, input).await?))
}
pub async fn heartbeat(
    State(state): State<AppState>,
    headers: HeaderMap,
    input: Result<Json<OaHeartbeat>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<Value>, AppError> {
    super::admin::require_token(&state, &headers)?;
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    oa_sync::heartbeat(&state, input).await?;
    Ok(Json(json!({"status":"ok"})))
}
pub async fn status(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    Ok(Json(oa_sync::status(&state).await?))
}
