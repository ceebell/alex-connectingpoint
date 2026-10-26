mod admin;
mod checkout;
mod customers;
mod drafts;
mod mcp;
mod oa_sync;
mod products;
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
        .route("/api/inquiries/{id}/checkout", get(checkout::list))
        .route(
            "/api/inquiries/{id}/checkout/availability",
            post(checkout::availability),
        )
        .route(
            "/api/inquiries/{id}/checkout/review",
            post(checkout::review),
        )
        .route(
            "/api/inquiries/{id}/checkout/submit",
            post(checkout::submit),
        )
        .route(
            "/api/inquiries/{id}/checkout/resolve",
            post(checkout::resolve),
        )
        .route(
            "/api/inquiries/{id}/draft-images",
            post(drafts::upload_image).layer(DefaultBodyLimit::max(12 * 1024 * 1024)),
        )
        .route(
            "/api/inquiries/{id}/customer-extract",
            post(drafts::extract_customer),
        )
        .route("/api/admin/alex", get(products::settings))
        .route("/api/inquiries/{id}/products", get(products::list))
        .route(
            "/api/inquiries/{id}/products/search",
            post(products::search),
        )
        .route(
            "/api/inquiries/{id}/products/confirm",
            post(products::confirm),
        )
        .route(
            "/api/inquiries/{id}/product-searches/{search}/images/{product}",
            get(products::image),
        )
        .route("/api/inquiries/{id}/summary", post(drafts::save_summary))
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
        .route(
            "/api/inquiries/{id}/media",
            // Base64 of a 10 MB original plus a 1 MB preview.
            post(customers::upload_media).layer(DefaultBodyLimit::max(16 * 1024 * 1024)),
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
        .route("/api/admin/alex/mcp", post(mcp::handle).get(mcp::get))
        .route("/api/admin/login", post(session::login))
        .route("/api/admin/login-options", get(session::options))
        .route("/api/admin/line/start", get(session::line_start))
        .route("/api/admin/line/callback", get(session::line_callback))
        .route(
            "/api/health",
            get(|| async { Json(serde_json::json!({"status":"ok"})) }),
        )
        .route("/api/line/webhook", post(webhook::receive))
        .route(
            "/api/line/outbound-media/{token}/{variant}",
            get(customers::outbound_media),
        )
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
