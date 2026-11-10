use crate::error::AppError;
use mongodb::bson::DateTime;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OaChatState {
    pub chat_id: String,
    pub anchor_message_ids: Vec<String>,
    pub display_name: String,
    pub unread: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OaInboxBatch {
    pub bot_id: String,
    pub started_at_ms: i64,
    pub chats: Vec<OaChatState>,
}

impl OaInboxBatch {
    pub fn validate(&self) -> Result<(), AppError> {
        let now = DateTime::now().timestamp_millis();
        if !super::oa_sync::valid_id(&self.bot_id)
            || self.started_at_ms < now - 300_000
            || self.started_at_ms > now + 30_000
            || self.chats.len() > 100
            || self.chats.iter().any(|chat| {
                !super::oa_sync::valid_id(&chat.chat_id)
                    || chat.display_name.trim().is_empty()
                    || chat.display_name.chars().count() > 100
                    || chat.anchor_message_ids.is_empty()
                    || chat.anchor_message_ids.len() > 20
                    || chat
                        .anchor_message_ids
                        .iter()
                        .any(|id| !super::oa_sync::valid_id(id))
            })
        {
            return Err(AppError::BadRequest);
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RefreshResult {
    pub state: String,
}

impl RefreshResult {
    pub fn validate(&self) -> Result<(), AppError> {
        if ![
            "DONE",
            "NEEDS_LOGIN",
            "UNSUPPORTED_SCHEMA",
            "UNLINKED",
            "LINE_UNAVAILABLE",
        ]
        .contains(&self.state.as_str())
        {
            return Err(AppError::BadRequest);
        }
        Ok(())
    }
}
