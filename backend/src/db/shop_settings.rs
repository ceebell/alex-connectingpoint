use crate::{
    error::AppError,
    models::shop_settings::{BillingRules, SaveInput, compile},
    state::AppState,
};
use mongodb::{
    bson::{self, DateTime, Document, doc, oid::ObjectId},
    options::ReturnDocument,
};
use serde_json::{Value, json};

pub struct Settings {
    pub version: i64,
    pub rules: BillingRules,
    pub record: Option<Document>,
}

pub async fn load(state: &AppState) -> Result<Settings, AppError> {
    let record = state
        .collection("shop_settings")
        .find_one(state.scope())
        .await?;
    let (version, rules) = match &record {
        Some(record) => (
            record.get_i64("version").map_err(|_| AppError::Data)?,
            compile(
                record
                    .get_str("instruction_text")
                    .map_err(|_| AppError::Data)?,
            )?,
        ),
        None => (0, BillingRules::default()),
    };
    Ok(Settings {
        version,
        rules,
        record,
    })
}

pub async fn view(state: &AppState) -> Result<Value, AppError> {
    let settings = load(state).await?;
    Ok(
        json!({"version":settings.version,"instruction_text":settings.record.as_ref().and_then(|record| record.get_str("instruction_text").ok()).unwrap_or(""),
        "rules":settings.rules,"updated_by":settings.record.as_ref().and_then(|record|record.get_str("updated_by").ok()),
        "updated_at":settings.record.as_ref().and_then(|record|record.get_datetime("updated_at").ok())}),
    )
}

pub async fn save(state: &AppState, input: SaveInput) -> Result<Value, AppError> {
    let access = super::team::require_owner(state)?;
    if !(0..i64::MAX).contains(&input.expected_version) {
        return Err(AppError::BadRequest);
    }
    let text = input.instruction_text.trim();
    let rules = compile(text)?;
    let now = DateTime::now();
    let audit = doc! {"version":input.expected_version+1,"instruction_text":text,"rules":bson::to_document(&rules).map_err(|_|AppError::Data)?,"updated_by":&access.name,"updated_by_id":&access.line_user_id,"updated_at":now};
    let mut filter = state.scope();
    filter.insert("version", input.expected_version);
    let change = doc! {"$set":&audit,"$setOnInsert":{"_id":ObjectId::new(),"created_at":now},"$push":{"history":{"$each":[audit.clone()],"$slice":-100}}};
    match state
        .collection("shop_settings")
        .find_one_and_update(filter, change)
        .upsert(input.expected_version == 0)
        .return_document(ReturnDocument::After)
        .await
    {
        Ok(Some(_)) => view(state).await,
        Ok(None) => Err(AppError::RecordConflict),
        Err(error) if super::events::duplicate(&error) => Err(AppError::RecordConflict),
        Err(error) => Err(error.into()),
    }
}

// Rent after discount and before any fee line, which the shipping threshold compares against. Under the
// summary-amounts rule it is the summary's ค่าเช่า as written, not a total recomputed from the cart.
pub fn net_rental(group: &crate::models::checkout::Group) -> Result<i64, AppError> {
    if group.amounts_from_summary {
        return group.quoted_rental_fee_satang.ok_or(AppError::BillNotReady);
    }
    Ok(group
        .lines
        .iter()
        .map(|line| line.rental_fee_satang * line.quantity)
        .sum::<i64>()
        - group.rental_discount_satang)
}

pub async fn apply_shipping(
    state: &AppState,
    settings: &Settings,
    group: &mut crate::models::checkout::Group,
) -> Result<(), AppError> {
    let Some(rule) = &settings.rules.shipping_product else {
        return Ok(());
    };
    group
        .lines
        .retain(|line| !line.code.eq_ignore_ascii_case(&rule.product_code));
    if group.lines.is_empty() {
        return Err(AppError::BillNotReady);
    }
    group.quoted_shipping_fee_satang = Some(0);
    if !settings.rules.shipping_applies(net_rental(group)?) {
        return Ok(());
    }
    let session = crate::alex::login(state).await?;
    let input = crate::models::product::SearchInput {
        draft_version: 0,
        context_hash: String::new(),
        item_index: 0,
        branch_id: group.branch_id.clone(),
        mode: crate::models::product::SearchMode::Code,
        query: Some(rule.product_code.clone()),
        color: None,
        size: None,
        image_event_id: None,
        category: None,
        texture: None,
        page: 1,
    };
    let (products, more) = crate::alex::search_page(state, &session, &input).await?;
    if more || products.len() != 1 {
        return Err(AppError::BillingPolicy("BILLING_FEE_UNAVAILABLE"));
    }
    let product = products.into_iter().next().ok_or(AppError::Data)?;
    if product.rental_fee_satang != Some(rule.fee_satang)
        || product.deposit_satang != Some(0)
        || group.lines.len() >= 20
    {
        return Err(AppError::BillingPolicy("BILLING_FEE_UNAVAILABLE"));
    }
    let index = group
        .lines
        .iter()
        .map(|line| line.item_index)
        .max()
        .unwrap_or(0)
        + 1;
    group.lines.push(crate::models::checkout::Line {
        item_index: index,
        product_item_id: product.product_item_id,
        code: product.code,
        name: product.name,
        color: product.color,
        size: product.size,
        quantity: 1,
        rental_fee_satang: rule.fee_satang,
        deposit_satang: 0,
        image_url: product.image_url,
    });
    group.notes = format!(
        "ค่าขนส่งตามเงื่อนไขร้าน {}.{:02} บาท · {}",
        rule.fee_satang / 100,
        rule.fee_satang % 100,
        rule.product_code
    );
    Ok(())
}
