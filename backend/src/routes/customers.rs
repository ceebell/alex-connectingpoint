use crate::{
    db::{customers, orders, outbound},
    error::AppError,
    models::{
        customer::{ApproveInput, ProfileInput, valid_user_id},
        outbound::SendInput,
    },
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode},
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
