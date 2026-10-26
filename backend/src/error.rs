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
    #[error("OA messages cannot be linked to one verified inquiry")]
    OaUnlinked,
    #[error("OA message ID has conflicting content")]
    OaMessageConflict,
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
    #[error("{0}")]
    Validation(&'static str),
    #[error("Draft changed or another preparation is in progress")]
    DraftConflict,
    #[error("Admin login is not configured")]
    AdminUnavailable,
    #[error("Too many login attempts; try again in one minute")]
    LoginLimited,
    #[error("Hermes is not configured")]
    HermesUnavailable,
    #[error("Hermes did not return a valid completed draft")]
    HermesFailed,
    #[error("Hermes extraction profile must have all tools disabled")]
    HermesToolsEnabled,
    #[error("New or withdrawn messages require preparing a fresh draft")]
    ContextChanged,
    #[error("Waiting for incoming messages to finish processing")]
    DraftWaiting,
    #[error("Record changed; reload and try again")]
    RecordConflict,
    #[error("Admin summary changed; reload it before saving")]
    SummaryConflict,
    #[error("This inquiry already has an approved order")]
    OrderExists,
    #[error("Only the latest complete, conflict-free and current draft can be approved")]
    OrderNotReady,
    #[error("Replies can only be sent to a known 1:1 LINE chat")]
    ReplyUnavailable,
    #[error("LINE did not accept the message")]
    LineSend(&'static str),
    #[error("Sending images needs a public HTTPS address (PUBLIC_BASE_URL)")]
    ImageUnavailable,
    #[error("Alex catalog is not configured")]
    AlexUnavailable,
    #[error("Alex did not return a valid catalog response")]
    AlexFailed,
    #[error("The Alex account or branch is outside the configured shop scope")]
    AlexScope,
    #[error("Product selection changed or expired; search again")]
    ProductConflict,
    #[error(
        "No readable code or usable catalog images; search by text or add catalog images in Alex"
    )]
    CatalogImagesUnavailable,
    #[error("The bill needs current products, complete dates and an available queue")]
    BillNotReady,
    #[error("An Alex order exists or a submission requires manual verification")]
    BillPending,
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::BadRequest => "BAD_REQUEST",
            Self::Unauthorized => "UNAUTHORIZED",
            Self::NotFound => "NOT_FOUND",
            Self::MethodNotAllowed => "METHOD_NOT_ALLOWED",
            Self::Conflict => "INQUIRY_SELECTION_REQUIRED",
            Self::OaUnlinked => "OA_UNLINKED",
            Self::OaMessageConflict => "OA_MESSAGE_CONFLICT",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::DraftConflict => "DRAFT_CONFLICT",
            Self::AdminUnavailable => "ADMIN_NOT_CONFIGURED",
            Self::LoginLimited => "LOGIN_RATE_LIMITED",
            Self::HermesUnavailable => "HERMES_NOT_CONFIGURED",
            Self::HermesFailed => "HERMES_FAILED",
            Self::HermesToolsEnabled => "HERMES_TOOLS_ENABLED",
            Self::ContextChanged => "CONTEXT_CHANGED",
            Self::DraftWaiting => "DRAFT_WAITING",
            Self::RecordConflict => "RECORD_CONFLICT",
            Self::SummaryConflict => "SUMMARY_CONFLICT",
            Self::OrderExists => "ORDER_EXISTS",
            Self::OrderNotReady => "ORDER_NOT_READY",
            Self::ReplyUnavailable => "REPLY_UNAVAILABLE",
            Self::LineSend(code) => code,
            Self::ImageUnavailable => "IMAGE_UNAVAILABLE",
            Self::AlexUnavailable => "ALEX_NOT_CONFIGURED",
            Self::AlexFailed => "ALEX_FAILED",
            Self::AlexScope => "ALEX_SCOPE_DENIED",
            Self::ProductConflict => "PRODUCT_CONFLICT",
            Self::CatalogImagesUnavailable => "CATALOG_IMAGES_UNAVAILABLE",
            Self::BillNotReady => "BILL_NOT_READY",
            Self::BillPending => "BILL_PENDING",
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
            Self::OaUnlinked | Self::OaMessageConflict => StatusCode::CONFLICT,
            Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::DraftConflict | Self::ContextChanged | Self::DraftWaiting => StatusCode::CONFLICT,
            Self::RecordConflict
            | Self::SummaryConflict
            | Self::OrderExists
            | Self::OrderNotReady => StatusCode::CONFLICT,
            Self::ReplyUnavailable => StatusCode::CONFLICT,
            Self::LineSend(_) => StatusCode::BAD_GATEWAY,
            Self::ImageUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::AlexUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::AlexFailed => StatusCode::BAD_GATEWAY,
            Self::AlexScope => StatusCode::FORBIDDEN,
            Self::ProductConflict => StatusCode::CONFLICT,
            Self::CatalogImagesUnavailable => StatusCode::UNPROCESSABLE_ENTITY,
            Self::BillNotReady | Self::BillPending => StatusCode::CONFLICT,
            Self::LoginLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::AdminUnavailable | Self::HermesUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::HermesFailed | Self::HermesToolsEnabled => StatusCode::BAD_GATEWAY,
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
