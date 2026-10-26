use crate::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Branch {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Candidate {
    pub product_item_id: String,
    pub code: String,
    pub name: String,
    pub color: Option<String>,
    pub size: Option<String>,
    pub rental_fee_satang: Option<i64>,
    pub deposit_satang: Option<i64>,
    // Public Alex catalog link (images/{shopInfoId}/item/{mainImage}), checked against allowed origins.
    pub image_url: Option<String>,
    pub score: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SearchMode {
    Code,
    Text,
    ProductUrl,
    Image,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SearchInput {
    // Draft binding is optional: cart searches are scoped to the inquiry only.
    #[serde(default)]
    pub draft_version: i64,
    #[serde(default)]
    pub context_hash: String,
    #[serde(default)]
    pub item_index: usize,
    pub branch_id: String,
    pub mode: SearchMode,
    pub query: Option<String>,
    pub color: Option<String>,
    pub size: Option<String>,
    pub image_event_id: Option<String>,
    pub category: Option<String>,
    pub texture: Option<String>,
    #[serde(default = "first_page")]
    pub page: u32,
}
fn first_page() -> u32 {
    1
}

impl SearchInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.draft_version < 0
            || self.item_index >= 20
            || !valid_id(&self.branch_id)
            || !(self.context_hash.is_empty()
                || (self.context_hash.len() == 64
                    && self.context_hash.bytes().all(|b| b.is_ascii_hexdigit())))
            || !(1..=100).contains(&self.page)
            || [
                &self.query,
                &self.color,
                &self.size,
                &self.category,
                &self.texture,
            ]
            .iter()
            .filter_map(|v| v.as_ref())
            .any(|v| v.chars().count() > 500)
            || (matches!(self.mode, SearchMode::Code | SearchMode::ProductUrl)
                && self.query.as_ref().is_none_or(|v| v.trim().is_empty()))
            || (self.mode == SearchMode::Image && self.image_event_id.is_none())
        {
            return Err(AppError::Validation("Invalid product search"));
        }
        Ok(())
    }
}

pub fn valid_quantity(value: i64) -> bool {
    (1..=999).contains(&value)
}

pub fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

// Convert decimal baht to exact integer satang without floating-point arithmetic.
pub fn satang(value: &serde_json::Value) -> Result<Option<i64>, AppError> {
    if value.is_null() {
        return Ok(None);
    }
    let text = match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        _ => return Err(AppError::AlexFailed),
    };
    let mut parts = text.split('.');
    let whole = parts.next().ok_or(AppError::AlexFailed)?;
    let fraction = parts.next().unwrap_or("");
    if whole.is_empty()
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || fraction.len() > 2
        || !fraction.bytes().all(|b| b.is_ascii_digit())
        || parts.next().is_some()
    {
        return Err(AppError::AlexFailed);
    }
    let whole: i64 = whole.parse().map_err(|_| AppError::AlexFailed)?;
    let cents = if fraction.is_empty() {
        0
    } else {
        fraction.parse::<i64>().map_err(|_| AppError::AlexFailed)?
            * if fraction.len() == 1 { 10 } else { 1 }
    };
    whole
        .checked_mul(100)
        .and_then(|v| v.checked_add(cents))
        .filter(|v| *v <= 10_000_000_000)
        .map(Some)
        .ok_or(AppError::AlexFailed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn catalog_prices_require_exact_satang() {
        assert_eq!(satang(&json!("1500.01")).unwrap(), Some(150001));
        assert_eq!(satang(&json!(1500)).unwrap(), Some(150000));
        assert_eq!(satang(&json!("0.5")).unwrap(), Some(50));
        for value in [
            json!("-1"),
            json!("1.001"),
            json!("1e6"),
            json!("NaN"),
            json!("99999999999999999999"),
        ] {
            assert!(satang(&value).is_err());
        }
    }
}
