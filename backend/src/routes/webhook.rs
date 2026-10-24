use crate::{
    db::{events, media},
    error::AppError,
    models::line::Webhook,
    state::AppState,
};
use axum::{
    Json,
    body::Bytes,
    extract::{State, rejection::BytesRejection},
    http::HeaderMap,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use hmac::{Hmac, Mac};
use sha2::Sha256;

pub fn verify(secret: &str, signature: &str, body: &[u8]) -> Result<(), AppError> {
    let signature = STANDARD
        .decode(signature)
        .map_err(|_| AppError::Unauthorized)?;
    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret.as_bytes()).map_err(|_| AppError::Unauthorized)?;
    mac.update(body);
    mac.verify_slice(&signature)
        .map_err(|_| AppError::Unauthorized)
}

pub async fn receive(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<Bytes, BytesRejection>,
) -> Result<Json<serde_json::Value>, AppError> {
    let body = body.map_err(|_| AppError::BadRequest)?;
    let signature = headers
        .get("x-line-signature")
        .and_then(|s| s.to_str().ok())
        .ok_or(AppError::Unauthorized)?;
    verify(&state.config.channel_secret, signature, &body)?;
    let mut webhook: Webhook = serde_json::from_slice(&body).map_err(|_| AppError::BadRequest)?;
    if webhook.destination != state.config.destination {
        return Err(AppError::Unauthorized);
    }
    if !webhook.events.iter().all(|e| e.validate()) {
        return Err(AppError::BadRequest);
    }
    let mut accepted = 0;
    for event in &mut webhook.events {
        if let Some(message) = event.message.as_mut().and_then(|m| m.as_object_mut()) {
            message.remove("quoteToken");
        }
        // Persist an unsend tombstone before acknowledging, including out-of-order deliveries.
        if let Some(id) = event.unsend.as_ref().and_then(|u| u["messageId"].as_str()) {
            media::tombstone(&state, id).await?;
        }
        accepted += usize::from(events::persist(&state, event).await?);
    }
    tracing::info!(
        accepted,
        total = webhook.events.len(),
        "LINE webhook durably received"
    );
    Ok(Json(
        serde_json::json!({"status":"ok", "accepted":accepted}),
    ))
}
