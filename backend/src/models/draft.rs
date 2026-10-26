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
    pub color: Option<String>,
    pub size: Option<String>,
    pub quantity: Option<i64>,
    pub rental: Option<Rental>,
    pub amounts: Option<Amounts>,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Rental {
    pub pickup_date: Option<String>,
    pub return_date: Option<String>,
    pub fulfillment_method: Option<String>,
    pub delivery_provider: Option<String>,
    pub return_shipping_deadline: Option<String>,
    // Retired: Alex books only pickup and return dates. Older drafts may still carry these keys,
    // so they are accepted on input but never stored, validated or shown again.
    #[serde(default, skip_serializing)]
    pub usage_date: IgnoredAny,
    #[serde(default, skip_serializing)]
    pub dispatch_date: IgnoredAny,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Amounts {
    // Integer satang, never floating point. These are unverified quoted amounts.
    pub rental_fee_satang: Option<i64>,
    pub deposit_satang: Option<i64>,
    pub shipping_fee_satang: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub field: String,
    pub source_type: String,
    pub source_ref: String,
    pub quote: String,
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
}

impl DraftResult {
    // Evidence for retired rental dates has no stored value to support, so it is dropped with them.
    pub fn drop_retired_evidence(&mut self) {
        self.evidence.retain(|item| {
            !matches!(
                item.field.rsplit('.').next(),
                Some("usage_date" | "dispatch_date")
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
        for item in &self.items {
            item.effective_rental(&self.rental).validate()?;
            if let Some(amounts) = &item.amounts {
                amounts.validate()?;
            }
        }
        Ok(())
    }

    // Applied after evidence checks: these are house rules, not facts read from a source.
    pub fn normalize(&mut self) {
        self.customer_phone = self
            .customer_phone
            .as_deref()
            .map(normalize_phone)
            .filter(|phone| !phone.is_empty());
        for item in &mut self.items {
            item.quantity.get_or_insert(1);
        }
    }

    pub fn missing_fields(&self) -> Vec<String> {
        let values = self.values();
        let mut missing = Vec::new();
        for key in ["customer_name", "branch"] {
            if !values.contains_key(key) {
                missing.push(key.into());
            }
        }
        if self.items.is_empty() {
            missing.push("items".into());
            for key in [
                "rental.pickup_date",
                "rental.return_date",
                "rental.fulfillment_method",
            ] {
                if !values.contains_key(key) {
                    missing.push(key.into());
                }
            }
        }
        for (i, item) in self.items.iter().enumerate() {
            for (name, present) in [
                (
                    "product_name",
                    item.product_code.is_some() || item.product_name.is_some(),
                ),
                ("color", item.color.is_some()),
                ("size", item.size.is_some()),
                ("quantity", item.quantity.is_some()),
            ] {
                if !present {
                    missing.push(format!("items.{i}.{name}"));
                }
            }
            let rental = item.effective_rental(&self.rental);
            for (name, present) in [
                ("pickup_date", rental.pickup_date.is_some()),
                ("return_date", rental.return_date.is_some()),
                ("fulfillment_method", rental.fulfillment_method.is_some()),
            ] {
                if !present {
                    missing.push(format!("items.{i}.rental.{name}"));
                }
            }
        }
        missing
    }
}

impl DraftItem {
    pub fn effective_rental(&self, common: &Rental) -> Rental {
        let own = self.rental.as_ref().unwrap_or(common);
        Rental {
            pickup_date: own
                .pickup_date
                .clone()
                .or_else(|| common.pickup_date.clone()),
            return_date: own
                .return_date
                .clone()
                .or_else(|| common.return_date.clone()),
            fulfillment_method: own
                .fulfillment_method
                .clone()
                .or_else(|| common.fulfillment_method.clone()),
            delivery_provider: own
                .delivery_provider
                .clone()
                .or_else(|| common.delivery_provider.clone()),
            return_shipping_deadline: own
                .return_shipping_deadline
                .clone()
                .or_else(|| common.return_shipping_deadline.clone()),
            ..Default::default()
        }
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
        if let Some(time) = &self.return_shipping_deadline {
            let parts: Vec<_> = time.split(':').collect();
            if time.len() != 5
                || parts.len() != 2
                || time.bytes().enumerate().any(|(index, byte)| {
                    if index == 2 {
                        byte != b':'
                    } else {
                        !byte.is_ascii_digit()
                    }
                })
                || parts[0].parse::<u8>().ok().is_none_or(|h| h > 23)
                || parts[1].parse::<u8>().ok().is_none_or(|m| m > 59)
            {
                return Err(AppError::Validation("Shipping deadline must be HH:MM"));
            }
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
    let mut covered = BTreeSet::new();
    for item in &result.evidence {
        let key = format!("{}:{}", item.source_type, item.source_ref);
        let Some(source) = sources.get(&key) else {
            return Err(AppError::Validation("Unknown evidence source"));
        };
        if !values.contains_key(&item.field)
            || item.quote.is_empty()
            || item.quote.len() > 4000
            || (!images.contains(&key) && !source.contains(&item.quote))
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
}
