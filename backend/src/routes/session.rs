use crate::{
    error::AppError,
    models::customer::{Actor, valid_user_id},
    state::AppState,
};
use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, HeaderValue, header},
    response::{IntoResponse, Redirect, Response},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

const COOKIE: &str = "alex_admin";
const STATE_COOKIE: &str = "alex_line_state";
const STATE_PATH: &str = "/api/admin/line";
const MAX_AGE: u64 = 8 * 60 * 60;
const STATE_AGE: u64 = 10 * 60;
const AUTHORIZE_URL: &str = "https://access.line.me/oauth2/v2.1/authorize";

fn now() -> Result<u64, AppError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|n| n.as_secs())
        .map_err(|_| AppError::Data)
}

// Domain-separated HMAC keyed by the server-only internal token.
fn mac(state: &AppState, purpose: &str, value: &str) -> Result<Hmac<Sha256>, AppError> {
    let mut mac = Hmac::<Sha256>::new_from_slice(state.config.admin_token.as_bytes())
        .map_err(|_| AppError::Data)?;
    mac.update(b"alex-admin-v2:");
    mac.update(purpose.as_bytes());
    mac.update(b":");
    mac.update(value.as_bytes());
    Ok(mac)
}

fn sign(state: &AppState, purpose: &str, value: &str) -> Result<String, AppError> {
    Ok(URL_SAFE_NO_PAD.encode(mac(state, purpose, value)?.finalize().into_bytes()))
}

fn verify(state: &AppState, purpose: &str, value: &str, signature: &str) -> bool {
    let Ok(signature) = URL_SAFE_NO_PAD.decode(signature) else {
        return false;
    };
    mac(state, purpose, value).is_ok_and(|m| m.verify_slice(&signature).is_ok())
}

fn cookie_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .find_map(|v| v.trim().strip_prefix(name)?.strip_prefix('='))
}

fn fresh(expiry: &str, max_age: u64) -> bool {
    let (Ok(expiry), Ok(current)) = (expiry.parse::<u64>(), now()) else {
        return false;
    };
    expiry > current && expiry <= current + max_age
}

#[derive(Clone, Serialize, Deserialize)]
struct Admin {
    kind: String,
    id: String,
    name: String,
}

// Secret material a session is bound to. Rotating the password, removing a LINE
// user from ADMIN_LINE_USER_IDS or disabling a login method revokes its sessions.
fn binding(state: &AppState, admin: &Admin) -> Option<String> {
    let config = &state.config;
    match admin.kind.as_str() {
        "password" if config.password_login_enabled() && admin.id == config.admin_username => {
            config
                .admin_password
                .clone()
                .map(|p| format!("password:{p}"))
        }
        "line"
            if config.line_login_enabled()
                && config.admin_line_user_ids.iter().any(|id| id == &admin.id) =>
        {
            config
                .line_login_channel_id
                .as_ref()
                .map(|c| format!("line:{c}"))
        }
        _ => None,
    }
}

fn session_token(state: &AppState, admin: &Admin) -> Result<String, AppError> {
    let expiry = now()? + MAX_AGE;
    let body = URL_SAFE_NO_PAD.encode(serde_json::to_vec(admin).map_err(|_| AppError::Data)?);
    let bound = binding(state, admin).ok_or(AppError::Unauthorized)?;
    let signature = sign(state, "session", &format!("{expiry}.{body}.{bound}"))?;
    Ok(format!("{expiry}.{body}.{signature}"))
}

fn session(state: &AppState, headers: &HeaderMap) -> Option<Admin> {
    let mut parts = cookie_value(headers, COOKIE)?.splitn(3, '.');
    let (expiry, body, signature) = (parts.next()?, parts.next()?, parts.next()?);
    if !fresh(expiry, MAX_AGE) {
        return None;
    }
    let admin: Admin = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(body).ok()?).ok()?;
    let bound = binding(state, &admin)?;
    verify(
        state,
        "session",
        &format!("{expiry}.{body}.{bound}"),
        signature,
    )
    .then_some(admin)
}

pub fn valid_cookie(state: &AppState, headers: &HeaderMap) -> bool {
    session(state, headers).is_some()
}

// Recorded on admin writes; the internal Bearer token has no person behind it.
pub fn actor(state: &AppState, headers: &HeaderMap) -> Actor {
    match session(state, headers) {
        Some(admin) => Actor {
            id: format!("{}:{}", admin.kind, admin.id),
            name: admin.name,
        },
        None => Actor {
            id: "internal-api".into(),
            name: "internal-api".into(),
        },
    }
}

pub fn require_browser_header(headers: &HeaderMap) -> Result<(), AppError> {
    if headers.get("x-admin-request").and_then(|v| v.to_str().ok()) != Some("1") {
        return Err(AppError::Unauthorized);
    }
    Ok(())
}

fn cookie(
    state: &AppState,
    name: &str,
    value: &str,
    path: &str,
    max_age: u64,
    lax: bool,
) -> String {
    format!(
        "{name}={value}; Path={path}; HttpOnly; SameSite={}; Max-Age={max_age}{}",
        if lax { "Lax" } else { "Strict" },
        if state.config.admin_cookie_secure {
            "; Secure"
        } else {
            ""
        }
    )
}

fn cookie_headers(cookies: &[String]) -> Result<HeaderMap, AppError> {
    let mut headers = HeaderMap::new();
    for cookie in cookies {
        headers.append(
            header::SET_COOKIE,
            HeaderValue::from_str(cookie).map_err(|_| AppError::Data)?,
        );
    }
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok(headers)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Login {
    username: String,
    password: String,
}

pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    input: Result<Json<Login>, axum::extract::rejection::JsonRejection>,
) -> Result<(HeaderMap, Json<serde_json::Value>), AppError> {
    require_browser_header(&headers)?;
    let Json(input) = input.map_err(|_| AppError::BadRequest)?;
    if !state.config.password_login_enabled() {
        return Err(AppError::AdminUnavailable);
    }
    let expected = state
        .config
        .admin_password
        .as_ref()
        .ok_or(AppError::AdminUnavailable)?;
    let mut attempts = state.login_attempts.lock().await;
    if attempts.1.elapsed().as_secs() >= 60 {
        *attempts = (0, std::time::Instant::now());
    }
    if attempts.0 >= 10 {
        return Err(AppError::LoginLimited);
    }
    let actual = format!("{}:{}", input.username, input.password);
    let expected_credentials = format!("{}:{expected}", state.config.admin_username);
    let signature = sign(&state, "login", &expected_credentials)?;
    if input.username.len() > 200
        || input.password.len() > 1000
        || !verify(&state, "login", &actual, &signature)
    {
        attempts.0 += 1;
        return Err(AppError::Unauthorized);
    }
    attempts.0 = 0;
    drop(attempts);
    let admin = Admin {
        kind: "password".into(),
        id: state.config.admin_username.clone(),
        name: state.config.admin_username.clone(),
    };
    let token = session_token(&state, &admin)?;
    Ok((
        cookie_headers(&[cookie(&state, COOKIE, &token, "/", MAX_AGE, false)])?,
        Json(serde_json::json!({"username":admin.name})),
    ))
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<serde_json::Value>), AppError> {
    require_browser_header(&headers)?;
    Ok((
        cookie_headers(&[cookie(&state, COOKIE, "", "/", 0, false)])?,
        Json(serde_json::json!({"status":"ok"})),
    ))
}

pub async fn info(State(state): State<AppState>, headers: HeaderMap) -> Json<serde_json::Value> {
    let admin = session(&state, &headers);
    Json(serde_json::json!({
        "username":admin.as_ref().map_or("internal-api", |a| a.name.as_str()),
        "login":admin.as_ref().map_or("token", |a| a.kind.as_str()),
        "hermes_configured":state.config.hermes_url.is_some() && state.config.hermes_api_key.is_some()
    }))
}

pub async fn options(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "line":state.config.line_login_enabled(),
        "password":state.config.password_login_enabled()
    }))
}

fn login_redirect(state: &AppState, query: &str) -> Response {
    let clear = cookie(state, STATE_COOKIE, "", STATE_PATH, 0, true);
    match cookie_headers(&[clear]) {
        Ok(headers) => (headers, Redirect::to(&format!("/?{query}"))).into_response(),
        Err(error) => error.into_response(),
    }
}

pub async fn line_start(State(state): State<AppState>) -> Response {
    let config = &state.config;
    let (Some(client_id), Some(redirect_uri)) = (
        &config.line_login_channel_id,
        &config.line_login_redirect_uri,
    ) else {
        return login_redirect(&state, "login_error=LINE_NOT_CONFIGURED");
    };
    let result = (|| -> Result<Response, AppError> {
        // ObjectId is unique per call; the keyed HMAC makes it unguessable.
        let seed = format!("{}:{:?}", ObjectId::new().to_hex(), SystemTime::now());
        let random = sign(&state, "state-seed", &seed)?;
        let expiry = now()? + STATE_AGE;
        let value = format!("{expiry}.{random}");
        let token = format!("{value}.{}", sign(&state, "line-state", &value)?);
        let nonce = sign(&state, "line-nonce", &random)?;
        let url = reqwest::Url::parse_with_params(
            AUTHORIZE_URL,
            [
                ("response_type", "code"),
                ("client_id", client_id),
                ("redirect_uri", redirect_uri),
                ("state", &token),
                ("scope", "openid profile"),
                ("nonce", &nonce),
            ],
        )
        .map_err(|_| AppError::Config("Invalid LINE_LOGIN_REDIRECT_URI".into()))?;
        // Lax: the cookie must come back on LINE's top-level redirect to the callback.
        let headers = cookie_headers(&[cookie(
            &state,
            STATE_COOKIE,
            &token,
            STATE_PATH,
            STATE_AGE,
            true,
        )])?;
        Ok((headers, Redirect::to(url.as_str())).into_response())
    })();
    result.unwrap_or_else(IntoResponse::into_response)
}

#[derive(Deserialize)]
pub struct Callback {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct TokenResponse {
    id_token: String,
}

#[derive(Deserialize)]
struct IdToken {
    sub: String,
    aud: String,
    name: Option<String>,
}

async fn line_identity(state: &AppState, code: &str, nonce: &str) -> Result<IdToken, AppError> {
    let config = &state.config;
    let (Some(client_id), Some(secret), Some(redirect_uri)) = (
        &config.line_login_channel_id,
        &config.line_login_channel_secret,
        &config.line_login_redirect_uri,
    ) else {
        return Err(AppError::AdminUnavailable);
    };
    let token: TokenResponse = state
        .http
        .post(format!("{}/oauth2/v2.1/token", config.line_api_base))
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("client_id", client_id),
            ("client_secret", secret),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    // LINE checks the signature, expiry, audience and nonce of the ID token.
    let identity: IdToken = state
        .http
        .post(format!("{}/oauth2/v2.1/verify", config.line_api_base))
        .form(&[
            ("id_token", token.id_token.as_str()),
            ("client_id", client_id),
            ("nonce", nonce),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    if &identity.aud != client_id || !valid_user_id(&identity.sub) {
        return Err(AppError::Unauthorized);
    }
    Ok(identity)
}

pub async fn line_callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<Callback>,
) -> Response {
    if query.error.is_some() {
        return login_redirect(&state, "login_error=LINE_CANCELLED");
    }
    let (Some(code), Some(returned)) = (query.code.as_deref(), query.state.as_deref()) else {
        return login_redirect(&state, "login_error=LINE_STATE_INVALID");
    };
    let mut parts = returned.splitn(3, '.');
    let valid_state = cookie_value(&headers, STATE_COOKIE) == Some(returned)
        && match (parts.next(), parts.next(), parts.next()) {
            (Some(expiry), Some(random), Some(signature)) => {
                fresh(expiry, STATE_AGE)
                    && verify(
                        &state,
                        "line-state",
                        &format!("{expiry}.{random}"),
                        signature,
                    )
            }
            _ => false,
        };
    let Some(random) = returned.split('.').nth(1).filter(|_| valid_state) else {
        return login_redirect(&state, "login_error=LINE_STATE_INVALID");
    };
    let identity = match sign(&state, "line-nonce", random) {
        Ok(nonce) => line_identity(&state, code, &nonce).await,
        Err(error) => Err(error),
    };
    let Ok(identity) = identity else {
        return login_redirect(&state, "login_error=LINE_LOGIN_FAILED");
    };
    if !state
        .config
        .admin_line_user_ids
        .iter()
        .any(|id| id == &identity.sub)
    {
        // The owner needs this ID to grant access; it is not a secret.
        return login_redirect(
            &state,
            &format!("login_error=LINE_NOT_ALLOWED&line_user_id={}", identity.sub),
        );
    }
    let name: String = identity
        .name
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| identity.sub.clone())
        .chars()
        .take(100)
        .collect();
    let admin = Admin {
        kind: "line".into(),
        id: identity.sub,
        name,
    };
    let result = session_token(&state, &admin).and_then(|token| {
        cookie_headers(&[
            cookie(&state, COOKIE, &token, "/", MAX_AGE, false),
            cookie(&state, STATE_COOKIE, "", STATE_PATH, 0, true),
        ])
    });
    match result {
        Ok(headers) => (headers, Redirect::to("/")).into_response(),
        Err(_) => login_redirect(&state, "login_error=LINE_LOGIN_FAILED"),
    }
}
