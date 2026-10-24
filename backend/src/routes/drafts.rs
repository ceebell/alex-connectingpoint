use crate::{db::drafts, error::AppError, models::draft::PrepareInput, state::AppState};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use mongodb::bson::{Document, oid::ObjectId};
use serde_json::{Value, json};

fn id(value: &str) -> Result<ObjectId, AppError> {
    ObjectId::parse_str(value).map_err(|_| AppError::BadRequest)
}

fn job_view(record: &Document) -> Result<Value, AppError> {
    Ok(
        json!({"id":record.get_object_id("_id").map_err(|_|AppError::Data)?.to_hex(),
        "status":record.get_str("status").unwrap_or("PENDING"), "error_code":record.get_str("error_code").ok()}),
    )
}

pub async fn prepare(
    State(state): State<AppState>,
    Path(inquiry): Path<String>,
    Json(input): Json<PrepareInput>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    if input.fields.is_some() || !input.confirmed_fields.is_empty() {
        return Err(AppError::BadRequest);
    }
    Ok((
        StatusCode::ACCEPTED,
        Json(job_view(
            &drafts::enqueue(&state, id(&inquiry)?, input, false).await?,
        )?),
    ))
}

pub async fn revise(
    State(state): State<AppState>,
    Path(inquiry): Path<String>,
    Json(input): Json<PrepareInput>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    if input.fields.is_none() {
        return Err(AppError::BadRequest);
    }
    Ok((
        StatusCode::ACCEPTED,
        Json(job_view(
            &drafts::enqueue(&state, id(&inquiry)?, input, true).await?,
        )?),
    ))
}

pub async fn job(
    State(state): State<AppState>,
    Path(job): Path<String>,
) -> Result<Json<Value>, AppError> {
    let mut filter = state.scope();
    filter.insert("_id", id(&job)?);
    let record = state
        .collection("draft_requests")
        .find_one(filter)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(job_view(&record)?))
}

pub async fn list(
    State(state): State<AppState>,
    Path(inquiry): Path<String>,
) -> Result<Json<Value>, AppError> {
    let inquiry = drafts::inquiry(&state, id(&inquiry)?).await?;
    Ok(Json(
        json!({"version":drafts::version(&inquiry),"drafts":drafts::safe_drafts(&state,&inquiry).await?}),
    ))
}
