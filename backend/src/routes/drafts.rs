use crate::{
    db::drafts,
    error::AppError,
    models::draft::{PrepareInput, SummaryInput},
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
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
    input: Result<Json<PrepareInput>, axum::extract::rejection::JsonRejection>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
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

pub async fn upload_image(
    State(state): State<AppState>,
    Path(inquiry): Path<String>,
    input: Result<
        Json<crate::db::attachments::UploadInput>,
        axum::extract::rejection::JsonRejection,
    >,
) -> Result<Json<Value>, AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    Ok(Json(
        crate::db::attachments::upload(&state, id(&inquiry)?, input).await?,
    ))
}

pub async fn extract_customer(
    State(state): State<AppState>,
    Path(inquiry): Path<String>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(
        crate::customer_extract::extract(&state, id(&inquiry)?).await?,
    ))
}

pub async fn revise(
    State(state): State<AppState>,
    Path(inquiry): Path<String>,
    input: Result<Json<PrepareInput>, axum::extract::rejection::JsonRejection>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
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
    let mut filter = state.scope();
    filter.insert(
        "inquiry_id",
        inquiry.get_object_id("_id").map_err(|_| AppError::Data)?,
    );
    let latest = state
        .collection("draft_requests")
        .find_one(filter)
        .sort(mongodb::bson::doc! {"created_at":-1})
        .await?;
    let latest = latest.as_ref().map(job_view).transpose()?;
    let snapshot = drafts::snapshot(&state, &inquiry).await?;
    let summary = drafts::safe_summary(&inquiry, &snapshot)?;
    let attachments = crate::db::attachments::list(
        &state,
        inquiry.get_object_id("_id").map_err(|_| AppError::Data)?,
    )
    .await?;
    Ok(Json(
        json!({"version":drafts::version(&inquiry),"drafts":drafts::safe_drafts(&state,&inquiry).await?,"latest_request":latest,"context_hash":snapshot.hash,"summary":summary,"attachments":attachments}),
    ))
}

pub async fn save_summary(
    State(state): State<AppState>,
    Path(inquiry): Path<String>,
    headers: HeaderMap,
    input: Result<Json<SummaryInput>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<Value>, AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    let actor = super::session::actor(&state, &headers);
    Ok(Json(
        drafts::save_summary(&state, id(&inquiry)?, input, &actor).await?,
    ))
}
