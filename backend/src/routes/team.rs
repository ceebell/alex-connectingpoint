use super::access::ScopedState;
use crate::{
    db::team,
    error::AppError,
    models::team::{EditInput, InviteInput, VersionInput},
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, Query, State, rejection::JsonRejection},
    http::{HeaderMap, HeaderValue, header},
    response::{IntoResponse, Response},
};
use mongodb::bson::{Bson, DateTime, Document, oid::ObjectId};
use serde::Deserialize;
use serde_json::{Value, json};

const INVITE_COOKIE: &str = "alex_team_invite";
fn id(value: &str) -> Result<ObjectId, AppError> {
    ObjectId::parse_str(value).map_err(|_| AppError::BadRequest)
}
fn cookie_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|header| header.to_str().ok())
        .flat_map(|header| header.split(';'))
        .find_map(|part| part.trim().strip_prefix(INVITE_COOKIE)?.strip_prefix('='))
}
fn response(state: &AppState, value: Value, token: Option<&str>) -> Result<Response, AppError> {
    let mut headers = HeaderMap::new();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    if let Some(token) = token {
        let max_age = if token.is_empty() { 0 } else { 86400 };
        let cookie = format!(
            "{INVITE_COOKIE}={token}; Path=/api/admin; HttpOnly; SameSite=Lax; Max-Age={max_age}{}",
            if state.config.admin_cookie_secure {
                "; Secure"
            } else {
                ""
            }
        );
        headers.insert(
            header::SET_COOKIE,
            HeaderValue::from_str(&cookie).map_err(|_| AppError::Data)?,
        );
    }
    Ok((headers, Json(value)).into_response())
}
pub async fn context(state: &AppState, headers: &HeaderMap) -> Result<Option<Document>, AppError> {
    let Some(token) = cookie_token(headers) else {
        return Ok(None);
    };
    team::invitation(state, token).await.map(Some)
}
pub async fn member_view(state: &AppState, record: Document) -> Result<Value, AppError> {
    let profile = team::profile(state, &record).await?;
    let line_name = profile
        .as_ref()
        .and_then(|profile| profile.get_str("line_name").ok());
    let name = record
        .get_str("name")
        .ok()
        .filter(|name| !name.trim().is_empty())
        .or(line_name)
        .unwrap_or("เจ้าของร้าน");
    let mut status = record.get_str("status").map_err(|_| AppError::Data)?;
    if status == "INVITED"
        && record
            .get_datetime("expires_at")
            .is_ok_and(|expiry| *expiry <= DateTime::now())
    {
        status = "EXPIRED";
    }
    Ok(
        json!({"id":record.get_object_id("_id").map_err(|_|AppError::Data)?.to_hex(),"name":name,"role":record.get_str("role").ok(),
        "status":status,"version":record.get_i64("version").map_err(|_|AppError::Data)?,"branch_ids":record.get_array("branch_ids").ok(),"branches":record.get_array("branches").ok(),
        "note":record.get_str("note").unwrap_or(""),"line_user_id":record.get_str("line_user_id").ok(),"line_name":line_name,
        "picture_url":profile.as_ref().and_then(|profile|profile.get_str("picture_url").ok()),
        "last_login_at":profile.as_ref().and_then(|profile|profile.get_datetime("last_login_at").ok()),
        "created_by":record.get_str("created_by").ok(),"created_at":record.get_datetime("created_at").ok(),"updated_at":record.get_datetime("updated_at").ok(),
        "expires_at":record.get_datetime("expires_at").ok(),"accepted_at":record.get_datetime("accepted_at").ok(),"approved_at":record.get_datetime("approved_at").ok()}),
    )
}
fn invitation_view(record: &Document) -> Value {
    let status = match record.get_str("status").unwrap_or("") {
        "INVITED"
            if record
                .get_datetime("expires_at")
                .is_ok_and(|expiry| *expiry > DateTime::now()) =>
        {
            "INVITED"
        }
        "INVITED" => "EXPIRED",
        "CANCELLED" => "CANCELLED",
        _ => "USED",
    };
    json!({"id":record.get_object_id("_id").ok().map(|id|id.to_hex()),"version":record.get_i64("version").ok(),"shop_name":"Arisa Rental","name":record.get_str("name").ok(),"role":record.get_str("role").ok(),
        "branches":record.get_array("branches").ok(),"invited_by":record.get_str("created_by").ok(),"expires_at":record.get_datetime("expires_at").ok(),"status":status})
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpenInput {
    token: String,
}
pub async fn open(
    State(state): State<AppState>,
    headers: HeaderMap,
    input: Result<Json<OpenInput>, JsonRejection>,
) -> Result<Response, AppError> {
    super::session::require_browser_header(&headers)?;
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    let record = team::invitation(&state, &input.token).await?;
    response(
        &state,
        json!({"invitation":invitation_view(&record)}),
        Some(&input.token),
    )
}
pub async fn invitation(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let record = context(&state, &headers).await?;
    response(
        &state,
        json!({"invitation":record.as_ref().map(invitation_view)}),
        None,
    )
}
pub async fn clear(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    super::session::require_browser_header(&headers)?;
    response(&state, json!({"status":"ok"}), Some(""))
}
pub async fn accept(
    State(state): State<AppState>,
    headers: HeaderMap,
    input: Result<Json<AcceptInput>, JsonRejection>,
) -> Result<Response, AppError> {
    super::session::require_browser_header(&headers)?;
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    let identity = super::session::identity(&state, &headers)
        .filter(|identity| identity.is_line())
        .ok_or(AppError::Unauthorized)?;
    let token = cookie_token(&headers).ok_or(AppError::Team("INVITE_INVALID"))?;
    let invitation = team::invitation(&state, token).await?;
    if invitation.get_object_id("_id").ok() != Some(id(&input.invitation_id)?)
        || (invitation.get_str("status").ok() == Some("INVITED")
            && invitation.get_i64("version").ok() != Some(input.expected_version))
    {
        return Err(AppError::RecordConflict);
    }
    let record = team::accept(&state, token, &identity.id, &identity.name).await?;
    response(
        &state,
        json!({"membership":member_view(&state,record).await?}),
        Some(""),
    )
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcceptInput {
    invitation_id: String,
    expected_version: i64,
}
#[derive(Deserialize)]
pub struct Page {
    before: Option<String>,
    before_version: Option<i64>,
}
pub async fn list(
    ScopedState(state): ScopedState,
    Query(page): Query<Page>,
) -> Result<Json<Value>, AppError> {
    let records = team::list(&state, page.before.as_deref().map(id).transpose()?).await?;
    let next = if records.len() == 100 {
        records
            .last()
            .and_then(|record| record.get_object_id("_id").ok())
            .map(|id| id.to_hex())
    } else {
        None
    };
    let mut members = Vec::new();
    for record in records {
        members.push(member_view(&state, record).await?);
    }
    Ok(Json(json!({"members":members,"next":next})))
}
pub async fn branches(ScopedState(state): ScopedState) -> Result<Json<Value>, AppError> {
    team::require_owner(&state)?;
    let branches = if state.config.alex.is_some() {
        crate::alex::login(&state).await?.branches
    } else {
        vec![]
    };
    Ok(Json(json!({"branches":branches})))
}
pub async fn detail(
    ScopedState(state): ScopedState,
    Path(member): Path<String>,
) -> Result<Json<Value>, AppError> {
    team::require_owner(&state)?;
    Ok(Json(
        json!({"member":member_view(&state,team::by_id(&state,id(&member)?).await?).await?}),
    ))
}
pub async fn create(
    ScopedState(state): ScopedState,
    input: Result<Json<InviteInput>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    team::require_owner(&state)?;
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    let record = team::create_invitation(&state, input).await?;
    let token = team::invitation_token(&state, &record)?;
    Ok(Json(
        json!({"member":member_view(&state,record).await?,"token":token}),
    ))
}
pub async fn link(
    ScopedState(state): ScopedState,
    Path(member): Path<String>,
) -> Result<Json<Value>, AppError> {
    team::require_owner(&state)?;
    let record = team::by_id(&state, id(&member)?).await?;
    Ok(Json(
        json!({"token":team::invitation_token(&state,&record)?}),
    ))
}
async fn change(
    state: AppState,
    member: String,
    input: Result<Json<VersionInput>, JsonRejection>,
    action: &str,
) -> Result<Json<Value>, AppError> {
    team::require_owner(&state)?;
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    let record = team::transition(&state, id(&member)?, input.expected_version, action).await?;
    Ok(Json(json!({"member":member_view(&state,record).await?})))
}
pub async fn approve(
    ScopedState(state): ScopedState,
    Path(member): Path<String>,
    input: Result<Json<VersionInput>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    change(state, member, input, "APPROVED").await
}
pub async fn reject(
    ScopedState(state): ScopedState,
    Path(member): Path<String>,
    input: Result<Json<VersionInput>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    change(state, member, input, "REJECTED").await
}
pub async fn cancel(
    ScopedState(state): ScopedState,
    Path(member): Path<String>,
    input: Result<Json<VersionInput>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    change(state, member, input, "CANCELLED").await
}
pub async fn edit(
    ScopedState(state): ScopedState,
    Path(member): Path<String>,
    input: Result<Json<EditInput>, JsonRejection>,
) -> Result<Json<Value>, AppError> {
    team::require_owner(&state)?;
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    let record = team::edit(&state, id(&member)?, input).await?;
    Ok(Json(json!({"member":member_view(&state,record).await?})))
}
pub async fn history(
    ScopedState(state): ScopedState,
    Path(member): Path<String>,
    Query(page): Query<Page>,
) -> Result<Json<Value>, AppError> {
    team::require_owner(&state)?;
    if page.before_version.is_some_and(|version| version < 1) {
        return Err(AppError::BadRequest);
    }
    let record = team::by_id(&state, id(&member)?).await?;
    let events: Vec<&Document> = record
        .get_array("history")
        .map_err(|_| AppError::Data)?
        .iter()
        .rev()
        .filter_map(Bson::as_document)
        .filter(|event| {
            page.before_version.is_none_or(|before| {
                event
                    .get_i64("version")
                    .is_ok_and(|version| version < before)
            })
        })
        .take(50)
        .collect();
    let next = if events.len() == 50 {
        events
            .last()
            .and_then(|event| event.get_i64("version").ok())
    } else {
        None
    };
    Ok(Json(json!({"events":events,"next":next})))
}
