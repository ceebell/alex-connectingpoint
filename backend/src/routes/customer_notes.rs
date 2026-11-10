use crate::{
    db::{customer_notes, customers, users},
    error::AppError,
    models::{
        customer::valid_user_id,
        customer_notes::{AliasInput, CreateInput, EditInput},
    },
};
use axum::{
    Json,
    extract::{Path, Query, rejection::JsonRejection},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use mongodb::bson::oid::ObjectId;
use serde::Deserialize;
use serde_json::{Value, json};

fn user(value: &str) -> Result<&str, AppError> {
    valid_user_id(value)
        .then_some(value)
        .ok_or(AppError::BadRequest)
}
fn id(value: &str) -> Result<ObjectId, AppError> {
    ObjectId::parse_str(value).map_err(|_| AppError::BadRequest)
}

pub async fn alias(
    super::access::ScopedState(state): super::access::ScopedState,
    Path(user_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    let user = user(&user_id)?;
    customers::ensure_user(&state, user).await?;
    let profiles = users::profiles(&state, vec![user.to_string()]).await?;
    Ok(Json(
        json!({"alias":customers::alias(&state,user).await?,"line_display_name":profiles.get(user).and_then(|profile|profile.get_str("display_name").ok())}),
    ))
}

pub async fn save_alias(
    super::access::ScopedState(state): super::access::ScopedState,
    Path(user_id): Path<String>,
    headers: HeaderMap,
    input: Result<Json<AliasInput>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    let actor = super::session::actor(&state, &headers);
    Ok(Json(
        json!({"alias":customers::save_alias(&state,user(&user_id)?,input.validate()?,&actor).await?}),
    ))
}

#[derive(Deserialize)]
pub struct Page {
    before: Option<String>,
}

pub async fn list(
    super::access::ScopedState(state): super::access::ScopedState,
    Path(user_id): Path<String>,
    Query(page): Query<Page>,
) -> Result<Json<Value>, AppError> {
    let before = page.before.as_deref().map(id).transpose()?;
    let (notes, next) = customer_notes::list(&state, user(&user_id)?, before).await?;
    Ok(Json(json!({"notes":notes,"next":next})))
}

pub async fn detail(
    super::access::ScopedState(state): super::access::ScopedState,
    Path((user_id, note)): Path<(String, String)>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(
        json!({"note":customer_notes::get(&state,user(&user_id)?,id(&note)?).await?}),
    ))
}

pub async fn create(
    super::access::ScopedState(state): super::access::ScopedState,
    Path(user_id): Path<String>,
    headers: HeaderMap,
    input: Result<Json<CreateInput>, JsonRejection>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    let content = input.validate()?;
    let actor = super::session::actor(&state, &headers);
    let note = customer_notes::create(&state, user(&user_id)?, &input, content, &actor).await?;
    Ok((StatusCode::CREATED, Json(json!({"note":note}))))
}

pub async fn edit(
    super::access::ScopedState(state): super::access::ScopedState,
    Path((user_id, note)): Path<(String, String)>,
    headers: HeaderMap,
    input: Result<Json<EditInput>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    let content = input.validate()?;
    let actor = super::session::actor(&state, &headers);
    Ok(Json(
        json!({"note":customer_notes::edit(&state,user(&user_id)?,id(&note)?,input.expected_version,content,&actor).await?}),
    ))
}

pub async fn image(
    super::access::ScopedState(state): super::access::ScopedState,
    Path((user_id, image)): Path<(String, String)>,
) -> Result<Response, AppError> {
    let (mime, bytes) = customer_notes::read_image(&state, user(&user_id)?, id(&image)?).await?;
    let mut response = bytes.into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&mime).map_err(|_| AppError::Data)?,
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    Ok(response)
}
