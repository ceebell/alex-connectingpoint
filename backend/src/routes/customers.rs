use crate::{
    db::{customers, orders, outbound},
    error::AppError,
    models::{
        customer::{ApproveInput, ProfileInput, valid_user_id},
        outbound::{SendInput, UploadInput},
    },
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use mongodb::bson::oid::ObjectId;
use serde_json::{Value, json};

fn user(value: &str) -> Result<&str, AppError> {
    valid_user_id(value)
        .then_some(value)
        .ok_or(AppError::BadRequest)
}

fn inquiry(value: &str) -> Result<ObjectId, AppError> {
    ObjectId::parse_str(value).map_err(|_| AppError::BadRequest)
}

pub async fn profile(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    let profile = customers::get(&state, user(&user_id)?).await?;
    Ok(Json(json!({"line_user_id":user_id,"profile":profile})))
}

pub async fn save_profile(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    headers: HeaderMap,
    input: Result<Json<ProfileInput>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    let actor = super::session::actor(&state, &headers);
    let profile = customers::save(&state, user(&user_id)?, input.validate()?, &actor).await?;
    Ok(Json(json!({"line_user_id":user_id,"profile":profile})))
}

pub async fn order(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(
        json!({"order":orders::get(&state, inquiry(&id)?).await?}),
    ))
}

pub async fn approve(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    input: Result<Json<ApproveInput>, JsonRejection>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    let actor = super::session::actor(&state, &headers);
    let order = orders::approve(&state, inquiry(&id)?, input, &actor).await?;
    Ok((StatusCode::CREATED, Json(json!({"order":order}))))
}

pub async fn send_message(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    input: Result<Json<SendInput>, JsonRejection>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    let actor = super::session::actor(&state, &headers);
    let message = outbound::send(&state, inquiry(&id)?, input, &actor).await?;
    Ok((StatusCode::CREATED, Json(json!({"message":message}))))
}

pub async fn upload_media(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    input: Result<Json<UploadInput>, JsonRejection>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    let actor = super::session::actor(&state, &headers);
    let media = outbound::upload(&state, inquiry(&id)?, input, &actor).await?;
    Ok((StatusCode::CREATED, Json(json!({"media":media}))))
}

// Public by design: LINE downloads sent images from this unguessable URL.
pub async fn outbound_media(
    State(state): State<AppState>,
    Path((token, variant)): Path<(String, String)>,
) -> Result<Response, AppError> {
    let (mime, bytes) = outbound::read(&state, &token, &variant).await?;
    let mut response = bytes.into_response();
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&mime).map_err(|_| AppError::Data)?,
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=3600"),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    Ok(response)
}
