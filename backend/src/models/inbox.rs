use serde::Deserialize;

#[derive(Deserialize)]
pub struct ReadInput {
    pub message_id: String,
}
