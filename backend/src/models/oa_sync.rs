use crate::error::AppError;
use mongodb::bson::DateTime;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OaMessage {
    pub id: String,
    pub text: String,
    pub sent_at_ms: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OaBatch {
    pub bot_id: String,
    pub chat_id: String,
    pub anchor_message_ids: Vec<String>,
    pub messages: Vec<OaMessage>,
}

pub fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.:-".contains(&b))
}

impl OaBatch {
    pub fn validate(&self) -> Result<(), AppError> {
        if !valid_id(&self.bot_id)
            || !valid_id(&self.chat_id)
            || self.anchor_message_ids.is_empty()
            || self.anchor_message_ids.len() > 20
            || self.anchor_message_ids.iter().any(|id| !valid_id(id))
            || self.messages.is_empty()
            || self.messages.len() > 50
            || self.messages.iter().any(|m| {
                !valid_id(&m.id)
                    || m.text.trim().is_empty()
                    || m.text.chars().count() > 5000
                    || m.sent_at_ms < 0
                    || m.sent_at_ms > DateTime::now().timestamp_millis() + 300_000
            })
        {
            return Err(AppError::Validation("Invalid OA connector message batch"));
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OaHeartbeat {
    pub state: String,
    pub synced: u64,
    pub pending: u64,
}

impl OaHeartbeat {
    pub fn validate(&self) -> Result<(), AppError> {
        if ![
            "NEEDS_LOGIN",
            "READY",
            "SYNCING",
            "UNSUPPORTED_SCHEMA",
            "UNLINKED",
            "LINE_UNAVAILABLE",
            "BACKEND_UNAVAILABLE",
            "STOPPED",
        ]
        .contains(&self.state.as_str())
            || self.synced > i64::MAX as u64
            || self.pending > 1_000_000
        {
            return Err(AppError::BadRequest);
        }
        Ok(())
    }
}
