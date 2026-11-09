use crate::{db, error::AppError, models::inbox::ReadInput, state::AppState};
use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
};
use mongodb::bson::oid::ObjectId;

pub async fn mark_read(
    State(state): State<AppState>,
    Path(id): Path<String>,
    input: Result<Json<ReadInput>, JsonRejection>,
) -> Result<Json<serde_json::Value>, AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    let inquiry_id = ObjectId::parse_str(&id).map_err(|_| AppError::BadRequest)?;
    let message_id = ObjectId::parse_str(&input.message_id).map_err(|_| AppError::BadRequest)?;
    let line_status = db::inbox::mark_read(&state, inquiry_id, message_id).await?;
    let mut inquiries = vec![db::drafts::inquiry(&state, inquiry_id).await?];
    db::inbox::decorate(&state, &mut inquiries).await?;
    // Keep the response small; the inbox poll supplies names and other card metadata.
    Ok(Json(
        serde_json::json!({"unread":inquiries[0].get_bool("unread").unwrap_or(false),"line_read_status":line_status}),
    ))
}
