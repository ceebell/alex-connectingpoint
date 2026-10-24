use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize)]
pub struct Webhook {
    pub destination: String,
    pub events: Vec<LineEvent>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LineEvent {
    #[serde(rename = "type")]
    pub kind: String,
    pub webhook_event_id: String,
    pub timestamp: i64,
    pub source: Option<Source>,
    pub message: Option<Value>,
    pub postback: Option<Value>,
    pub unsend: Option<Value>,
    // replyToken is intentionally not deserialized or persisted.
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    #[serde(rename = "type")]
    pub kind: String,
    pub user_id: Option<String>,
    pub group_id: Option<String>,
    pub room_id: Option<String>,
}

impl LineEvent {
    pub fn validate(&self) -> bool {
        if self.webhook_event_id.is_empty()
            || self.webhook_event_id.len() > 128
            || self.timestamp < 0
        {
            return false;
        }
        match self.kind.as_str() {
            "message" => self.message.as_ref().is_some_and(|m| {
                m["id"].as_str().is_some_and(|id| !id.is_empty())
                    && m["type"].is_string()
                    && (m["type"] != "text" || m["text"].is_string())
            }),
            "postback" => self
                .postback
                .as_ref()
                .is_some_and(|p| p["data"].is_string()),
            "unsend" => self
                .unsend
                .as_ref()
                .is_some_and(|u| u["messageId"].is_string()),
            _ => true,
        }
    }

    pub fn conversation_key(&self) -> Option<String> {
        let source = self.source.as_ref()?;
        let user = source.user_id.as_deref().filter(|s| !s.is_empty())?;
        let chat = match source.kind.as_str() {
            "user" => user,
            "group" => source.group_id.as_deref()?,
            "room" => source.room_id.as_deref()?,
            _ => return None,
        };
        // JSON encoding avoids delimiter collisions and separates group participants.
        serde_json::to_string(&(&source.kind, chat, user)).ok()
    }
}
