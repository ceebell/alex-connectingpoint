use crate::{
    db::product_favorites,
    error::AppError,
    models::product_favorites::{ListQuery, SaveInput},
};
use axum::{
    Json,
    extract::{
        Path, Query,
        rejection::{JsonRejection, QueryRejection},
    },
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
    query: Result<Query<ListQuery>, QueryRejection>,
) -> Result<Json<Value>, AppError> {
    let Query(query) = query.map_err(|_| AppError::BadRequest)?;
    Ok(Json(
        product_favorites::list(&state, id(&inquiry)?, &query.branch_id).await?,
    ))
}

pub async fn save(
    super::access::ScopedState(state): super::access::ScopedState,
    Path(inquiry): Path<String>,
    headers: HeaderMap,
    input: Result<Json<SaveInput>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    Ok(Json(
        product_favorites::save(
            &state,
            id(&inquiry)?,
            input,
            &super::session::actor(&state, &headers),
        )
        .await?,
    ))
}

pub async fn remove(
    super::access::ScopedState(state): super::access::ScopedState,
    Path((inquiry, favorite)): Path<(String, String)>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(
        product_favorites::remove(&state, id(&inquiry)?, id(&favorite)?).await?,
    ))
}

pub async fn add_to_cart(
    super::access::ScopedState(state): super::access::ScopedState,
    Path((inquiry, favorite)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
    Ok(Json(
        product_favorites::add_to_cart(
            &state,
            id(&inquiry)?,
            id(&favorite)?,
            &super::session::actor(&state, &headers),
        )
        .await?,
    ))
}
