use crate::error::AppError;
use serde::{Deserialize, Serialize};

// Who performed an admin write: a stable ID for audit plus a display name.
#[derive(Clone, Debug)]
pub struct Actor {
    pub id: String,
    pub name: String,
}

pub fn valid_user_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 64 && value.bytes().all(|b| b.is_ascii_alphanumeric())
}

fn valid_text(value: &Option<String>, max: usize) -> bool {
    value.as_ref().is_none_or(|v| {
        v.chars().count() <= max
            && !v
                .chars()
                .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
    })
}

fn clean(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ProfileInput {
    pub expected_version: i64,
    pub display_name: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub notes: Option<String>,
}

impl ProfileInput {
    pub fn validate(self) -> Result<Self, AppError> {
        let input = Self {
            expected_version: self.expected_version,
            display_name: clean(self.display_name),
            phone: clean(self.phone),
            address: clean(self.address),
            notes: clean(self.notes),
        };
        if input.expected_version < 0
            || !valid_text(&input.display_name, 200)
            || !valid_text(&input.address, 1000)
            || !valid_text(&input.notes, 5000)
            || input.phone.as_ref().is_some_and(|p| {
                p.len() > 40 || !p.chars().all(|c| c.is_ascii_digit() || " +-()".contains(c))
            })
        {
            return Err(AppError::Validation("Invalid customer profile"));
        }
        Ok(input)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ApproveInput {
    pub draft_id: String,
    pub draft_version: i64,
    pub context_hash: String,
}

impl ApproveInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.draft_version < 1
            || self.context_hash.len() != 64
            || !self.context_hash.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(AppError::Validation("Invalid order approval"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_input_trims_and_rejects_unsafe_values() {
        let input = ProfileInput {
            display_name: Some("  คุณเอ ".into()),
            phone: Some(" 081-234-5678 ".into()),
            notes: Some("   ".into()),
            ..Default::default()
        }
        .validate()
        .unwrap();
        assert_eq!(input.display_name.as_deref(), Some("คุณเอ"));
        assert_eq!(input.phone.as_deref(), Some("081-234-5678"));
        assert_eq!(input.notes, None);
        for bad in [
            ProfileInput {
                phone: Some("08x".into()),
                ..Default::default()
            },
            ProfileInput {
                expected_version: -1,
                ..Default::default()
            },
            ProfileInput {
                display_name: Some("a\u{0}b".into()),
                ..Default::default()
            },
            ProfileInput {
                notes: Some("ก".repeat(5001)),
                ..Default::default()
            },
        ] {
            assert!(bad.validate().is_err());
        }
        assert!(valid_user_id("U0123456789abcdef0123456789abcdef"));
        assert!(!valid_user_id("U$bad") && !valid_user_id(""));
    }
}
