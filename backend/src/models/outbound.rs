use crate::error::AppError;
use serde::{Deserialize, Serialize};

// LINE requires X-Line-Retry-Key to be a UUID; reusing it makes retries safe.
pub fn valid_request_id(value: &str) -> bool {
    value.len() == 36
        && value.char_indices().all(|(i, c)| {
            if [8, 13, 18, 23].contains(&i) {
                c == '-'
            } else {
                c.is_ascii_hexdigit()
            }
        })
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SendInput {
    pub request_id: String,
    pub text: String,
}

impl SendInput {
    pub fn validate(self) -> Result<Self, AppError> {
        let text = self.text.trim().to_string();
        if !valid_request_id(&self.request_id)
            || text.is_empty()
            || text.chars().count() > 5000
            || text
                .chars()
                .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
        {
            return Err(AppError::Validation("Invalid reply message"));
        }
        Ok(Self {
            request_id: self.request_id.to_ascii_lowercase(),
            text,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reply_input_requires_uuid_and_bounded_text() {
        let ok = SendInput {
            request_id: "3F2504E0-4F89-41D3-9A0C-0305E82C3301".into(),
            text: "  สวัสดีค่ะ \n".into(),
        }
        .validate()
        .unwrap();
        assert_eq!(ok.text, "สวัสดีค่ะ");
        assert_eq!(ok.request_id, "3f2504e0-4f89-41d3-9a0c-0305e82c3301");
        for (id, text) in [
            ("not-a-uuid", "hi".to_string()),
            ("3f2504e0-4f89-41d3-9a0c-0305e82c3301", "   ".to_string()),
            ("3f2504e0-4f89-41d3-9a0c-0305e82c3301", "ก".repeat(5001)),
            (
                "3f2504e0-4f89-41d3-9a0c-0305e82c3301",
                "a\u{7}b".to_string(),
            ),
        ] {
            assert!(
                SendInput {
                    request_id: id.into(),
                    text
                }
                .validate()
                .is_err()
            );
        }
    }
}
