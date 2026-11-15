use crate::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BillingRules {
    pub customer_name_prefix: Option<String>,
    pub shipping_product: Option<ShippingRule>,
    // Rent and deposit are billed exactly as the admin's bill summary states them, never recomputed from the cart.
    #[serde(default)]
    pub summary_amounts: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ShippingRule {
    pub rental_below_satang: i64,
    pub fee_satang: i64,
    pub product_code: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveInput {
    pub instruction_text: String,
    pub expected_version: i64,
}

fn invalid() -> AppError {
    AppError::BillingPolicy("BILLING_RULES_INVALID")
}

fn amount(text: &str) -> Result<i64, AppError> {
    let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
    if whole.is_empty()
        || whole.len() > 7
        || !whole.bytes().all(|c| c.is_ascii_digit())
        || fraction.len() > 2
        || !fraction.bytes().all(|c| c.is_ascii_digit())
    {
        return Err(invalid());
    }
    let whole = whole.parse::<i64>().map_err(|_| invalid())?;
    let fraction = format!("{fraction:0<2}")
        .parse::<i64>()
        .map_err(|_| invalid())?;
    let value = whole * 100 + fraction;
    if !(1..=100_000_000).contains(&value) {
        return Err(invalid());
    }
    Ok(value)
}

// The first listed option that prefixes the text, returning the rest; list longer options first.
fn strip_any<'a>(text: &'a str, options: &[&str]) -> Option<&'a str> {
    options.iter().find_map(|option| text.strip_prefix(option))
}

// "แอดมินสรุปบิลให้เอาค่าเช่า ค่าประกันตรงหน้าสรุปบิลเป็นหลัก ไม่คำนวณจากตะกร้าเอง" and its close
// variants, matched on whitespace-free text. Every part is fixed wording and nothing may follow it.
fn summary_amounts_clause(compact: &str) -> bool {
    let rest = strip_any(compact, &["แอดมินสรุปบิลให้", "แอดมินสรุปบิล", "ให้", ""])
        .and_then(|rest| strip_any(rest, &["เอา", "ใช้"]))
        .and_then(|rest| rest.strip_prefix("ค่าเช่า"))
        .and_then(|rest| strip_any(rest, &["และ", "กับ", ",", "/", ""]))
        .and_then(|rest| rest.strip_prefix("ค่าประกัน"))
        .and_then(|rest| strip_any(rest, &["ตรง", "จาก", "ตาม", "ใน"]))
        .and_then(|rest| strip_any(rest, &["หน้าสรุปบิล", "สรุปบิล"]))
        .and_then(|rest| rest.strip_prefix("เป็นหลัก"))
        .and_then(|rest| strip_any(rest, &[",", ""]));
    let Some(rest) = rest else {
        return false;
    };
    rest.is_empty()
        || rest
            .strip_prefix("ไม่คำนวณจากตะกร้า")
            .is_some_and(|tail| tail.is_empty() || tail == "เอง")
}

// Parse a closed billing language. Raw text is never a prompt, program, URL or tool argument.
// Unknown clauses fail closed rather than trusting a model or a blacklist to identify scripts.
pub fn compile(text: &str) -> Result<BillingRules, AppError> {
    if text.chars().count() > 4000 || text.chars().any(|c| (c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
        || matches!(c, '\u{200b}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')) { return Err(invalid()); }
    let mut rules = BillingRules::default();
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let compact: String = line.chars().filter(|c| !c.is_whitespace()).collect();
        let prefix = ["ชื่อลูกค้าทุกคนต้องมี", "ชื่อลูกค้าต้องมี"]
            .iter()
            .find_map(|start| compact.strip_prefix(start));
        if let Some(prefix) = prefix.and_then(|value| value.strip_suffix("นำหน้า")) {
            let prefix = prefix.trim_matches(['`', '"', '\'']);
            if rules.customer_name_prefix.is_some()
                || prefix.is_empty()
                || prefix.chars().count() > 20
                || !prefix.chars().all(|c| {
                    c.is_alphanumeric() || matches!(c, '\u{0e00}'..='\u{0e7f}' | '.' | '-')
                })
            {
                return Err(invalid());
            }
            rules.customer_name_prefix = Some(prefix.to_string());
            continue;
        }
        if summary_amounts_clause(&compact) {
            if rules.summary_amounts {
                return Err(invalid());
            }
            rules.summary_amounts = true;
            continue;
        }
        let condition = ["หากค่าเช่าในออเดอร์นั้นต่ำกว่า", "หากค่าเช่าต่ำกว่า", "ถ้าค่าเช่าต่ำกว่า"]
            .iter()
            .find_map(|start| compact.strip_prefix(start));
        if let Some(condition) = condition {
            let (threshold, rest) = condition.split_once("บาท").ok_or_else(invalid)?;
            let rest = rest
                .trim_start_matches(',')
                .strip_prefix("จะมีค่าขนส่ง")
                .ok_or_else(invalid)?;
            let (fee, rest) = rest.split_once("บาท").ok_or_else(invalid)?;
            let rest = rest
                .trim_start_matches(',')
                .strip_prefix("ให้เพิ่มค่าขนส่งจาก")
                .ok_or_else(invalid)?;
            let rest = rest.to_ascii_lowercase();
            let code = rest.strip_prefix("stockproductcode:").ok_or_else(invalid)?;
            if rules.shipping_product.is_some()
                || code.is_empty()
                || code.len() > 60
                || !code
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_'))
            {
                return Err(invalid());
            }
            rules.shipping_product = Some(ShippingRule {
                rental_below_satang: amount(threshold)?,
                fee_satang: amount(fee)?,
                product_code: code.to_ascii_uppercase(),
            });
            continue;
        }
        return Err(invalid());
    }
    Ok(rules)
}

impl BillingRules {
    pub fn customer_name(&self, name: &str) -> String {
        let name = name.trim();
        match &self.customer_name_prefix {
            Some(prefix) if !name.is_empty() && !name.starts_with(prefix) => {
                format!("{prefix} {name}")
            }
            _ => name.to_string(),
        }
    }
    pub fn shipping_applies(&self, net_rental: i64) -> bool {
        self.shipping_product
            .as_ref()
            .is_some_and(|rule| net_rental >= 0 && net_rental < rule.rental_below_satang)
    }
}
