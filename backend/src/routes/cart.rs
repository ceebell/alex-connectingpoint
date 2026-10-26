use crate::{
    db::{cart, checkout},
    error::AppError,
    models::cart::{AddInput, CheckInput, UpdateInput},
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
    http::HeaderMap,
};
use mongodb::bson::oid::ObjectId;
use serde_json::Value;

fn id(value: &str) -> Result<ObjectId, AppError> {
    ObjectId::parse_str(value).map_err(|_| AppError::BadRequest)
}

pub async fn list(
    State(state): State<AppState>,
    Path(inquiry): Path<String>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(cart::list(&state, id(&inquiry)?).await?))
}

pub async fn add(
    State(state): State<AppState>,
    Path(inquiry): Path<String>,
    headers: HeaderMap,
    input: Result<Json<AddInput>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    let actor = super::session::actor(&state, &headers);
    Ok(Json(cart::add(&state, id(&inquiry)?, input, &actor).await?))
}

pub async fn update(
    State(state): State<AppState>,
    Path(inquiry): Path<String>,
    input: Result<Json<UpdateInput>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    Ok(Json(cart::update(&state, id(&inquiry)?, input).await?))
}

pub async fn availability(
    State(state): State<AppState>,
    Path(inquiry): Path<String>,
    input: Result<Json<CheckInput>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    Ok(Json(
        checkout::cart_availability(&state, id(&inquiry)?, input).await?,
    ))
}
