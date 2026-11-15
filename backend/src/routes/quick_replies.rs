use crate::{
    db::quick_replies,
    error::AppError,
    models::quick_replies::{CreateInput, EditInput, VersionInput},
};
use axum::{
    Json,
    extract::{Path, Query, rejection::JsonRejection},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use mongodb::bson::oid::ObjectId;
use serde::Deserialize;
use serde_json::{Value, json};

fn id(value: &str) -> Result<ObjectId, AppError> {
    ObjectId::parse_str(value).map_err(|_| AppError::BadRequest)
}
#[derive(Deserialize)]
pub struct Page {
    before: Option<String>,
}
pub async fn list(
    super::access::ScopedState(state): super::access::ScopedState,
    Query(page): Query<Page>,
) -> Result<Json<Value>, AppError> {
    let (items, next) =
        quick_replies::list(&state, page.before.as_deref().map(id).transpose()?).await?;
    Ok(Json(json!({"items":items,"next":next})))
}
pub async fn create(
    super::access::ScopedState(state): super::access::ScopedState,
    headers: HeaderMap,
    input: Result<Json<CreateInput>, JsonRejection>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    let (request, content) = input.validate()?;
    let actor = super::session::actor(&state, &headers);
    let item = quick_replies::create(&state, &request, content, &actor).await?;
    Ok((StatusCode::CREATED, Json(json!({"item":item}))))
}
pub async fn edit(
    super::access::ScopedState(state): super::access::ScopedState,
    Path(reply): Path<String>,
    headers: HeaderMap,
    input: Result<Json<EditInput>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    let (version, content) = input.validate()?;
    let actor = super::session::actor(&state, &headers);
    let item = quick_replies::edit(&state, id(&reply)?, version, content, &actor).await?;
    Ok(Json(json!({"item":item})))
}
pub async fn delete(
    super::access::ScopedState(state): super::access::ScopedState,
    Path(reply): Path<String>,
    headers: HeaderMap,
    input: Result<Json<VersionInput>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    input.validate()?;
    quick_replies::delete(
        &state,
        id(&reply)?,
        input.expected_version,
        &super::session::actor(&state, &headers),
    )
    .await?;
    Ok(Json(json!({"deleted":true})))
}
pub async fn image(
    super::access::ScopedState(state): super::access::ScopedState,
    Path((reply, variant)): Path<(String, String)>,
) -> Result<Response, AppError> {
    let image = quick_replies::image(&state, id(&reply)?, &variant).await?;
    let mut response = image.bytes.into_response();
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static(image.mime));
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    Ok(response)
}
pub async fn prepare_image(
    super::access::ScopedState(state): super::access::ScopedState,
    Path((inquiry, reply)): Path<(String, String)>,
    headers: HeaderMap,
    input: Result<Json<VersionInput>, JsonRejection>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    input.validate()?;
    let media = quick_replies::prepare_image(
        &state,
        id(&inquiry)?,
        id(&reply)?,
        input.expected_version,
        &super::session::actor(&state, &headers),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(json!({"media":media}))))
}
