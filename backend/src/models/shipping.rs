use crate::{error::AppError, models::outbound::valid_request_id};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const MAX_IMAGE: usize = 8 * 1024 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UploadInput {
    pub request_id: String,
    pub name: String,
    pub image_base64: String,
}
impl UploadInput {
    pub fn image(&self) -> Result<(Vec<u8>, &'static str), AppError> {
        if !valid_request_id(&self.request_id)
            || self.name.is_empty()
            || self.name.chars().count() > 200
            || self.name.chars().any(char::is_control)
            || self.image_base64.len() > MAX_IMAGE / 3 * 4 + 8
        {
            return Err(AppError::Validation("Invalid shipping receipt"));
        }
        let bytes = STANDARD
            .decode(&self.image_base64)
            .map_err(|_| AppError::BadRequest)?;
        let mime = crate::models::outbound::image_type(&bytes)
            .ok_or(AppError::Validation("Receipt must be JPEG or PNG"))?;
        if bytes.len() > MAX_IMAGE {
            return Err(AppError::Validation("Receipt exceeds 8 MB"));
        }
        Ok((bytes, mime))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExtractedParcel {
    pub row: i64,
    pub recipient_name: Option<String>,
    pub tracking_number: Option<String>,
    pub postal_code: Option<String>,
    pub service: Option<String>,
    pub shipping_fee_satang: Option<i64>,
    pub raw_text: String,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Extraction {
    pub carrier: String,
    pub receipt_number: Option<String>,
    pub receipt_date: Option<String>,
    pub total_satang: Option<i64>,
    pub parcels: Vec<ExtractedParcel>,
}
fn text(value: &str, max: usize) -> bool {
    value.chars().count() <= max
        && !value
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
}
fn money(value: Option<i64>) -> bool {
    value.is_none_or(|value| (0..=100_000_000).contains(&value))
}
impl Extraction {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.carrier != "THAILAND_POST"
            || self.parcels.is_empty()
            || self.parcels.len() > 100
            || !money(self.total_satang)
            || self.receipt_number.as_ref().is_some_and(|s| !text(s, 100))
            || self
                .receipt_date
                .as_ref()
                .is_some_and(|s| !crate::models::draft::valid_date(s))
        {
            return Err(AppError::Validation("Invalid receipt extraction"));
        }
        let mut rows = BTreeSet::new();
        for item in &self.parcels {
            if !(1..=1000).contains(&item.row)
                || !rows.insert(item.row)
                || !text(&item.raw_text, 3000)
                || !money(item.shipping_fee_satang)
                || [
                    &item.recipient_name,
                    &item.tracking_number,
                    &item.postal_code,
                    &item.service,
                ]
                .into_iter()
                .flatten()
                .any(|s| !text(s, 200))
            {
                return Err(AppError::Validation("Invalid parcel extraction"));
            }
        }
        Ok(())
    }
}

pub fn tracking(value: &str) -> String {
    value
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .collect::<String>()
        .to_ascii_uppercase()
}
pub fn valid_tracking(value: &str) -> bool {
    let b = value.as_bytes();
    b.len() == 13
        && b[..2].iter().all(u8::is_ascii_uppercase)
        && b[2..11].iter().all(u8::is_ascii_digit)
        && &b[11..] == b"TH"
}
pub fn normalized_name(value: &str) -> String {
    let lower = value.trim().to_lowercase();
    let lower = ["k.", "k ", "คุณ", "นางสาว", "น.ส.", "นาย", "นาง"]
        .iter()
        .find_map(|prefix| lower.strip_prefix(prefix))
        .unwrap_or(&lower);
    lower
        .chars()
        .filter(|c| c.is_alphanumeric() || ('\u{0e00}'..='\u{0e7f}').contains(c))
        .collect()
}
// A match is a suggestion only. No score can authorize saving or sending a parcel.
pub fn name_score(written: &str, customer: &str) -> i32 {
    let a = normalized_name(written);
    let b = normalized_name(customer);
    if a.is_empty() || b.is_empty() {
        return 0;
    }
    if a == b {
        return 100;
    }
    let (short, long) = if a.chars().count() < b.chars().count() {
        (&a, &b)
    } else {
        (&b, &a)
    };
    if short.chars().count() >= 3 && long.contains(short) {
        return 85;
    }
    let pairs = |s: &str| {
        s.chars()
            .collect::<Vec<_>>()
            .windows(2)
            .map(|p| (p[0], p[1]))
            .collect::<BTreeSet<_>>()
    };
    let (a, b) = (pairs(&a), pairs(&b));
    let denominator = a.len() + b.len();
    if denominator == 0 {
        return 0;
    }
    let score = (200 * a.intersection(&b).count() / denominator) as i32;
    if score >= 60 { score.min(80) } else { 0 }
}

// Notes are supporting evidence only: require a literal name, never fuzzy similarity.
pub fn note_excerpt(query: &str, text: &str) -> Option<String> {
    let needle = normalized_name(query);
    if needle.chars().count() < 3 {
        return None;
    }
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let chars: Vec<_> = collapsed.chars().collect();
    let mut normalized = String::new();
    let mut positions = Vec::new();
    for (index, c) in chars.iter().enumerate() {
        for lower in c.to_lowercase() {
            if lower.is_alphanumeric() || ('\u{0e00}'..='\u{0e7f}').contains(&lower) {
                normalized.push(lower);
                positions.push(index);
            }
        }
    }
    let byte = normalized.find(&needle)?;
    let offset = normalized[..byte].chars().count();
    let start = positions.get(offset)?.saturating_sub(24);
    let end = (positions.get(offset + needle.chars().count() - 1)? + 25)
        .min(chars.len())
        .min(start + 240);
    let excerpt: String = chars[start..end].iter().collect();
    Some(format!(
        "{}{}{}",
        if start > 0 { "…" } else { "" },
        excerpt,
        if end < chars.len() { "…" } else { "" }
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveInput {
    pub expected_version: i64,
    pub submission_id: Option<String>,
    pub inquiry_id: Option<String>,
    pub recipient_name: String,
    pub tracking_number: String,
    pub postal_code: String,
    pub service: String,
    pub shipping_fee_satang: Option<i64>,
    pub message: String,
}
impl SaveInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.expected_version < 0
            || self.expected_version == i64::MAX
            || self.submission_id.is_some() == self.inquiry_id.is_some()
            || self
                .submission_id
                .as_ref()
                .or(self.inquiry_id.as_ref())
                .is_none_or(|id| mongodb::bson::oid::ObjectId::parse_str(id).is_err())
            || self.recipient_name.trim().is_empty()
            || !text(&self.recipient_name, 200)
            || !valid_tracking(&tracking(&self.tracking_number))
            || !money(self.shipping_fee_satang)
            || !text(&self.service, 100)
            || !text(&self.postal_code, 5)
            || (!self.postal_code.is_empty()
                && (self.postal_code.len() != 5
                    || !self.postal_code.bytes().all(|b| b.is_ascii_digit())))
            || self.message.trim().is_empty()
            || !text(&self.message, 5000)
            || !self.message.contains(&tracking(&self.tracking_number))
        {
            return Err(AppError::Validation(
                "Check the recipient, tracking number and message",
            ));
        }
        Ok(())
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VersionInput {
    pub expected_version: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tracking_requires_exactly_one_valid_order_or_chat_destination() {
        let mut value = serde_json::json!({"expected_version":0,"recipient_name":"Recipient","tracking_number":"JD333458790TH","postal_code":"10150","service":"EMS","shipping_fee_satang":2400,"message":"Tracking JD333458790TH"});
        let valid = |value: &serde_json::Value| {
            serde_json::from_value::<SaveInput>(value.clone())
                .unwrap()
                .validate()
                .is_ok()
        };
        assert!(!valid(&value));
        value["inquiry_id"] = "111111111111111111111111".into();
        assert!(valid(&value));
        value["submission_id"] = "222222222222222222222222".into();
        assert!(!valid(&value));
        value.as_object_mut().unwrap().remove("inquiry_id");
        assert!(valid(&value));
        value["submission_id"] = "invalid".into();
        assert!(!valid(&value));
        value.as_object_mut().unwrap().remove("submission_id");
        value["inquiry_id"] = "invalid".into();
        assert!(!valid(&value));
        value["inquiry_id"] = "111111111111111111111111".into();
        value["message"] = "Missing tracking".into();
        assert!(!valid(&value));
    }
    #[test]
    fn tracking_keeps_the_printed_digits_and_names_keep_thai_combining_marks() {
        assert_eq!(tracking("JD 3334 5879 0 TH"), "JD333458790TH");
        assert!(valid_tracking("JD333458790TH"));
        assert!(!valid_tracking("JD33345879OTH"));
        assert!(!valid_tracking("JD333458790XX"));
        assert_eq!(name_score("พิมพ์นิภา", "K.พิมพ์นิภา"), 100);
        assert_eq!(name_score("กัญญาณัฐ", "กัญญาณัฐ ใจดี"), 85);
        assert_eq!(name_score("", "กัญญาณัฐ"), 0);
        assert_eq!(name_score("Alice", "Bob"), 0);
        assert_ne!(normalized_name("พิณ"), normalized_name("พิน"));
    }

    #[test]
    fn note_matching_requires_literal_names_and_returns_a_bounded_current_excerpt() {
        assert_eq!(
            note_excerpt("สมมติ ใจดี", "ชื่อผู้รับ: คุณสมมติ\nใจดี ส่งวันจันทร์"),
            Some("ชื่อผู้รับ: คุณสมมติ ใจดี ส่งวันจันทร์".into())
        );
        assert_eq!(
            note_excerpt("K. Alice Smith", "Recipient: ALICE  SMITH"),
            Some("Recipient: ALICE SMITH".into())
        );
        assert_eq!(note_excerpt("Alice", "Alicia"), None);
        assert_eq!(note_excerpt("เอ", "ชื่อ เอ"), None);
        assert_eq!(note_excerpt("คุณ", "ชื่อผู้รับ"), None);
        let excerpt = note_excerpt(
            "Example Recipient",
            &format!(
                "{} Example Recipient {}",
                "before ".repeat(500),
                "after ".repeat(500)
            ),
        )
        .unwrap();
        assert!(excerpt.contains("Example Recipient"));
        assert!(excerpt.chars().count() <= 242);
        assert!(excerpt.starts_with('…') && excerpt.ends_with('…'));
    }
}
