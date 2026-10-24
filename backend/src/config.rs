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
        })
    }

    pub fn http_timeout(&self) -> Duration {
        Duration::from_secs(30)
    }
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
