use serde::Deserialize;

// The cart belongs to the inquiry, not to a draft version, so product choices survive draft saves.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AddInput {
    pub search_id: String,
    pub product_item_id: String,
    pub quantity: Option<i64>,
}

// Sets a line quantity, or removes the line when quantity is None; the revision guards concurrent edits.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateInput {
    pub item_id: String,
    pub expected_revision: i64,
    pub quantity: Option<i64>,
}

// Dates come from the order form on screen, so a queue check does not need a saved draft.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckInput {
    pub pickup_date: String,
    pub return_date: String,
}
