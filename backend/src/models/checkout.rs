use serde::{Deserialize, Serialize};

// Alex's own payment choices for rent and deposit; the first means not paid yet.
pub const UNPAID: &str = "ยังไม่ชำระ";
pub const PAYMENT_METHODS: [&str; 6] = [
    UNPAID,
    "เงินสด",
    "โอนเงิน",
    "Alipay/wechat",
    "บัตรเครดิต",
    "ช่องทางอื่น",
];

#[derive(Clone, Serialize, Deserialize)]
pub struct Line {
    pub item_index: usize,
    pub product_item_id: String,
    pub code: String,
    pub name: String,
    pub color: Option<String>,
    pub size: Option<String>,
    pub quantity: i64,
    pub rental_fee_satang: i64,
    pub deposit_satang: i64,
    #[serde(default)]
    pub image_url: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Group {
    pub key: String,
    pub branch_id: String,
    pub pickup_date: String,
    pub return_date: String,
    pub fulfillment_method: String,
    pub delivery_provider: Option<String>,
    pub lines: Vec<Line>,
    pub notes: String,
    pub quoted_rental_fee_satang: Option<i64>,
    pub quoted_deposit_satang: Option<i64>,
    pub quoted_shipping_fee_satang: Option<i64>,
    // Order discounts from the draft; validated against catalog totals at review time.
    #[serde(default)]
    pub rental_discount_satang: i64,
    #[serde(default)]
    pub deposit_discount_satang: i64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckInput {
    pub group_key: String,
    pub source_hash: String,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewInput {
    pub group_key: String,
    pub source_hash: String,
    pub customer_name: String,
    pub customer_phone: String,
    pub customer_address: String,
    pub receiving_method: String,
    // One of PAYMENT_METHODS; empty means not paid yet.
    #[serde(default)]
    pub rental_payment_method: String,
    #[serde(default)]
    pub bail_payment_method: String,
    // Alex has no shipping field, so a shipping fee is always written into the order note.
    pub shipping_fee_satang: i64,
    pub birth_date: Option<String>,
    #[serde(default)]
    pub use_free_item_promotion: bool,
    #[serde(default)]
    pub use_discount_promotion: bool,
    #[serde(default)]
    pub notes: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubmitInput {
    pub review_id: String,
    pub confirmed: bool,
    // The admin saw that some items are booked (or the queue could not be read) and sends anyway.
    #[serde(default)]
    pub accept_queue: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolveInput {
    pub submission_id: String,
    pub outcome: String,
    pub order_id: Option<String>,
    pub verified_in_alex: bool,
}
