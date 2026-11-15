use crate::{db, error::AppError, models::shop_settings::SaveInput};
use axum::{Json, extract::rejection::JsonRejection};
use serde_json::Value;

pub async fn get(
    super::access::ScopedState(state): super::access::ScopedState,
) -> Result<Json<Value>, AppError> {
    db::team::require_owner(&state)?;
    Ok(Json(db::shop_settings::view(&state).await?))
}
pub async fn save(
    super::access::ScopedState(state): super::access::ScopedState,
    input: Result<Json<SaveInput>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    Ok(Json(db::shop_settings::save(&state, input).await?))
}
