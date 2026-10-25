mod admin;
mod customers;
mod drafts;
mod oa_sync;
mod session;
mod users;
pub(crate) mod webhook;

use crate::{error::AppError, state::AppState};
use axum::{
    Json, Router,
    extract::DefaultBodyLimit,
    http::{HeaderValue, Method},
    routing::{get, post},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

pub fn router(state: AppState) -> Result<Router, AppError> {
    let admin = Router::new()
        .route(
            "/api/admin/session",
            get(session::info).delete(session::logout),
        )
        .route("/api/inquiries/{id}/drafts", get(drafts::list))
        .route("/api/inquiries/{id}/prepare", post(drafts::prepare))
        .route("/api/inquiries/{id}/draft", post(drafts::revise))
        .route("/api/draft-requests/{id}", get(drafts::job))
        .route("/api/inquiries", get(admin::list))
        .route("/api/inquiries/{id}/context", get(admin::context))
        .route("/api/conversations/{id}/inquiries", post(admin::create))
        .route("/api/line/events", get(admin::events))
        .route("/api/line/users", get(users::list))
        .route("/api/line/users/{user_id}/messages", get(users::messages))
        .route(
            "/api/line/users/{user_id}/profile",
            get(customers::profile).post(customers::save_profile),
        )
        .route(
            "/api/inquiries/{id}/order",
            get(customers::order).post(customers::approve),
        )
        .route(
            "/api/inquiries/{id}/messages",
            post(customers::send_message),
        )
        .route("/api/line/events/{id}/assign", post(admin::assign))
        .route("/api/line/events/{id}/retry", post(admin::retry))
        .route("/api/line/status", get(admin::status))
        .route("/api/line/oa-sync/status", get(oa_sync::status))
        .route("/api/media/{id}", get(admin::media))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            admin::authorize,
        ));
    let mut router = Router::new()
        .route("/api/admin/login", post(session::login))
        .route("/api/admin/login-options", get(session::options))
        .route("/api/admin/line/start", get(session::line_start))
        .route("/api/admin/line/callback", get(session::line_callback))
        .route(
            "/api/health",
            get(|| async { Json(serde_json::json!({"status":"ok"})) }),
        )
        .route("/api/line/webhook", post(webhook::receive))
        .route("/api/connectors/line-oa/messages", post(oa_sync::ingest))
        .route("/api/connectors/line-oa/status", post(oa_sync::heartbeat))
        .merge(admin)
        .fallback(|| async { AppError::NotFound })
        .method_not_allowed_fallback(|| async { AppError::MethodNotAllowed })
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024))
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());
    if let Some(origin) = &state.config.cors_origin {
        let origin: HeaderValue = origin
            .parse()
            .map_err(|_| AppError::Config("Invalid CORS_ORIGIN".into()))?;
        router = router.layer(
            CorsLayer::new()
                .allow_origin(origin)
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([
                    axum::http::header::CONTENT_TYPE,
                    axum::http::header::AUTHORIZATION,
                ]),
        );
    }
    Ok(router)
}
