use crate::{
    db::checkout,
    error::AppError,
    models::checkout::{CheckInput, ResolveInput, ReviewInput, SubmitInput},
};
use axum::{
    Json,
    extract::{Path, rejection::JsonRejection},
    http::HeaderMap,
};
use mongodb::bson::oid::ObjectId;
use serde_json::Value;
fn id(value: &str) -> Result<ObjectId, AppError> {
    ObjectId::parse_str(value).map_err(|_| AppError::BadRequest)
}
pub async fn list(
    super::access::ScopedState(state): super::access::ScopedState,
    Path(inquiry): Path<String>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(checkout::list(&state, id(&inquiry)?).await?))
}
pub async fn availability(
    super::access::ScopedState(state): super::access::ScopedState,
    Path(inquiry): Path<String>,
    input: Result<Json<CheckInput>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    Ok(Json(
        checkout::availability(&state, id(&inquiry)?, input).await?,
    ))
}
pub async fn review(
    super::access::ScopedState(state): super::access::ScopedState,
    Path(inquiry): Path<String>,
    input: Result<Json<ReviewInput>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    Ok(Json(checkout::review(&state, id(&inquiry)?, input).await?))
}
pub async fn submit(
    super::access::ScopedState(state): super::access::ScopedState,
    Path(inquiry): Path<String>,
    headers: HeaderMap,
    input: Result<Json<SubmitInput>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    let actor = super::session::actor(&state, &headers);
    Ok(Json(
        checkout::submit(&state, id(&inquiry)?, input, &actor).await?,
    ))
}
pub async fn resolve(
    super::access::ScopedState(state): super::access::ScopedState,
    Path(inquiry): Path<String>,
    headers: HeaderMap,
    input: Result<Json<ResolveInput>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    let actor = super::session::actor(&state, &headers);
    Ok(Json(
        checkout::resolve(&state, id(&inquiry)?, input, &actor).await?,
    ))
}
