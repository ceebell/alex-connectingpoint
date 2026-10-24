use crate::{
    db::{events, users},
    error::AppError,
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use mongodb::bson::{Document, oid::ObjectId};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct UserQuery {
    pub before: Option<String>,
}

#[derive(Deserialize)]
pub struct MessageQuery {
    pub after: Option<String>,
}

fn cursor(value: Option<String>) -> Result<Option<ObjectId>, AppError> {
    value
        .map(|id| ObjectId::parse_str(id).map_err(|_| AppError::BadRequest))
        .transpose()
}

fn truncate_page(records: &mut Vec<Document>, limit: usize) -> Option<String> {
    let has_more = records.len() > limit;
    records.truncate(limit);
    has_more
        .then(|| {
            records
                .last()
                .and_then(|record| record.get_object_id("_id").ok())
                .map(|id| id.to_hex())
        })
        .flatten()
}

pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<UserQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut records = users::list(&state, cursor(query.before)?).await?;
    let next = truncate_page(&mut records, 100);
    Ok(Json(serde_json::json!({"users":records, "next":next})))
}

pub async fn messages(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Query(query): Query<MessageQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut records = users::messages(&state, &user_id, cursor(query.after)?).await?;
    let next = truncate_page(&mut records, 200);
    events::redact_withdrawn(&state, &mut records).await?;
    Ok(Json(
        serde_json::json!({"line_user_id":user_id, "messages":records, "next":next}),
    ))
}
