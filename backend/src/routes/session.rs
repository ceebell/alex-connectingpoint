use crate::{error::AppError, state::AppState};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, HeaderValue, header},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

const COOKIE: &str = "alex_admin";
const MAX_AGE: u64 = 8 * 60 * 60;

fn now() -> Result<u64, AppError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|n| n.as_secs())
        .map_err(|_| AppError::Data)
}

fn mac(state: &AppState, value: &[u8]) -> Result<Hmac<Sha256>, AppError> {
    let mut mac = Hmac::<Sha256>::new_from_slice(state.config.admin_token.as_bytes())
        .map_err(|_| AppError::Data)?;
    mac.update(b"alex-admin-session-v1:");
    mac.update(value);
    Ok(mac)
}

pub fn valid_cookie(state: &AppState, headers: &HeaderMap) -> bool {
    let Some(password) = &state.config.admin_password else {
        return false;
    };
    let token = headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| {
            v.split(';')
                .map(str::trim)
                .find_map(|v| v.strip_prefix("alex_admin="))
        });
    let Some((expiry, signature)) = token.and_then(|v| v.split_once('.')) else {
        return false;
    };
    let Ok(expiry_time) = expiry.parse::<u64>() else {
        return false;
    };
    let Ok(current) = now() else {
        return false;
    };
    if expiry_time <= current || expiry_time > current + MAX_AGE {
        return false;
    }
    let Ok(signature) = URL_SAFE_NO_PAD.decode(signature) else {
        return false;
    };
    // Rotating either the login password or internal token invalidates sessions.
    let payload = format!("{}:{expiry}:{password}", state.config.admin_username);
    mac(state, payload.as_bytes()).is_ok_and(|m| m.verify_slice(&signature).is_ok())
}

pub fn require_browser_header(headers: &HeaderMap) -> Result<(), AppError> {
    if headers.get("x-admin-request").and_then(|v| v.to_str().ok()) != Some("1") {
        return Err(AppError::Unauthorized);
    }
    Ok(())
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
    Json(input): Json<Login>,
) -> Result<(HeaderMap, Json<serde_json::Value>), AppError> {
    require_browser_header(&headers)?;
    let expected = state
        .config
        .admin_password
        .as_ref()
        .ok_or(AppError::AdminUnavailable)?;
    if expected.len() < 16 {
        return Err(AppError::AdminUnavailable);
    }
    let mut attempts = state.login_attempts.lock().await;
    if attempts.1.elapsed().as_secs() >= 60 {
        *attempts = (0, std::time::Instant::now());
    }
    if attempts.0 >= 10 {
        return Err(AppError::LoginLimited);
    }
    let actual = format!("{}:{}", input.username, input.password);
    let expected_credentials = format!("{}:{expected}", state.config.admin_username);
    let signature = mac(&state, expected_credentials.as_bytes())?
        .finalize()
        .into_bytes();
    if input.username.len() > 200
        || input.password.len() > 1000
        || mac(&state, actual.as_bytes())?
            .verify_slice(&signature)
            .is_err()
    {
        attempts.0 += 1;
        return Err(AppError::Unauthorized);
    }
    attempts.0 = 0;
    drop(attempts);
    let expiry = (now()? + MAX_AGE).to_string();
    let payload = format!("{}:{expiry}:{expected}", state.config.admin_username);
    let signature =
        URL_SAFE_NO_PAD.encode(mac(&state, payload.as_bytes())?.finalize().into_bytes());
    let cookie = format!(
        "{COOKIE}={expiry}.{signature}; Path=/; HttpOnly; SameSite=Strict; Max-Age={MAX_AGE}{}",
        if state.config.admin_cookie_secure {
            "; Secure"
        } else {
            ""
        }
    );
    Ok((
        cookie_headers(cookie)?,
        Json(serde_json::json!({"username":state.config.admin_username})),
    ))
}

fn cookie_headers(cookie: String) -> Result<HeaderMap, AppError> {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(|_| AppError::Data)?,
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok(headers)
}

pub async fn logout(headers: HeaderMap) -> Result<(HeaderMap, Json<serde_json::Value>), AppError> {
    require_browser_header(&headers)?;
    Ok((
        cookie_headers(format!(
            "{COOKIE}=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0"
        ))?,
        Json(serde_json::json!({"status":"ok"})),
    ))
}

pub async fn info(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(
        serde_json::json!({"username":state.config.admin_username, "hermes_configured":state.config.hermes_url.is_some() && state.config.hermes_api_key.is_some()}),
    )
}
