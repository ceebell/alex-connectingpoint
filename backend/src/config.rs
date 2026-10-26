use std::{env, time::Duration};

use crate::error::AppError;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub mongodb_uri: String,
    pub mongodb_db: String,
    pub cors_origin: Option<String>,
    pub shop_id: String,
    pub channel_id: String,
    pub destination: String,
    pub channel_secret: String,
    pub channel_access_token: String,
    pub admin_token: String,
    pub burst_ms: i64,
    pub media_max_bytes: usize,
    pub media_retention_days: i64,
    pub admin_username: String,
    pub admin_password: Option<String>,
    pub admin_cookie_secure: bool,
    pub hermes_url: Option<String>,
    pub hermes_api_key: Option<String>,
    pub hermes_model: String,
    pub hermes_timeout_seconds: u64,
    pub line_api_base: String,
    pub line_login_channel_id: Option<String>,
    pub line_login_channel_secret: Option<String>,
    pub line_login_redirect_uri: Option<String>,
    pub admin_line_user_ids: Vec<String>,
    pub oa_connector_enabled: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        Ok(Self {
            port: number("PORT", 8080, 1, 65535)? as u16,
            mongodb_uri: env::var("MONGODB_URI")
                .unwrap_or_else(|_| "mongodb://localhost:27017".into()),
            mongodb_db: env::var("MONGODB_DB").unwrap_or_else(|_| "myapp".into()),
            cors_origin: env::var("CORS_ORIGIN").ok().filter(|s| !s.is_empty()),
            shop_id: required("LINE_SHOP_ID")?,
            channel_id: required("LINE_CHANNEL_ID")?,
            destination: required("LINE_DESTINATION_ID")?,
            channel_secret: required("LINE_CHANNEL_SECRET")?,
            channel_access_token: required("LINE_CHANNEL_ACCESS_TOKEN")?,
            admin_token: required("INQUIRY_ADMIN_TOKEN")?,
            burst_ms: number("LINE_BURST_WINDOW_MS", 3000, 100, 60000)?,
            media_max_bytes: number("LINE_MEDIA_MAX_BYTES", 20971520, 1024, 52428800)? as usize,
            media_retention_days: number("LINE_MEDIA_RETENTION_DAYS", 30, 1, 3650)?,
            admin_username: env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".into()),
            admin_password: optional("ADMIN_PASSWORD"),
            admin_cookie_secure: env::var("ADMIN_COOKIE_SECURE").is_ok_and(|v| v == "true"),
            hermes_url: optional("HERMES_API_URL"),
            hermes_api_key: optional("HERMES_API_KEY"),
            hermes_model: env::var("HERMES_MODEL").unwrap_or_else(|_| "hermes-agent".into()),
            hermes_timeout_seconds: number("HERMES_TIMEOUT_SECONDS", 120, 5, 180)? as u64,
            line_api_base: env::var("LINE_API_BASE")
                .ok()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "https://api.line.me".into())
                .trim_end_matches('/')
                .to_string(),
            line_login_channel_id: optional("LINE_LOGIN_CHANNEL_ID"),
            line_login_channel_secret: optional("LINE_LOGIN_CHANNEL_SECRET"),
            line_login_redirect_uri: optional("LINE_LOGIN_REDIRECT_URI"),
            admin_line_user_ids: env::var("ADMIN_LINE_USER_IDS")
                .unwrap_or_default()
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect(),
            oa_connector_enabled: env::var("OA_CONNECTOR_ENABLED").is_ok_and(|v| v == "true"),
        })
    }

    pub fn line_login_enabled(&self) -> bool {
        self.line_login_channel_id.is_some()
            && self.line_login_channel_secret.is_some()
            && self.line_login_redirect_uri.is_some()
    }

    pub fn password_login_enabled(&self) -> bool {
        self.admin_password.as_ref().is_some_and(|p| p.len() >= 16)
    }

    pub fn http_timeout(&self) -> Duration {
        Duration::from_secs(30)
    }
}

fn optional(name: &str) -> Option<String> {
    env::var(name).ok().filter(|v| !v.trim().is_empty())
}

fn required(name: &str) -> Result<String, AppError> {
    env::var(name)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| AppError::Config(format!("{name} must be set")))
}

fn number(name: &str, default: i64, min: i64, max: i64) -> Result<i64, AppError> {
    let value = match env::var(name) {
        Ok(value) => value
            .parse()
            .map_err(|_| AppError::Config(format!("{name} must be an integer")))?,
        Err(_) => default,
    };
    if !(min..=max).contains(&value) {
        return Err(AppError::Config(format!(
            "{name} must be between {min} and {max}"
        )));
    }
    Ok(value)
}
