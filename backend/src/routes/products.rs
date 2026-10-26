use crate::{
    alex,
    db::products,
    error::AppError,
    models::product::{ConfirmInput, SearchInput},
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
    http::{HeaderMap, HeaderValue, header},
    response::{IntoResponse, Response},
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

pub async fn confirm(
    State(state): State<AppState>,
    Path(inquiry): Path<String>,
    headers: HeaderMap,
    input: Result<Json<ConfirmInput>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    let actor = super::session::actor(&state, &headers);
    Ok(Json(
        products::confirm(&state, id(&inquiry)?, input, &actor).await?,
    ))
}

pub async fn image(
    State(state): State<AppState>,
    Path((inquiry, search, product)): Path<(String, String, String)>,
) -> Result<Response, AppError> {
    let (mime, bytes) = products::image(&state, id(&inquiry)?, id(&search)?, &product).await?;
    let mut response = bytes.into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&mime).map_err(|_| AppError::Data)?,
    );
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    Ok(response)
}
