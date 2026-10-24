use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Config(String),
    #[error("Invalid request")]
    BadRequest,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Not found")]
    NotFound,
    #[error("Method not allowed")]
    MethodNotAllowed,
    #[error("Inquiry selection required")]
    Conflict,
    #[error("Database unavailable")]
    Database(#[from] mongodb::error::Error),
    #[error("Media download failed")]
    Http(#[from] reqwest::Error),
    #[error("Media storage failed")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Media(&'static str),
    #[error("Invalid stored data")]
    Data,
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::BadRequest => "BAD_REQUEST",
            Self::Unauthorized => "UNAUTHORIZED",
            Self::NotFound => "NOT_FOUND",
            Self::MethodNotAllowed => "METHOD_NOT_ALLOWED",
            Self::Conflict => "INQUIRY_SELECTION_REQUIRED",
            Self::Database(_) => "DATABASE_UNAVAILABLE",
            Self::Http(_) | Self::Io(_) | Self::Media(_) => "MEDIA_UNAVAILABLE",
            Self::Config(_) | Self::Data => "INTERNAL_ERROR",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::BadRequest => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
            Self::Conflict => StatusCode::CONFLICT,
            Self::Database(_) => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        // Never log driver/HTTP errors: they may include credentials or customer data.
        if status.is_server_error() {
            tracing::error!(code = self.code(), "request failed");
        }
        (
            status,
            Json(json!({"error": {"code": self.code(), "message": self.to_string()}})),
        )
            .into_response()
    }
}
