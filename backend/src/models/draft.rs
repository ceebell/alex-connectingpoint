use crate::{error::AppError, models::customer::normalize_phone};
use serde::{Deserialize, Serialize, de::IgnoredAny};
use std::collections::{BTreeMap, BTreeSet};
pub const MAX_DRAFT_IMAGES: usize = 5;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DraftFields {
    pub intent: Option<String>,
    pub customer_name: Option<String>,
    pub customer_phone: Option<String>,
    pub customer_address: Option<String>,
    pub branch: Option<String>,
    #[serde(default)]
    pub items: Vec<DraftItem>,
    #[serde(default)]
    pub rental: Rental,
    #[serde(default)]
    pub amounts: Amounts,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DraftItem {
    pub product_code: Option<String>,
    pub product_name: Option<String>,
    // Alex product line (e.g. a poster title like "Haatiya"), resolved to a category code for search.
    pub category: Option<String>,
    pub color: Option<String>,
    pub size: Option<String>,
    pub quantity: Option<i64>,
    // Retired: dates, amounts and notes belong to the whole order. Older drafts may still carry
    // per-item values, so they are accepted on input but never stored again.
    #[serde(default, skip_serializing)]
    pub rental: IgnoredAny,
    #[serde(default, skip_serializing)]
    pub amounts: IgnoredAny,
    #[serde(default, skip_serializing)]
    pub notes: IgnoredAny,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Rental {
    pub pickup_date: Option<String>,
    pub return_date: Option<String>,
    pub fulfillment_method: Option<String>,
    // One of RECEIVING_METHODS when fulfillment is delivery.
    pub delivery_provider: Option<String>,
    // Retired: Alex books only pickup and return dates, and a return-by time belongs in notes.
    // Older drafts may still carry these keys, so they are accepted but never stored again.
    #[serde(default, skip_serializing)]
    pub usage_date: IgnoredAny,
    #[serde(default, skip_serializing)]
    pub dispatch_date: IgnoredAny,
    #[serde(default, skip_serializing)]
    pub return_shipping_deadline: IgnoredAny,
}

// Alex's own "receiving method" choices; store pickup is the first one.
pub const STORE_PICKUP: &str = "มารับเอง";
pub const RECEIVING_METHODS: [&str; 6] =
    [STORE_PICKUP, "Line Man", "Grab", "Kerry", "Flash", "ขนส่งอื่น"];

// Maps a carrier name as written (e.g. "flash express", "เคอรี่") to an Alex receiving method.
pub fn receiving_method(carrier: &str) -> Option<&'static str> {
    let key: String = carrier
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_lowercase();
    match key.as_str() {
        "flash" | "flashexpress" | "แฟลช" => Some("Flash"),
        "kerry" | "kerryexpress" | "เคอรี่" => Some("Kerry"),
        "grab" | "grabexpress" | "แกร็บ" => Some("Grab"),
        "lineman" | "ไลน์แมน" => Some("Line Man"),
        "ขนส่งอื่น" => Some("ขนส่งอื่น"),
        _ => None,
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Amounts {
    // Integer satang, never floating point. Fees are unverified quoted amounts.
    pub rental_fee_satang: Option<i64>,
    pub deposit_satang: Option<i64>,
    pub shipping_fee_satang: Option<i64>,
    // Order discounts off the Alex catalog totals, sent as rentalDiscount/bailDiscount.
    #[serde(default)]
    pub rental_discount_satang: Option<i64>,
    #[serde(default)]
    pub deposit_discount_satang: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub field: String,
    pub source_type: String,
    pub source_ref: String,
    pub quote: String,
}

// A pickup/return pair read from an image or summary. Like questions, it is a suggestion the
// admin may apply to the order; it is never booked without that choice.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DateOption {
    pub pickup_date: Option<String>,
    pub return_date: Option<String>,
    pub label: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DraftResult {
    pub fields: DraftFields,
    #[serde(default)]
    pub evidence: Vec<Evidence>,
    #[serde(default)]
    pub conflicts: Vec<String>,
    #[serde(default)]
    pub questions: Vec<String>,
    #[serde(default)]
    pub date_options: Vec<DateOption>,
}

impl DraftResult {
    // Evidence for retired fields has no stored value to support, so it is dropped with them.
    pub fn drop_retired_evidence(&mut self) {
        self.evidence.retain(|item| {
            let parts: Vec<_> = item.field.split('.').collect();
            let per_item = parts.len() >= 3
                && parts[0] == "items"
                && matches!(parts[2], "rental" | "amounts" | "notes");
            !per_item
                && !matches!(
                    parts.last(),
                    Some(&("usage_date" | "dispatch_date" | "return_shipping_deadline"))
                )
        });
    }
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct PrepareInput {
    pub request_id: String,
    pub expected_version: i64,
    pub summary: String,
    #[serde(default)]
    pub image_event_ids: Vec<String>,
    // Supplied only for a manual revision, never trusted as an AI approval.
    pub fields: Option<DraftFields>,
    #[serde(default)]
    pub confirmed_fields: Vec<String>,
    pub context_hash: Option<String>,
}

// Edit picks up a version that was never sent; clone duplicates any version. Both save it as a new version.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopyInput {
    pub request_id: String,
    pub expected_version: i64,
    pub context_hash: String,
    pub mode: CopyMode,
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CopyMode {
    Edit,
    Clone,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SummaryInput {
    pub expected_revision: i64,
    pub text: String,
    #[serde(default)]
    pub image_event_ids: Vec<String>,
    pub context_hash: String,
}

impl SummaryInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.expected_revision < 0
            || self.expected_revision == i64::MAX
            || self.text.chars().count() > 20_000
            || self.context_hash.len() != 64
            || !self.context_hash.bytes().all(|c| c.is_ascii_hexdigit())
            || self.image_event_ids.len() > MAX_DRAFT_IMAGES
            || self.image_event_ids.iter().collect::<BTreeSet<_>>().len()
                != self.image_event_ids.len()
        {
            return Err(AppError::Validation("Invalid admin summary"));
        }
        Ok(())
    }
}

impl PrepareInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.request_id.is_empty()
            || self.request_id.len() > 80
            || !self
                .request_id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-')
            || self.expected_version < 0
            || self.expected_version == i64::MAX
            || self.summary.chars().count() > 20_000
            || self.image_event_ids.len() > MAX_DRAFT_IMAGES
            || self.image_event_ids.iter().collect::<BTreeSet<_>>().len()
                != self.image_event_ids.len()
            || self.confirmed_fields.len() > 500
        {
            return Err(AppError::Validation("Invalid preparation request"));
        }
        if self.fields.is_none()
            && self.summary.trim().is_empty()
            && self.image_event_ids.is_empty()
        {
            return Err(AppError::Validation(
                "An admin summary or image is required",
            ));
        }
        if let Some(fields) = &self.fields {
            fields.validate()?;
        }
        Ok(())
    }
}

impl DraftFields {
    pub fn values(&self) -> BTreeMap<String, serde_json::Value> {
        fn walk(
            prefix: &str,
            value: &serde_json::Value,
            out: &mut BTreeMap<String, serde_json::Value>,
        ) {
            match value {
                serde_json::Value::Object(map) => {
                    for (key, value) in map {
                        walk(
                            &if prefix.is_empty() {
                                key.clone()
                            } else {
                                format!("{prefix}.{key}")
                            },
                            value,
                            out,
                        );
                    }
                }
                serde_json::Value::Array(items) => {
                    for (i, value) in items.iter().enumerate() {
                        walk(&format!("{prefix}.{i}"), value, out);
                    }
                }
                serde_json::Value::Null => (),
                _ => {
                    out.insert(prefix.to_string(), value.clone());
                }
            }
        }
        let mut values = BTreeMap::new();
        if let Ok(value) = serde_json::to_value(self) {
            walk("", &value, &mut values);
        }
        values
    }

    pub fn validate(&self) -> Result<(), AppError> {
        if self.items.len() > 20
            || self.values().values().any(|v| {
                v.as_str()
                    .is_some_and(|s| s.trim().is_empty() || s.len() > 4000)
            })
        {
            return Err(AppError::Validation(
                "Draft contains empty or oversized fields",
            ));
        }
        if self
            .items
            .iter()
            .any(|i| i.quantity.is_some_and(|n| !(1..=999).contains(&n)))
        {
            return Err(AppError::Validation("Quantity must be between 1 and 999"));
        }
        self.rental.validate()?;
        self.amounts.validate()?;
        Ok(())
    }

    // Applied after evidence checks: these are house rules, not facts read from a source.
    // The shop ships by Flash unless the order says otherwise; carriers use Alex's choices, and
    // an unlisted carrier becomes "ขนส่งอื่น" with its name kept in the notes.
    pub fn normalize(&mut self) {
        let method = self
            .rental
            .fulfillment_method
            .get_or_insert_with(|| "delivery".into());
        if method == "delivery" {
            let written = self.rental.delivery_provider.take();
            let carrier = match written.as_deref() {
                None => "Flash",
                Some(name) => receiving_method(name).unwrap_or_else(|| {
                    let note = format!("ขนส่ง: {name}");
                    if !self
                        .notes
                        .as_deref()
                        .is_some_and(|notes| notes.contains(&note))
                    {
                        self.notes = Some(
                            [self.notes.take(), Some(note)]
                                .into_iter()
                                .flatten()
                                .collect::<Vec<_>>()
                                .join("\n"),
                        );
                    }
                    "ขนส่งอื่น"
                }),
            };
            self.rental.delivery_provider = Some(carrier.into());
        } else {
            self.rental.delivery_provider = None;
        }
        self.customer_phone = self
            .customer_phone
            .as_deref()
            .map(normalize_phone)
            .filter(|phone| !phone.is_empty());
        for item in &mut self.items {
            item.quantity.get_or_insert(1);
        }
    }

    // Products are chosen into the inquiry cart, so a draft only needs the order-level facts.
    pub fn missing_fields(&self) -> Vec<String> {
        let values = self.values();
        [
            "customer_name",
            "rental.pickup_date",
            "rental.return_date",
            "rental.fulfillment_method",
        ]
        .into_iter()
        .filter(|key| !values.contains_key(*key))
        .map(String::from)
        .collect()
    }
}

impl Rental {
    pub fn validate(&self) -> Result<(), AppError> {
        for date in [&self.pickup_date, &self.return_date].into_iter().flatten() {
            if !valid_date(date) {
                return Err(AppError::Validation(
                    "Dates must be valid Gregorian YYYY-MM-DD values",
                ));
            }
        }
        if let (Some(pickup), Some(returned)) = (&self.pickup_date, &self.return_date)
            && returned < pickup
        {
            return Err(AppError::Validation(
                "Return date cannot precede pickup date",
            ));
        }
        if let Some(method) = &self.fulfillment_method
            && !["store_pickup", "delivery"].contains(&method.as_str())
        {
            return Err(AppError::Validation("Unknown fulfillment method"));
        }
        Ok(())
    }
}

impl Amounts {
    pub fn validate(&self) -> Result<(), AppError> {
        if [
            self.rental_fee_satang,
            self.deposit_satang,
            self.shipping_fee_satang,
            self.rental_discount_satang,
            self.deposit_discount_satang,
        ]
        .into_iter()
        .flatten()
        .any(|amount| !(0..=10_000_000_000).contains(&amount))
        {
            return Err(AppError::Validation("Invalid amount in satang"));
        }
        Ok(())
    }
}

pub fn valid_date(date: &str) -> bool {
    let bytes = date.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes
            .iter()
            .enumerate()
            .any(|(i, c)| i != 4 && i != 7 && !c.is_ascii_digit())
    {
        return false;
    }
    let (Ok(year), Ok(month), Ok(day)) = (
        date[..4].parse::<u32>(),
        date[5..7].parse::<u32>(),
        date[8..].parse::<u32>(),
    ) else {
        return false;
    };
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    (1900..=2199).contains(&year) && day >= 1 && day <= days
}

// Models often collapse runs of spaces or drop emoji variation selectors when quoting bill text
// such as "ค่าเช่า  600  บาท", so quotes are compared with those differences removed.
fn quote_text(value: &str) -> String {
    value
        .chars()
        .filter(|c| !matches!(c, '\u{fe0e}' | '\u{fe0f}' | '\u{200b}'..='\u{200d}'))
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn validate_evidence(
    result: &DraftResult,
    sources: &BTreeMap<String, String>,
    images: &BTreeSet<String>,
) -> Result<(), AppError> {
    result.fields.validate()?;
    let values = result.fields.values();
    if result.evidence.len() > 500
        || result.conflicts.len() > 50
        || result.questions.len() > 20
        || result
            .conflicts
            .iter()
            .chain(&result.questions)
            .any(|s| s.len() > 2000)
    {
        return Err(AppError::Validation("Oversized AI result"));
    }
    if result.date_options.len() > 10
        || result.date_options.iter().any(|option| {
            option.label.trim().is_empty()
                || option.label.chars().count() > 300
                || (option.pickup_date.is_none() && option.return_date.is_none())
                || [&option.pickup_date, &option.return_date]
                    .into_iter()
                    .flatten()
                    .any(|date| !valid_date(date))
                || matches!((&option.pickup_date, &option.return_date), (Some(pickup), Some(returned)) if returned < pickup)
        })
    {
        return Err(AppError::Validation("Invalid date options"));
    }
    let texts: BTreeMap<_, _> = sources
        .iter()
        .map(|(key, text)| (key, quote_text(text)))
        .collect();
    let mut covered = BTreeSet::new();
    for item in &result.evidence {
        let key = format!("{}:{}", item.source_type, item.source_ref);
        let Some(source) = texts.get(&key) else {
            return Err(AppError::Validation("Unknown evidence source"));
        };
        let quote = quote_text(&item.quote);
        if !values.contains_key(&item.field)
            || quote.is_empty()
            || item.quote.len() > 4000
            || (!images.contains(&key) && !source.contains(&quote))
        {
            return Err(AppError::Validation(
                "Evidence must quote a supplied source",
            ));
        }
        covered.insert(item.field.clone());
    }
    if values.keys().any(|key| !covered.contains(key)) {
        return Err(AppError::Validation(
            "Every extracted field requires evidence",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn summaries_accept_thai_text_and_bound_revisions_and_images() {
        let mut input = SummaryInput {
            expected_revision: 0,
            text: "ก".repeat(20_000),
            image_event_ids: vec![],
            context_hash: "a".repeat(64),
        };
        assert!(input.validate().is_ok());
        input.text.push('ก');
        assert!(input.validate().is_err());
        input.text.clear();
        input.expected_revision = i64::MAX;
        assert!(input.validate().is_err());
        input.expected_revision = 0;
        input.image_event_ids = vec!["same-image".into(), "same-image".into()];
        assert!(input.validate().is_err());
    }

    #[test]
    fn dates_money_and_evidence_are_checked() {
        assert!(valid_date("2028-02-29"));
        for date in [
            "2026-02-29",
            "2569-11-15",
            "2026-13-01",
            "26-1-1",
            "2026-00-00",
        ] {
            assert!(!valid_date(date));
        }
        let mut fields = DraftFields {
            customer_name: Some("A".into()),
            ..Default::default()
        };
        fields.amounts.deposit_satang = Some(-1);
        assert!(fields.validate().is_err());
        fields.amounts.deposit_satang = None;
        let mut result = DraftResult {
            fields,
            evidence: vec![],
            conflicts: vec![],
            questions: vec![],
            date_options: vec![],
        };
        let sources = BTreeMap::from([("admin_summary:s1".into(), "Customer A".into())]);
        assert!(validate_evidence(&result, &sources, &BTreeSet::new()).is_err());
        result.evidence.push(Evidence {
            field: "customer_name".into(),
            source_type: "admin_summary".into(),
            source_ref: "s1".into(),
            quote: "Customer A".into(),
        });
        assert!(validate_evidence(&result, &sources, &BTreeSet::new()).is_ok());
        result.evidence[0].source_ref = "another-inquiry".into();
        assert!(validate_evidence(&result, &sources, &BTreeSet::new()).is_err());
    }

    #[test]
    fn evidence_quotes_ignore_collapsed_spaces_and_emoji_selectors() {
        let sources = BTreeMap::from([(
            "admin_summary:s1".into(),
            "ค่าเช่า  600  บาท\n⚡\u{fe0f} จัดส่งชุด    : flash ฟรี".into(),
        )]);
        let mut fields = DraftFields::default();
        fields.amounts.rental_fee_satang = Some(60_000);
        let evidence = |quote: &str| Evidence {
            field: "amounts.rental_fee_satang".into(),
            source_type: "admin_summary".into(),
            source_ref: "s1".into(),
            quote: quote.into(),
        };
        let mut result = DraftResult {
            fields,
            evidence: vec![evidence("ค่าเช่า 600 บาท")],
            conflicts: vec![],
            questions: vec![],
            date_options: vec![],
        };
        let images = BTreeSet::new();
        assert!(validate_evidence(&result, &sources, &images).is_ok());
        result.evidence = vec![evidence("⚡ จัดส่งชุด : flash ฟรี")];
        assert!(validate_evidence(&result, &sources, &images).is_ok());
        for quote in ["ค่าเช่า 700 บาท", "   ", "\u{fe0f}"] {
            result.evidence = vec![evidence(quote)];
            assert!(validate_evidence(&result, &sources, &images).is_err());
        }
    }
}
