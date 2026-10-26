use crate::{alex, db::products, error::AppError, models::product::SearchInput, state::AppState};
use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
};
use mongodb::bson::oid::ObjectId;
use serde_json::{Value, json};

fn id(value: &str) -> Result<ObjectId, AppError> {
    ObjectId::parse_str(value).map_err(|_| AppError::BadRequest)
}

pub async fn settings(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let configured = state.config.alex.is_some();
    let branches = if configured {
        alex::login(&state).await?.branches
    } else {
        vec![]
    };
    Ok(Json(json!({"configured":configured,"branches":branches,
        "image_search_configured":state.config.hermes_url.is_some() && state.config.hermes_api_key.is_some(),
        "product_url_configured":state.config.alex.as_ref().is_some_and(|cfg| cfg.product_url_prefix.is_some())})))
}

pub async fn categories(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let session = alex::login(&state).await?;
    Ok(Json(
        json!({"categories":alex::categories(&state, &session).await?}),
    ))
}

pub async fn list(
    State(state): State<AppState>,
    Path(inquiry): Path<String>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(products::list(&state, id(&inquiry)?).await?))
}

pub async fn search(
    State(state): State<AppState>,
    Path(inquiry): Path<String>,
    input: Result<Json<SearchInput>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    Ok(Json(products::search(&state, id(&inquiry)?, input).await?))
}
