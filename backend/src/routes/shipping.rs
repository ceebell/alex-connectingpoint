use crate::{
    db::shipping,
    error::AppError,
    models::shipping::{SaveInput, UploadInput, VersionInput},
};
use axum::{
    Json,
    extract::{Path, Query, rejection::JsonRejection},
    http::{HeaderMap, header},
    response::{IntoResponse, Response},
};
use mongodb::bson::oid::ObjectId;
use serde::Deserialize;
use serde_json::Value;

fn id(value: &str) -> Result<ObjectId, AppError> {
    ObjectId::parse_str(value).map_err(|_| AppError::BadRequest)
}
#[derive(Deserialize)]
pub struct Search {
    pub query: Option<String>,
    pub before: Option<String>,
}
pub async fn list(
    super::access::ScopedState(state): super::access::ScopedState,
    Query(search): Query<Search>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(
        shipping::list(&state, search.before.map(|s| id(&s)).transpose()?).await?,
    ))
}
pub async fn upload(
    super::access::ScopedState(state): super::access::ScopedState,
    headers: HeaderMap,
    input: Result<Json<UploadInput>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    Ok(Json(
        shipping::upload(&state, input, &super::session::actor(&state, &headers)).await?,
    ))
}
pub async fn get(
    super::access::ScopedState(state): super::access::ScopedState,
    Path(receipt): Path<String>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(shipping::get(&state, id(&receipt)?).await?))
}
pub async fn image(
    super::access::ScopedState(state): super::access::ScopedState,
    Path(receipt): Path<String>,
) -> Result<Response, AppError> {
    let (mime, bytes) = shipping::image(&state, id(&receipt)?).await?;
    Ok((
        [
            (header::CONTENT_TYPE, mime),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff".into()),
        ],
        bytes,
    )
        .into_response())
}
pub async fn extract(
    super::access::ScopedState(state): super::access::ScopedState,
    Path(receipt): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
    Ok(Json(
        shipping::extract_again(
            &state,
            id(&receipt)?,
            &super::session::actor(&state, &headers),
        )
        .await?,
    ))
}
pub async fn candidates(
    super::access::ScopedState(state): super::access::ScopedState,
    Query(search): Query<Search>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(
        shipping::candidates(&state, search.query.as_deref().unwrap_or("")).await?,
    ))
}
pub async fn save(
    super::access::ScopedState(state): super::access::ScopedState,
    Path((receipt, row)): Path<(String, i64)>,
    headers: HeaderMap,
    input: Result<Json<SaveInput>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    Ok(Json(
        shipping::save(
            &state,
            id(&receipt)?,
            row,
            input,
            &super::session::actor(&state, &headers),
        )
        .await?,
    ))
}
pub async fn send(
    super::access::ScopedState(state): super::access::ScopedState,
    Path(tracking): Path<String>,
    headers: HeaderMap,
    input: Result<Json<VersionInput>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    Ok(Json(
        shipping::send(
            &state,
            id(&tracking)?,
            input.expected_version,
            &super::session::actor(&state, &headers),
        )
        .await?,
    ))
}
