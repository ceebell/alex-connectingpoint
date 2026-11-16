use crate::{
    alex,
    db::{cart, drafts, products},
    error::AppError,
    models::{
        cart::CheckInput as CartCheckInput,
        checkout::{
            CheckInput, Group, Line, PAYMENT_METHODS, ResolveInput, ReviewInput, SaveFormInput,
            SubmitInput, UNPAID,
        },
        customer::{Actor, ProfileInput, normalize_phone},
        draft::{Amounts, DraftFields, RECEIVING_METHODS, valid_date},
        payment_slip::{self, BANK_TRANSFER, BankTransferSlip},
    },
    state::AppState,
};
use futures_util::TryStreamExt;
use mongodb::bson::{self, DateTime, Document, doc, oid::ObjectId};
use serde_json::{Value, json};
use std::collections::{BTreeSet, HashMap};

struct Source {
    inquiry: Document,
    draft: Document,
    fields: DraftFields,
    source_hash: String,
    groups: Vec<Group>,
    missing: Vec<Value>,
    has_conflicts: bool,
    closed: Option<&'static str>,
    policy: super::shop_settings::Settings,
    bank_transfer_slips: Vec<BankTransferSlip>,
}

// What the version mapping below reads from each saved version.
pub fn version_keys() -> Document {
    doc! {"_id":1,"draft_version":1,"created_at":1,"cancelled_at":1}
}

// The saved version (id and number) a bill was made from. Bills record it; older bills belong to the
// version that was active when they were sent, the newest one saved before them.
fn bill_version(bill: &Document, versions: &[Document]) -> Option<(ObjectId, i64)> {
    if let Ok(id) = bill.get_object_id("draft_id") {
        return Some((id, bill.get_i64("draft_version").unwrap_or(0)));
    }
    let sent = bill.get_datetime("created_at").ok()?;
    let version = versions
        .iter()
        .filter(|version| {
            version
                .get_datetime("created_at")
                .is_ok_and(|created| created <= sent)
        })
        .max_by_key(|version| version.get_i64("draft_version").unwrap_or(0))?;
    Some((
        version.get_object_id("_id").ok()?,
        version.get_i64("draft_version").ok()?,
    ))
}

// The latest bill that reached Alex, or may have (everything but NOT_CREATED), for each saved version.
pub async fn version_bills(
    state: &AppState,
    inquiry_id: ObjectId,
    versions: &[Document],
) -> Result<HashMap<ObjectId, Document>, AppError> {
    let mut filter = state.scope();
    filter.extend(doc! {"inquiry_id":inquiry_id,"status":{"$ne":"NOT_CREATED"}});
    let bills: Vec<Document> = state
        .collection("alex_submissions")
        .find(filter)
        .sort(doc! {"created_at":1})
        .await?
        .try_collect()
        .await?;
    Ok(bills
        .into_iter()
        .filter_map(|bill| Some((bill_version(&bill, versions)?.0, bill)))
        .collect())
}

// A version that was sent to Alex or cancelled is closed: it stays as a record and can only be cloned.
pub fn closed(version: &Document, bills: &HashMap<ObjectId, Document>) -> Option<&'static str> {
    if version.contains_key("cancelled_at") {
        Some("CANCELLED")
    } else if version
        .get_object_id("_id")
        .is_ok_and(|id| bills.contains_key(&id))
    {
        Some("SENT")
    } else {
        None
    }
}

fn queue_state(results: &[Value]) -> &'static str {
    if results.iter().all(|value| value["state"] == "AVAILABLE") {
        "AVAILABLE"
    } else if results.iter().any(|value| value["state"] == "UNAVAILABLE") {
        "UNAVAILABLE"
    } else {
        "UNKNOWN"
    }
}

// One inquiry is one Alex order: cart lines plus the order-level dates, notes and discounts.
async fn source(state: &AppState, id: ObjectId) -> Result<Source, AppError> {
    let (inquiry, draft, mut fields, snapshot) = products::current(state, id).await?;
    let policy = super::shop_settings::load(state).await?;
    if let Some(name) = &fields.customer_name {
        fields.customer_name = Some(policy.rules.customer_name(name));
    }
    let draft_id = draft.get_object_id("_id").map_err(|_| AppError::Data)?;
    let mut bank_transfer_slips: Vec<BankTransferSlip> = draft
        .get_document("result")
        .ok()
        .and_then(|result| result.get("bank_transfer_slips"))
        .map(|value| bson::from_bson(value.clone()).map_err(|_| AppError::Data))
        .transpose()?
        .unwrap_or_default();
    let selected_images = snapshot
        .events
        .iter()
        .filter(|event| {
            event
                .get_document("payload")
                .ok()
                .and_then(|payload| payload.get_document("message").ok())
                .and_then(|message| message.get_str("type").ok())
                == Some("image")
                && !event.get_bool("withdrawn").unwrap_or(false)
        })
        .filter_map(|event| event.get_object_id("_id").ok().map(|id| id.to_hex()))
        .collect();
    payment_slip::validate(&bank_transfer_slips, &selected_images)?;
    if !bank_transfer_slips.is_empty() {
        // A previous order's slip cannot automatically mark a new draft as paid.
        let mut used_filter = state.scope();
        used_filter.extend(doc! {"inquiry_id":id,"draft_id":{"$ne":draft_id},"status":{"$ne":"NOT_CREATED"},"payment_slip.slip":{"$exists":true}});
        let previous: Vec<Document> = state
            .collection("alex_submissions")
            .find(used_filter)
            .projection(doc! {"payment_slip":1})
            .await?
            .try_collect()
            .await?;
        let used: Vec<BankTransferSlip> = previous
            .iter()
            .filter_map(|record| {
                record
                    .get_document("payment_slip")
                    .ok()?
                    .get_document("slip")
                    .ok()
            })
            .filter_map(|slip| bson::from_document(slip.clone()).ok())
            .collect();
        bank_transfer_slips.retain(|slip| {
            !used.iter().any(|previous| {
                previous.source_ref == slip.source_ref
                    || previous.fingerprint() == slip.fingerprint()
            })
        });
    }
    let records = cart::items(state, id).await?;
    let mut missing = Vec::new();
    if records.is_empty() {
        missing.push(json!({"field":"cart"}));
    }
    let rental = &fields.rental;
    for (field, present) in [
        ("pickup_date", rental.pickup_date.is_some()),
        ("return_date", rental.return_date.is_some()),
        ("fulfillment_method", rental.fulfillment_method.is_some()),
    ] {
        if !present {
            missing.push(json!({"field":field}));
        }
    }
    let CartLines {
        mut lines,
        unpriced,
        branches,
    } = cart_lines(&records)?;
    // Policy-owned fee lines are regenerated from the exact current catalog item, never duplicated from the cart.
    if let Some(rule) = policy
        .rules
        .shipping_product
        .as_ref()
        .filter(|_| !policy.rules.manual_shipping)
    {
        lines.retain(|line| !line.code.eq_ignore_ascii_case(&rule.product_code));
        if lines.is_empty() && !missing.iter().any(|item| item["field"] == "cart") {
            missing.push(json!({"field":"cart"}));
        }
    }
    let cart_complete = !lines.is_empty() && unpriced.is_empty() && branches.len() == 1;
    missing.extend(
        unpriced
            .into_iter()
            .map(|code| json!({"field":"catalog_price","code":code})),
    );
    if branches.len() > 1 {
        missing.push(json!({"field":"mixed_branches"}));
    }
    let catalog_rental: i64 = lines
        .iter()
        .filter(|line| !line.is_shipping)
        .map(|line| line.rental_fee_satang * line.quantity)
        .sum();
    let catalog_deposit: i64 = lines
        .iter()
        .map(|line| line.deposit_satang * line.quantity)
        .sum();
    let summary = policy.rules.summary_amounts;
    let summary_rental = summary && fields.amounts.rental_fee_satang.is_some();
    // Missing quotes use verified catalog prices. An explicit quote still takes priority,
    // while an admin discount can be applied to a catalog default without rerunning AI.
    if summary && cart_complete {
        fields
            .amounts
            .rental_fee_satang
            .get_or_insert(catalog_rental);
        fields.amounts.deposit_satang.get_or_insert(catalog_deposit);
    }
    if summary {
        missing.extend(summary_issues(
            &fields.amounts,
            catalog_rental,
            catalog_deposit,
            cart_complete,
        ));
    }
    // Saved explicit discounts, including zero, outrank quoted totals and shop defaults.
    let written_rental_discount = fields.amounts.rental_discount_satang;
    let written_deposit_discount = fields.amounts.deposit_discount_satang;
    let mut groups = Vec::new();
    if let (true, Some(branch), Some(pickup), Some(returned), Some(method)) = (
        missing.is_empty(),
        branches.into_iter().next(),
        rental.pickup_date.clone(),
        rental.return_date.clone(),
        rental.fulfillment_method.clone(),
    ) {
        if !valid_date(&pickup) || !valid_date(&returned) || returned < pickup {
            return Err(AppError::Validation("Invalid rental dates"));
        }
        let key = drafts::hash(&json!([
            branch,
            pickup,
            returned,
            method,
            rental.delivery_provider
        ]))?;
        groups.push(Group {
            key,
            branch_id: branch,
            pickup_date: pickup,
            return_date: returned,
            fulfillment_method: method,
            delivery_provider: rental.delivery_provider.clone(),
            lines,
            notes: String::new(),
            quoted_rental_fee_satang: fields.amounts.rental_fee_satang,
            quoted_deposit_satang: fields.amounts.deposit_satang,
            quoted_shipping_fee_satang: Some(0),
            rental_discount_satang: discount(
                written_rental_discount,
                fields.amounts.rental_fee_satang,
                catalog_rental,
            ),
            deposit_discount_satang: discount(
                written_deposit_discount,
                fields.amounts.deposit_satang,
                catalog_deposit,
            ),
            amounts_from_summary: summary_rental && written_rental_discount.is_none(),
            discounts_finalized: written_rental_discount.is_some()
                || written_deposit_discount.is_some(),
        });
        if let Some(group) = groups.last_mut() {
            super::shop_settings::apply_shipping(state, &policy, group).await?;
            if group.lines.iter().any(|line| line.is_shipping) {
                group.quoted_shipping_fee_satang = Some(0);
            }
        }
    }
    let has_conflicts = draft
        .get_document("result")
        .ok()
        .and_then(|r| r.get_array("conflicts").ok())
        .is_some_and(|c| !c.is_empty());
    let versions = drafts::versions(state, &inquiry, Some(version_keys())).await?;
    let closed = closed(&draft, &version_bills(state, id, &versions).await?);
    let source_hash = drafts::hash(
        &json!({"draft":draft_id.to_hex(),"context":snapshot.hash,"cart":records,
        "billing_settings_version":policy.version,"billing_rules":policy.rules,"groups":groups,
        "remark_policy":"manual_draft_notes_then_saved_form_v2","pricing_policy":"final_admin_draft_discounts_v2","bank_transfer_slips":bank_transfer_slips,"payment_tolerance_satang":payment_slip::TOLERANCE_SATANG}),
    )?;
    Ok(Source {
        inquiry,
        draft,
        fields,
        source_hash,
        groups,
        missing,
        has_conflicts,
        closed,
        policy,
        bank_transfer_slips,
    })
}

// A fee quoted to the customer below the Alex catalog total is the discount the admin gave
// (quoted rent 500 on a 700 catalog price is a 200 discount). A discount written in the draft
// always wins, and a quote above the catalog never becomes a negative discount.
fn discount(explicit: Option<i64>, quoted: Option<i64>, catalog: i64) -> i64 {
    explicit.unwrap_or_else(|| quoted.map_or(0, |quoted| (catalog - quoted).max(0)))
}

// What stops a bill when the shop bills rent and deposit as the summary states them: a total the
// summary lacks is never filled from the cart, and Alex can only discount, so a total above the
// catalog cannot be billed as written. Catalog comparisons wait until the cart is complete.
pub(crate) fn summary_issues(
    amounts: &Amounts,
    catalog_rental: i64,
    catalog_deposit: i64,
    cart_complete: bool,
) -> Vec<Value> {
    [
        ("rental", amounts.rental_fee_satang, catalog_rental),
        ("deposit", amounts.deposit_satang, catalog_deposit),
    ]
    .into_iter()
    .filter_map(|(name, quoted, catalog)| match quoted {
        _ if (name == "rental" && amounts.rental_discount_satang.is_some())
            || (name == "deposit" && amounts.deposit_discount_satang.is_some()) =>
        {
            None
        }
        None => Some(json!({"field":format!("summary_{name}")})),
        Some(quoted) if cart_complete && quoted > catalog => {
            Some(json!({"field":format!("summary_{name}_above_catalog")}))
        }
        Some(_) => None,
    })
    .collect()
}

// Bill lines from cart records, the codes that lack a catalog price, and the branches in use.
struct CartLines {
    lines: Vec<Line>,
    unpriced: Vec<String>,
    branches: BTreeSet<String>,
}

fn cart_lines(records: &[Document]) -> Result<CartLines, AppError> {
    let mut lines = Vec::new();
    let mut unpriced = Vec::new();
    let mut branches = BTreeSet::new();
    for (index, record) in records.iter().enumerate() {
        let candidate = cart::candidate(record)?;
        branches.insert(
            record
                .get_str("branch_id")
                .map_err(|_| AppError::Data)?
                .to_string(),
        );
        let (Some(rental_fee), Some(deposit)) =
            (candidate.rental_fee_satang, candidate.deposit_satang)
        else {
            unpriced.push(candidate.code);
            continue;
        };
        lines.push(Line {
            item_index: index,
            product_item_id: candidate.product_item_id,
            code: candidate.code,
            name: candidate.name,
            color: candidate.color,
            size: candidate.size,
            quantity: record.get_i64("quantity").unwrap_or(1),
            rental_fee_satang: rental_fee,
            deposit_satang: deposit,
            is_shipping: record.get_bool("is_shipping").unwrap_or(false),
            image_url: candidate.image_url,
        });
    }
    Ok(CartLines {
        lines,
        unpriced,
        branches,
    })
}

fn choose(source: &Source, key: &str) -> Result<Group, AppError> {
    let group = source
        .groups
        .iter()
        .find(|group| group.key == key)
        .cloned()
        .ok_or(AppError::BillNotReady)?;
    if source.closed.is_some() {
        return Err(AppError::DraftClosed);
    }
    if source.has_conflicts || !source.missing.is_empty() {
        return Err(AppError::BillNotReady);
    }
    Ok(group)
}

fn scoped_id(state: &AppState, inquiry: ObjectId, id: ObjectId) -> Document {
    let mut filter = state.scope();
    filter.extend(doc! {"_id":id,"inquiry_id":inquiry});
    filter
}

async fn adjusted_group(
    state: &AppState,
    source: &Source,
    key: &str,
    rental_discount: Option<i64>,
    deposit_discount: Option<i64>,
) -> Result<Group, AppError> {
    let mut group = choose(source, key)?;
    if rental_discount.is_none() && deposit_discount.is_none() {
        return Ok(group);
    }
    group.discounts_finalized = true;
    // Discounts apply to the rented products. Regenerate the policy-owned shipping item
    // afterwards, using the edited net rent, so a fee is neither discounted nor duplicated.
    if let Some(rule) = source
        .policy
        .rules
        .shipping_product
        .as_ref()
        .filter(|_| !source.policy.rules.manual_shipping)
    {
        group
            .lines
            .retain(|line| !line.code.eq_ignore_ascii_case(&rule.product_code));
        group.notes.clear();
    }
    let rental: i64 = group
        .lines
        .iter()
        .filter(|line| !line.is_shipping)
        .map(|line| line.rental_fee_satang * line.quantity)
        .sum();
    let deposit: i64 = group
        .lines
        .iter()
        .map(|line| line.deposit_satang * line.quantity)
        .sum();
    group.rental_discount_satang = rental_discount.unwrap_or(group.rental_discount_satang);
    group.deposit_discount_satang = deposit_discount.unwrap_or(group.deposit_discount_satang);
    if !(0..=rental).contains(&group.rental_discount_satang)
        || !(0..=deposit).contains(&group.deposit_discount_satang)
    {
        return Err(AppError::Validation(
            "Discounts must be between zero and the catalog totals",
        ));
    }
    group.quoted_rental_fee_satang = Some(rental - group.rental_discount_satang);
    group.quoted_deposit_satang = Some(deposit - group.deposit_discount_satang);
    super::shop_settings::apply_shipping(state, &source.policy, &mut group).await?;
    if group.lines.iter().any(|line| line.is_shipping) {
        group.quoted_shipping_fee_satang = Some(0);
    }
    Ok(group)
}

// Preview only: no bill, reservation or order is written until the admin presses send.
pub async fn amounts(state: &AppState, id: ObjectId, input: CheckInput) -> Result<Value, AppError> {
    let source = source(state, id).await?;
    if input.source_hash != source.source_hash {
        return Err(AppError::ContextChanged);
    }
    let group = adjusted_group(
        state,
        &source,
        &input.group_key,
        input.rental_discount_satang,
        input.deposit_discount_satang,
    )
    .await?;
    Ok(
        json!({"group":group,"amounts_hash":drafts::hash(&json!(group))?,"source_hash":source.source_hash}),
    )
}

fn bill_snapshot(record: &Document, version: Option<i64>) -> Result<Value, AppError> {
    let group: Group = bson::from_document(
        record
            .get_document("group")
            .map_err(|_| AppError::Data)?
            .clone(),
    )
    .map_err(|_| AppError::Data)?;
    let input = record
        .get_document("review_input")
        .or_else(|_| record.get_document("input"))
        .ok();
    let detail = record
        .get_document("payload")
        .ok()
        .and_then(|payload| payload.get_document("orderDetail").ok());
    let text = |key: &str, legacy: &str| {
        input
            .and_then(|input| input.get_str(key).ok())
            .or_else(|| detail.and_then(|detail| detail.get_str(legacy).ok()))
            .unwrap_or("")
    };
    let rental: i64 = group
        .lines
        .iter()
        .map(|line| line.rental_fee_satang * line.quantity)
        .sum();
    let deposit: i64 = group
        .lines
        .iter()
        .map(|line| line.deposit_satang * line.quantity)
        .sum();
    let status = record.get_str("status").unwrap_or("PREPARED");
    Ok(
        json!({"id":record.get_object_id("_id").map_err(|_|AppError::Data)?.to_hex(),"draft_id":record.get_object_id("draft_id").ok().map(|id|id.to_hex()),"draft_version":version.or_else(||record.get_i64("draft_version").ok()),"status":status,"group":group,
        "customer":{"name":text("customer_name","customerName"),"phone":text("customer_phone","customerPhone"),"address":text("customer_address","customerAddress")},
        "receiving_method":text("receiving_method","receivingMethod"),"rental_payment_method":text("rental_payment_method","rentalPaymentMethod"),"bail_payment_method":text("bail_payment_method","bailPaymentMethod"),
        "notes":detail.and_then(|detail|detail.get_str("remark1").ok()).unwrap_or_else(||text("notes","remark1")),
        "payment_slip":record.get("payment_slip"),"catalog_rental_satang":rental,"catalog_deposit_satang":deposit,"rental_discount_satang":group.rental_discount_satang,"deposit_discount_satang":group.deposit_discount_satang,"rental_fee_satang":rental-group.rental_discount_satang,"deposit_satang":deposit-group.deposit_discount_satang,
        "use_free_item_promotion":input.and_then(|input|input.get_bool("use_free_item_promotion").ok()).unwrap_or(false),"use_discount_promotion":input.and_then(|input|input.get_bool("use_discount_promotion").ok()).unwrap_or(false),
        "alex_order_id":record.get_str("alex_order_id").ok(),"alex_order_number":record.get_str("alex_order_number").ok(),"updated_by":record.get_str("updated_by").or_else(|_|record.get_str("submitted_by")).ok(),"updated_at":record.get_datetime("updated_at").or_else(|_|record.get_datetime("created_at")).ok()}),
    )
}

fn saved_form_view(record: &Document) -> Result<Value, AppError> {
    Ok(
        json!({"revision":record.get_i64("revision").map_err(|_|AppError::Data)?,"source_hash":record.get_str("source_hash").map_err(|_|AppError::Data)?,"input":bson::from_bson::<Value>(record.get("input").cloned().ok_or(AppError::Data)?).map_err(|_|AppError::Data)?,"bill":bill_snapshot(record,None)?}),
    )
}

fn manual_draft_note<'a>(draft: &Document, note: Option<&'a str>) -> Option<&'a str> {
    let reference = draft.get_object_id("_id").ok()?.to_hex();
    let evidence = draft
        .get_document("result")
        .ok()?
        .get_array("evidence")
        .ok()?;
    evidence
        .iter()
        .filter_map(|entry| entry.as_document())
        .any(|entry| {
            entry.get_str("field").ok() == Some("notes")
                && entry.get_str("source_type").ok() == Some("admin_edit")
                && entry.get_str("source_ref").ok() == Some(reference.as_str())
        })
        .then_some(note)
        .flatten()
}

fn matched_payment(
    source: &Source,
    group: &Group,
    input: &ReviewInput,
) -> Result<Option<Value>, AppError> {
    let Some(reference) = &input.payment_slip_source_ref else {
        return Ok(None);
    };
    let rental: i64 = group
        .lines
        .iter()
        .map(|line| line.rental_fee_satang * line.quantity)
        .sum();
    let deposit: i64 = group
        .lines
        .iter()
        .map(|line| line.deposit_satang * line.quantity)
        .sum();
    let rental_net = rental - group.rental_discount_satang;
    let deposit_net = deposit - group.deposit_discount_satang;
    let slip = source
        .bank_transfer_slips
        .iter()
        .find(|slip| &slip.source_ref == reference && slip.matches(rental_net, deposit_net))
        .ok_or(AppError::Validation(
            "The bank slip no longer matches the final net rental and deposit",
        ))?;
    if input.payment_methods_edited
        || input.rental_payment_method != BANK_TRANSFER
        || input.bail_payment_method != BANK_TRANSFER
    {
        return Err(AppError::Validation(
            "A matched bank slip must apply bank transfer to both payments",
        ));
    }
    Ok(Some(
        json!({"slip":slip,"net_satang":rental_net+deposit_net,"tolerance_satang":payment_slip::TOLERANCE_SATANG}),
    ))
}

pub async fn save_form(
    state: &AppState,
    id: ObjectId,
    mut input: SaveFormInput,
    actor: &Actor,
) -> Result<Value, AppError> {
    if !(0..i64::MAX).contains(&input.expected_revision) {
        return Err(AppError::BadRequest);
    }
    let source = source(state, id).await?;
    if source.source_hash != input.form.source_hash {
        return Err(AppError::ContextChanged);
    }
    input.form.shipping_fee_satang = 0;
    if input.form.rental_discount_satang.is_some() || input.form.deposit_discount_satang.is_some() {
        input.form.use_discount_promotion = false;
    }
    if input.form.customer_name.chars().count() > 200
        || input.form.customer_phone.chars().count() > 40
        || input.form.customer_address.chars().count() > 1000
        || input.form.notes.chars().count() > 2000
        || !RECEIVING_METHODS.contains(&input.form.receiving_method.as_str())
        || [
            &input.form.rental_payment_method,
            &input.form.bail_payment_method,
        ]
        .iter()
        .any(|method| !PAYMENT_METHODS.contains(&method.as_str()))
    {
        return Err(AppError::Validation("Check the saved bill form fields"));
    }
    let group = adjusted_group(
        state,
        &source,
        &input.form.group_key,
        input.form.rental_discount_satang,
        input.form.deposit_discount_satang,
    )
    .await?;
    let payment_slip = matched_payment(&source, &group, &input.form)?;
    if group.discounts_finalized {
        input.form.use_discount_promotion = false;
    }
    let draft_id = source
        .draft
        .get_object_id("_id")
        .map_err(|_| AppError::Data)?;
    let mut filter = state.scope();
    filter.extend(doc! {"inquiry_id":id,"draft_id":draft_id,"group.key":&group.key});
    let form = bson::to_document(&input.form).map_err(|_| AppError::Data)?;
    let existing = state
        .collection("checkout_forms")
        .find_one(filter.clone())
        .await?;
    if let Some(record) = &existing {
        check_record_branch(state, record)?;
        if record.get_document("input").ok() == Some(&form) {
            return saved_form_view(record);
        }
        if record.get_i64("revision").ok() != Some(input.expected_revision) {
            return Err(AppError::RecordConflict);
        }
    } else if input.expected_revision != 0 {
        return Err(AppError::RecordConflict);
    }
    if self::source(state, id).await?.source_hash != source.source_hash {
        return Err(AppError::ContextChanged);
    }
    super::team::revalidate(state).await?;
    let now = DateTime::now();
    let mut fields = doc! {"input":form,"group":bson::to_document(&group).map_err(|_|AppError::Data)?,"source_hash":&source.source_hash,"draft_version":source.draft.get_i64("draft_version").map_err(|_|AppError::Data)?,"updated_by":&actor.name,"updated_by_id":&actor.id,"updated_at":now};
    fields.insert(
        "payment_slip",
        bson::to_bson(&payment_slip).map_err(|_| AppError::Data)?,
    );
    let record = if existing.is_some() {
        filter.insert("revision", input.expected_revision);
        state
            .collection("checkout_forms")
            .find_one_and_update(filter, doc! {"$set":fields,"$inc":{"revision":1_i64}})
            .return_document(mongodb::options::ReturnDocument::After)
            .await?
            .ok_or(AppError::RecordConflict)?
    } else {
        let mut record = filter;
        record.remove("group.key");
        record.extend(fields);
        record.extend(doc! {"_id":ObjectId::new(),"created_at":now,"revision":1_i64});
        if let Err(error) = state
            .collection("checkout_forms")
            .insert_one(record.clone())
            .await
        {
            return Err(if super::events::duplicate(&error) {
                AppError::RecordConflict
            } else {
                error.into()
            });
        }
        record
    };
    saved_form_view(&record)
}

fn safe_submission(mut record: Document) -> Result<Value, AppError> {
    // Keep the reference contract explicit, including records written by older releases.
    for key in ["alex_order_id", "alex_order_number"] {
        if !record.contains_key(key) {
            record.insert(key, bson::Bson::Null);
        }
    }
    if let Ok(summary) = bill_snapshot(&record, None) {
        record.insert(
            "bill_summary",
            bson::to_bson(&summary).map_err(|_| AppError::Data)?,
        );
    }
    for key in [
        "payload",
        "alex_response",
        "submitted_by_id",
        "source_hash",
        "review_input",
    ] {
        record.remove(key);
    }
    serde_json::to_value(record).map_err(|_| AppError::Data)
}

fn check_record_branch(state: &AppState, record: &Document) -> Result<(), AppError> {
    if state.access.is_some() {
        let branch = record
            .get_document("group")
            .and_then(|group| group.get_str("branch_id"))
            .map_err(|_| AppError::Data)?;
        super::team::check_branch(state, branch)?;
    }
    Ok(())
}

pub async fn list(state: &AppState, id: ObjectId) -> Result<Value, AppError> {
    let inquiry = drafts::inquiry(state, id).await?;
    // Only a send in progress holds the lock; an unclear Alex reply no longer blocks the next version.
    if let Ok(submission) = inquiry.get_object_id("alex_submission_id")
        && let Some(record) = recover(state, id, submission).await?
        && ["CONFIRMED", "NOT_CREATED", "SUBMIT_UNKNOWN"]
            .contains(&record.get_str("status").unwrap_or(""))
    {
        unlock(
            state,
            id,
            record.get_object_id("_id").map_err(|_| AppError::Data)?,
        )
        .await?;
    }
    let mut filter = state.scope();
    filter.insert("inquiry_id", id);
    if let Some(access) = &state.access
        && access.role != crate::models::team::Role::Owner
    {
        filter.insert("group.branch_id", doc! {"$in": &access.branch_ids});
    }
    let forms: Vec<Document> = state
        .collection("checkout_forms")
        .find(filter.clone())
        .sort(doc! {"draft_version":-1,"updated_at":-1})
        .limit(50)
        .await?
        .try_collect()
        .await?;
    let submissions: Vec<Document> = state
        .collection("alex_submissions")
        .find(filter)
        .sort(doc! {"created_at":-1})
        .limit(50)
        .await?
        .try_collect()
        .await?;
    let versions = drafts::versions(state, &inquiry, Some(version_keys())).await?;
    let last_submission = submissions.first();
    let last_form = forms.first();
    let show_form = last_form.is_some_and(|form| {
        last_submission.is_none_or(|bill| {
            let bill_version = bill_version(bill, &versions)
                .map(|(_, number)| number)
                .unwrap_or(0);
            form.get_i64("draft_version").unwrap_or(0) > bill_version
                || (form.get_i64("draft_version").ok() == Some(bill_version)
                    && bill.get_str("status").ok() == Some("NOT_CREATED")
                    && form.get_datetime("updated_at").ok() > bill.get_datetime("created_at").ok())
        })
    });
    let latest_bill = if show_form {
        last_form.and_then(|form| bill_snapshot(form, None).ok())
    } else {
        last_submission.and_then(|bill| {
            bill_snapshot(
                bill,
                bill_version(bill, &versions).map(|(_, number)| number),
            )
            .ok()
        })
    };
    let submissions = submissions
        .into_iter()
        .map(|record| {
            let version = bill_version(&record, &versions).map(|(_, number)| number);
            let mut value = safe_submission(record)?;
            value["draft_version"] = json!(version);
            Ok(value)
        })
        .collect::<Result<Vec<_>, AppError>>()?;
    let source = match source(state, id).await {
        Ok(source) => source,
        Err(error) => {
            return Ok(
                json!({"source_hash":Value::Null,"draft_version":drafts::version(&inquiry),
        "groups":[],"missing":[],"has_conflicts":true,"submissions":submissions,"customer":{"name":Value::Null,"phone":Value::Null,"address":Value::Null},
        "notes":Value::Null,"context_error":error.code(),"latest_bill":latest_bill,"saved_form":Value::Null,"order_response_configured":state.config.alex.as_ref().is_some_and(|cfg|cfg.order_id_pointer.is_some())}),
            );
        }
    };
    let saved_form = forms
        .iter()
        .find(|form| {
            form.get_object_id("draft_id").ok() == source.draft.get_object_id("_id").ok()
                && source.groups.iter().any(|group| {
                    form.get_document("group")
                        .ok()
                        .and_then(|group| group.get_str("key").ok())
                        == Some(group.key.as_str())
                })
        })
        .map(saved_form_view)
        .transpose()?;
    let notes = manual_draft_note(&source.draft, source.fields.notes.as_deref()).map(str::to_owned);
    Ok(
        json!({"source_hash":source.source_hash,"draft_version":drafts::version(&source.inquiry),"closed":source.closed,"groups":source.groups,"missing":source.missing,
        "has_conflicts":source.has_conflicts,"submissions":submissions,"customer":{"name":source.fields.customer_name,"phone":source.fields.customer_phone,"address":source.fields.customer_address},
        // Prefill explicitly saved admin notes; saved modal input, including a cleared note, wins.
        "bank_transfer_slips":source.bank_transfer_slips,"payment_tolerance_satang":payment_slip::TOLERANCE_SATANG,"notes":notes,"latest_bill":latest_bill,"saved_form":saved_form,"order_response_configured":state.config.alex.as_ref().is_some_and(|cfg|cfg.order_id_pointer.is_some())}),
    )
}

async fn check(
    state: &AppState,
    session: &alex::Session,
    group: &Group,
) -> Result<Vec<Value>, AppError> {
    let mut results = Vec::new();
    for line in &group.lines {
        let latest = alex::lookup(
            state,
            session,
            &group.branch_id,
            &line.code,
            &line.product_item_id,
        )
        .await?;
        if latest.name != line.name
            || latest.color != line.color
            || latest.size != line.size
            || latest.rental_fee_satang != Some(line.rental_fee_satang)
            || latest.deposit_satang != Some(line.deposit_satang)
        {
            return Err(AppError::ProductConflict);
        }
        let state = alex::check_queue(
            state,
            session,
            &group.branch_id,
            &line.product_item_id,
            &group.pickup_date,
            &group.return_date,
            line.quantity,
        )
        .await
        .unwrap_or_else(|_| "UNKNOWN".into());
        results.push(json!({"item_index":line.item_index,"product_item_id":line.product_item_id,"code":line.code,"quantity":line.quantity,"state":state}));
    }
    Ok(results)
}

pub async fn availability(
    state: &AppState,
    id: ObjectId,
    input: CheckInput,
) -> Result<Value, AppError> {
    let source = source(state, id).await?;
    if input.source_hash != source.source_hash {
        return Err(AppError::ContextChanged);
    }
    let group = adjusted_group(
        state,
        &source,
        &input.group_key,
        input.rental_discount_satang,
        input.deposit_discount_satang,
    )
    .await?;
    let session = alex::login(state).await?;
    let results = check(state, &session, &group).await?;
    if self::source(state, id).await?.source_hash != source.source_hash {
        return Err(AppError::ContextChanged);
    }
    let status = queue_state(&results);
    let now = DateTime::now();
    let mut record = state.scope();
    record.extend(doc! {"_id":ObjectId::new(),"inquiry_id":id,"draft_id":source.draft.get_object_id("_id").map_err(|_|AppError::Data)?,
        "source_hash":source.source_hash,"group_key":group.key,"state":status,"items":bson::to_bson(&results).map_err(|_|AppError::Data)?,"created_at":now,"updated_at":now});
    state
        .collection("availability_checks")
        .insert_one(record)
        .await?;
    Ok(json!({"state":status,"items":results,"checked_at":now,"reserves_stock":false}))
}

// Queue check for the cart with the dates on screen; it reads Alex only and reserves nothing.
pub async fn cart_availability(
    state: &AppState,
    id: ObjectId,
    input: CartCheckInput,
) -> Result<Value, AppError> {
    if !valid_date(&input.pickup_date)
        || !valid_date(&input.return_date)
        || input.return_date < input.pickup_date
    {
        return Err(AppError::Validation("Invalid rental dates"));
    }
    drafts::inquiry(state, id).await?;
    let records = cart::items(state, id).await?;
    let CartLines {
        lines,
        unpriced,
        branches,
    } = cart_lines(&records)?;
    let branch = match (branches.len(), unpriced.is_empty()) {
        (1, true) => branches.into_iter().next().ok_or(AppError::Data)?,
        (0, _) => return Err(AppError::Validation("The cart is empty")),
        _ => {
            return Err(AppError::Validation(
                "Cart items need a catalog price and one branch",
            ));
        }
    };
    let group = Group {
        key: String::new(),
        branch_id: branch,
        pickup_date: input.pickup_date,
        return_date: input.return_date,
        fulfillment_method: String::new(),
        delivery_provider: None,
        lines,
        notes: String::new(),
        quoted_rental_fee_satang: None,
        quoted_deposit_satang: None,
        quoted_shipping_fee_satang: None,
        rental_discount_satang: 0,
        deposit_discount_satang: 0,
        amounts_from_summary: false,
        discounts_finalized: false,
    };
    let session = alex::login(state).await?;
    let results = check(state, &session, &group).await?;
    Ok(
        json!({"state":queue_state(&results),"items":results,"checked_at":DateTime::now(),"reserves_stock":false}),
    )
}

fn baht(value: i64) -> String {
    format!("{}.{:02}", value / 100, value % 100)
}

fn validate_input(input: &ReviewInput, group: &Group) -> Result<(), AppError> {
    ProfileInput {
        display_name: Some(input.customer_name.clone()),
        phone: Some(input.customer_phone.clone()),
        address: Some(input.customer_address.clone()),
        ..Default::default()
    }
    .validate()?;
    if input.customer_name.trim().is_empty()
        || input.customer_phone.trim().is_empty()
        || (group.fulfillment_method == "delivery" && input.customer_address.trim().is_empty())
        || !(0..=10_000_000_000).contains(&input.shipping_fee_satang)
        || !RECEIVING_METHODS.contains(&input.receiving_method.as_str())
        || [&input.rental_payment_method, &input.bail_payment_method]
            .iter()
            .any(|value| !PAYMENT_METHODS.contains(&value.as_str()))
        || input.notes.chars().count() > 2000
        || input
            .birth_date
            .as_ref()
            .is_some_and(|date| !valid_date(date))
    {
        return Err(AppError::Validation(
            "Review customer, prices, payment and shipping fields",
        ));
    }
    Ok(())
}

pub async fn review(
    state: &AppState,
    id: ObjectId,
    mut input: ReviewInput,
) -> Result<Value, AppError> {
    input.customer_phone = normalize_phone(&input.customer_phone);
    for method in [
        &mut input.rental_payment_method,
        &mut input.bail_payment_method,
    ] {
        if method.trim().is_empty() {
            *method = UNPAID.into();
        }
    }
    let source = source(state, id).await?;
    if source.source_hash != input.source_hash {
        return Err(AppError::ContextChanged);
    }
    input.customer_name = source.policy.rules.customer_name(&input.customer_name);
    // Shipping is a verified catalog item in this order, never an extra browser-supplied charge.
    input.shipping_fee_satang = 0;
    let discounts_finalized = input.rental_discount_satang.is_some()
        || input.deposit_discount_satang.is_some()
        || source.fields.amounts.rental_discount_satang.is_some()
        || source.fields.amounts.deposit_discount_satang.is_some();
    if discounts_finalized {
        // The final admin amounts replace defaults and cannot stack an automatic Alex discount.
        input.use_discount_promotion = false;
    }
    let group = adjusted_group(
        state,
        &source,
        &input.group_key,
        input.rental_discount_satang,
        input.deposit_discount_satang,
    )
    .await?;
    if let Some(expected) = &input.amounts_hash
        && *expected != drafts::hash(&json!(group))?
    {
        return Err(AppError::ContextChanged);
    }
    if source.policy.rules.shipping_product.is_some() {
        let original = choose(&source, &input.group_key)?;
        input.notes = input
            .notes
            .lines()
            .filter(|line| *line != original.notes && *line != group.notes)
            .collect::<Vec<_>>()
            .join("\n");
        if !group.notes.is_empty() {
            if !input.notes.trim().is_empty() {
                input.notes.push('\n');
            }
            input.notes.push_str(&group.notes);
        }
    }
    validate_input(&input, &group)?;
    let session = alex::login(state).await?;
    // A booked or unreadable queue does not stop the review; the admin confirms it before sending.
    let results = check(state, &session, &group).await?;
    let queue = json!({"state":queue_state(&results),"items":results});
    let rental: i64 = group
        .lines
        .iter()
        .map(|line| line.rental_fee_satang * line.quantity)
        .sum();
    let deposit: i64 = group
        .lines
        .iter()
        .map(|line| line.deposit_satang * line.quantity)
        .sum();
    let discountable: i64 = group
        .lines
        .iter()
        .filter(|line| !line.is_shipping)
        .map(|line| line.rental_fee_satang * line.quantity)
        .sum();
    if !(0..=discountable).contains(&group.rental_discount_satang)
        || !(0..=deposit).contains(&group.deposit_discount_satang)
    {
        return Err(AppError::Validation(
            "Discounts cannot exceed the catalog rental or deposit totals",
        ));
    }
    let rental_fee = rental - group.rental_discount_satang;
    let deposit_fee = deposit - group.deposit_discount_satang;
    let notes = input.notes.trim();
    let mut detail = json!({"branchId":group.branch_id,"customerId":"","customerName":input.customer_name,"customerAddress":input.customer_address,"customerPhone":input.customer_phone,
        "lineId":"","howToKnowShop":"","accountNumber":"","accountName":"","bankName":"","startDate":group.pickup_date,"endDate":group.return_date,
        "receivingMethod":input.receiving_method,"remark1":notes,"rentalPaymentMethod":input.rental_payment_method,"bailPaymentMethod":input.bail_payment_method,
        "pointDiscount":0,"voucherDiscount":0,"rentalDiscount":baht(group.rental_discount_satang),"bailDiscount":baht(group.deposit_discount_satang),
        "useFreeItemPromotion":input.use_free_item_promotion,"useDiscountPromotion":input.use_discount_promotion});
    if let Some(birth) = &input.birth_date {
        detail["birthDate"] = json!(birth);
    }
    let payload = json!({"orderDetail":detail,"orderItems":group.lines.iter().map(|line|json!({"productItemId":line.product_item_id,"reservedQuantity":line.quantity})).collect::<Vec<_>>()});
    let payment_slip = matched_payment(&source, &group, &input)?;
    if self::source(state, id).await?.source_hash != source.source_hash {
        return Err(AppError::ContextChanged);
    }
    let now = DateTime::now();
    let review_id = ObjectId::new();
    let mut reservations: Vec<_> = group
        .lines
        .iter()
        .map(|line| (line.product_item_id.clone(), line.quantity))
        .collect();
    reservations.sort();
    let reservation_key = drafts::hash(&json!([
        group.branch_id,
        group.pickup_date,
        group.return_date,
        reservations
    ]))?;
    let mut record = state.scope();
    record.extend(doc! {"_id":review_id,"inquiry_id":id,"source_hash":&source.source_hash,"group":bson::to_bson(&group).map_err(|_|AppError::Data)?,
        "input":bson::to_document(&input).map_err(|_|AppError::Data)?,"payload":bson::to_bson(&payload).map_err(|_|AppError::Data)?,"reservation_key":reservation_key,
        "queue":bson::to_bson(&queue).map_err(|_|AppError::Data)?,
        "created_at":now,"updated_at":now,"expires_at":DateTime::from_millis(now.timestamp_millis()+300_000)});
    record.insert("billing_settings_version", source.policy.version);
    record.insert("discounts_finalized", discounts_finalized);
    record.insert(
        "payment_slip",
        bson::to_bson(&payment_slip).map_err(|_| AppError::Data)?,
    );
    record.insert(
        "billing_rules",
        bson::to_bson(&source.policy.rules).map_err(|_| AppError::Data)?,
    );
    state.collection("bill_reviews").insert_one(&record).await?;
    Ok(
        json!({"review_id":review_id.to_hex(),"group":group,"queue":queue,"customer":{"name":input.customer_name,"phone":input.customer_phone,"address":input.customer_address},
        "payment_slip":record.get("payment_slip"),"catalog_rental_satang":rental,"catalog_deposit_satang":deposit,"rental_fee_satang":rental_fee,"deposit_satang":deposit_fee,"shipping_fee_satang":input.shipping_fee_satang,
        "rental_discount_satang":group.rental_discount_satang,"deposit_discount_satang":group.deposit_discount_satang,"discounts_finalized":discounts_finalized,
        "receiving_method":input.receiving_method,"rental_payment_method":input.rental_payment_method,"bail_payment_method":input.bail_payment_method,"notes":input.notes,"expires_at":record_expiry(now)}),
    )
}

fn record_expiry(now: DateTime) -> DateTime {
    DateTime::from_millis(now.timestamp_millis() + 300_000)
}

fn reference_value(value: &Value) -> Option<String> {
    let text = value.as_str().map(str::to_string).or_else(|| {
        value
            .as_u64()
            .filter(|value| *value > 0)
            .map(|value| value.to_string())
    })?;
    (!text.trim().is_empty()
        && text.trim() != "0"
        && text.len() <= 200
        && !text.chars().any(char::is_control))
    .then(|| text.trim().to_string())
}

fn reference_field(value: &Value, names: &[&str]) -> Option<String> {
    let values: BTreeSet<_> = names
        .iter()
        .filter_map(|name| value.get(*name).and_then(reference_value))
        .collect();
    (values.len() == 1)
        .then(|| values.into_iter().next())
        .flatten()
}

// Read explicit order fields only, never an unrelated customer/product ID or a guessed number.
// Without configured paths, both references must come from the same response object.
fn order_reference(
    status: u16,
    value: &Value,
    id_path: Option<&str>,
    number_path: Option<&str>,
) -> (Option<String>, Option<String>) {
    if !(200..300).contains(&status) || value.get("success") == Some(&json!(false)) {
        return (None, None);
    }
    let id_names = ["orderId", "OrderId", "orderID", "OrderID", "order_id"];
    let number_names = [
        "orderNumber",
        "OrderNumber",
        "orderNo",
        "OrderNo",
        "order_number",
    ];
    let (id, container) = if let Some(path) = id_path {
        let container = if path == "$" {
            value
        } else {
            value
                .pointer(path.rsplit_once('/').map_or("", |(parent, _)| parent))
                .unwrap_or(&Value::Null)
        };
        let id = if path == "$" {
            reference_value(value)
        } else {
            value.pointer(path).and_then(reference_value)
        };
        (id, container)
    } else {
        let candidates: Vec<_> = [
            "",
            "/data",
            "/result",
            "/order",
            "/orderDetail",
            "/data/order",
            "/data/orderDetail",
        ]
        .iter()
        .filter_map(|path| value.pointer(path))
        .filter(|object| object.get("success") != Some(&json!(false)))
        .filter(|object| {
            reference_field(object, &id_names).is_some()
                || reference_field(object, &number_names).is_some()
        })
        .collect();
        if candidates.len() != 1 {
            return (None, None);
        }
        (reference_field(candidates[0], &id_names), candidates[0])
    };
    if container.get("success") == Some(&json!(false)) {
        return (None, None);
    }
    let number = number_path.map_or_else(
        || reference_field(container, &number_names),
        |path| {
            if path == "$" {
                reference_value(value)
            } else {
                value.pointer(path).and_then(reference_value)
            }
        },
    );
    (id, number)
}

fn sanitize(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.retain(|key, _| {
                !["token", "password", "secret", "authorization"]
                    .iter()
                    .any(|name| key.to_lowercase().contains(name))
            });
            for value in map.values_mut() {
                sanitize(value);
            }
        }
        Value::Array(items) => {
            for value in items {
                sanitize(value);
            }
        }
        _ => (),
    }
}

async fn unlock(state: &AppState, id: ObjectId, submission: ObjectId) -> Result<(), AppError> {
    let mut filter = state.scope();
    filter.extend(doc! {"_id":id,"alex_submission_id":submission});
    state
        .collection("inquiries")
        .update_one(
            filter,
            doc! {"$unset":{"alex_submission_id":"","alex_submission_started_at":""}},
        )
        .await?;
    Ok(())
}

async fn recover(
    state: &AppState,
    id: ObjectId,
    submission: ObjectId,
) -> Result<Option<Document>, AppError> {
    let identity = scoped_id(state, id, submission);
    if let Some(record) = state
        .collection("alex_submissions")
        .find_one(identity.clone())
        .await?
    {
        check_record_branch(state, &record)?;
        return Ok(Some(record));
    }
    let inquiry = drafts::inquiry(state, id).await?;
    if inquiry.get_object_id("alex_submission_id").ok() != Some(submission) {
        return Ok(None);
    }
    let Some(review) = state
        .collection("bill_reviews")
        .find_one(identity.clone())
        .await?
    else {
        return Err(AppError::Data);
    };
    check_record_branch(state, &review)?;
    let now = DateTime::now();
    let started = inquiry
        .get_datetime("alex_submission_started_at")
        .copied()
        .unwrap_or(now);
    let mut record = identity.clone();
    record.extend(doc! {"reservation_key":review.get_str("reservation_key").map_err(|_|AppError::Data)?,"active":false,"status":"SUBMIT_UNKNOWN",
        "group":review.get("group").cloned().ok_or(AppError::Data)?,"submitted_by":"ระบบกู้สถานะหลังรีสตาร์ต","created_at":started,"updated_at":now});
    match state
        .collection("alex_submissions")
        .insert_one(record.clone())
        .await
    {
        Ok(_) => Ok(Some(record)),
        Err(error) if crate::db::events::duplicate(&error) => {
            if let Some(existing) = state
                .collection("alex_submissions")
                .find_one(identity)
                .await?
            {
                check_record_branch(state, &existing)?;
                return Ok(Some(existing));
            }
            let mut filter = state.scope();
            filter.extend(doc! {"inquiry_id":id,"reservation_key":review.get_str("reservation_key").map_err(|_|AppError::Data)?,"active":true});
            let existing = state
                .collection("alex_submissions")
                .find_one(filter)
                .await?
                .ok_or(AppError::Data)?;
            check_record_branch(state, &existing)?;
            let mut owner = state.scope();
            owner.extend(doc! {"_id":id,"alex_submission_id":submission});
            state.collection("inquiries").update_one(owner,doc! {"$set":{"alex_submission_id":existing.get_object_id("_id").map_err(|_|AppError::Data)?}}).await?;
            Ok(Some(existing))
        }
        Err(error) => Err(error.into()),
    }
}

pub async fn submit(
    state: &AppState,
    id: ObjectId,
    input: SubmitInput,
    actor: &Actor,
) -> Result<Value, AppError> {
    if !input.confirmed {
        return Err(AppError::BadRequest);
    }
    let review_id = ObjectId::parse_str(&input.review_id).map_err(|_| AppError::BadRequest)?;
    let identity = scoped_id(state, id, review_id);
    if let Some(existing) = state
        .collection("alex_submissions")
        .find_one(identity.clone())
        .await?
    {
        check_record_branch(state, &existing)?;
        return safe_submission(existing);
    }
    let mut filter = identity.clone();
    filter.insert("expires_at", doc! {"$gt":DateTime::now()});
    let review = state
        .collection("bill_reviews")
        .find_one(filter)
        .await?
        .ok_or(AppError::BillNotReady)?;
    check_record_branch(state, &review)?;
    let group: Group = bson::from_bson(review.get("group").cloned().ok_or(AppError::Data)?)
        .map_err(|_| AppError::Data)?;
    let source = source(state, id).await?;
    if review.get_str("source_hash").ok() != Some(&source.source_hash) {
        return Err(AppError::ContextChanged);
    }
    let mut lock = state.scope();
    lock.extend(doc! {"_id":id,"alex_submission_id":{"$exists":false}});
    if state
        .collection("inquiries")
        .update_one(lock, doc! {"$set":{"alex_submission_id":review_id,"alex_submission_started_at":DateTime::now()}})
        .await?
        .matched_count
        != 1
    {
        return Err(AppError::BillPending);
    }
    // Each version is sent once; bills of other versions do not block it (the queue check covers stock).
    // Checked under the lock so two tabs cannot both send the same version.
    let draft_id = source
        .draft
        .get_object_id("_id")
        .map_err(|_| AppError::Data)?;
    let mut sent = state.scope();
    sent.extend(doc! {"inquiry_id":id,"draft_id":draft_id,"status":{"$ne":"NOT_CREATED"}});
    if source.closed.is_some()
        || state
            .collection("alex_submissions")
            .find_one(sent)
            .await?
            .is_some()
    {
        unlock(state, id, review_id).await?;
        return Err(AppError::DraftClosed);
    }
    let now = DateTime::now();
    let mut record = identity.clone();
    record.insert("alex_order_id", bson::Bson::Null);
    record.insert("alex_order_number", bson::Bson::Null);
    // `active` marks a send in progress; it is cleared once Alex has answered.
    record.extend(doc! {"reservation_key":review.get_str("reservation_key").map_err(|_|AppError::Data)?,"active":true,
        "status":"SUBMITTING","group":review.get("group").cloned().ok_or(AppError::Data)?,"source_hash":&source.source_hash,
        "draft_id":draft_id,"draft_version":source.draft.get_i64("draft_version").map_err(|_|AppError::Data)?,
        "payload":review.get("payload").cloned().ok_or(AppError::Data)?,"submitted_by":&actor.name,"submitted_by_id":&actor.id,"created_at":now,"updated_at":now});
    record.insert("billing_settings_version", source.policy.version);
    record.insert(
        "billing_rules",
        bson::to_bson(&source.policy.rules).map_err(|_| AppError::Data)?,
    );
    record.insert(
        "review_input",
        review.get("input").cloned().ok_or(AppError::Data)?,
    );
    record.insert(
        "payment_slip",
        review
            .get("payment_slip")
            .cloned()
            .unwrap_or(bson::Bson::Null),
    );
    record.insert(
        "discounts_finalized",
        review.get_bool("discounts_finalized").unwrap_or(false),
    );
    record.insert(
        "line_user_id",
        source
            .inquiry
            .get("line_user_id")
            .cloned()
            .unwrap_or(bson::Bson::Null),
    );
    record.insert(
        "conversation_key",
        source
            .inquiry
            .get("conversation_key")
            .cloned()
            .unwrap_or(bson::Bson::Null),
    );
    let reviewed_input = review.get_document("input").map_err(|_| AppError::Data)?;
    record.insert("customer", doc! {"name":reviewed_input.get_str("customer_name").map_err(|_|AppError::Data)?,"phone":reviewed_input.get_str("customer_phone").map_err(|_|AppError::Data)?,"address":reviewed_input.get_str("customer_address").map_err(|_|AppError::Data)?});
    if let Err(error) = state
        .collection("alex_submissions")
        .insert_one(record.clone())
        .await
    {
        unlock(state, id, review_id).await?;
        return Err(if crate::db::events::duplicate(&error) {
            AppError::BillPending
        } else {
            error.into()
        });
    }
    let preflight = async {
        let session = alex::login(state).await?;
        let results = check(state, &session, &group).await?;
        let queue = queue_state(&results);
        if queue != "AVAILABLE" && !input.accept_queue {
            return Err(AppError::QueueBlocked);
        }
        if self::source(state, id).await?.source_hash != source.source_hash {
            return Err(AppError::ContextChanged);
        }
        super::team::revalidate(state).await?;
        Ok::<_, AppError>((session, json!({"state":queue,"items":results})))
    }
    .await;
    let (session, queue) = match preflight {
        Ok(checked) => checked,
        Err(error) => {
            let mut failed = identity.clone();
            failed.insert("status", "SUBMITTING");
            state.collection("alex_submissions").update_one(failed,doc! {"$set":{"status":"NOT_CREATED","active":false,"updated_at":DateTime::now()}}).await?;
            unlock(state, id, review_id).await?;
            return Err(error);
        }
    };
    let payload: Value = bson::from_bson(review.get("payload").cloned().ok_or(AppError::Data)?)
        .map_err(|_| AppError::Data)?;
    // A manual resolution during a long preflight must cancel the original sender.
    let mut sending = identity.clone();
    sending.extend(doc! {"status":"SUBMITTING","active":true});
    let sent_at = DateTime::now();
    // The queue as Alex reported it right before sending, and whether the admin accepted a booked one.
    let queue = bson::to_bson(&queue).map_err(|_| AppError::Data)?;
    if state
        .collection("alex_submissions")
        .update_one(
            sending,
            doc! {"$set":{"sent_at":sent_at,"queue":queue,"queue_accepted":input.accept_queue,"updated_at":sent_at}},
        )
        .await?
        .matched_count
        != 1
    {
        return Err(AppError::BillPending);
    }
    let outcome = alex::create_order(state, &session, &payload).await;
    let mut updates = doc! {"status":"SUBMIT_UNKNOWN","active":false,"updated_at":DateTime::now()};
    if let Ok((status, mut response)) = outcome {
        let config = state.config.alex.as_ref().ok_or(AppError::Data)?;
        let (order, number) = order_reference(
            status,
            &response,
            config.order_id_pointer.as_deref(),
            config.order_number_pointer.as_deref(),
        );
        if let Some(order) = order {
            updates.insert("status", "CONFIRMED");
            updates.insert("alex_order_id", order);
            updates.insert("verification", "api_response");
        }
        if let Some(number) = number {
            updates.insert("alex_order_number", number);
        }
        sanitize(&mut response);
        updates.insert("http_status", i32::from(status));
        updates.insert(
            "alex_response",
            bson::to_bson(&response).map_err(|_| AppError::Data)?,
        );
    }
    let mut finishing = identity.clone();
    finishing.insert("status", "SUBMITTING");
    state
        .collection("alex_submissions")
        .update_one(finishing, doc! {"$set":updates})
        .await?;
    // Alex has answered, so the next version may be sent; an unclear answer is recorded, never retried.
    unlock(state, id, review_id).await?;
    safe_submission(
        state
            .collection("alex_submissions")
            .find_one(identity)
            .await?
            .ok_or(AppError::Data)?,
    )
}

pub async fn resolve(
    state: &AppState,
    id: ObjectId,
    input: ResolveInput,
    actor: &Actor,
) -> Result<Value, AppError> {
    if !input.verified_in_alex || !["created", "not_created"].contains(&input.outcome.as_str()) {
        return Err(AppError::BadRequest);
    }
    let submission = ObjectId::parse_str(&input.submission_id).map_err(|_| AppError::BadRequest)?;
    let mut filter = scoped_id(state, id, submission);
    let record = recover(state, id, submission)
        .await?
        .ok_or(AppError::NotFound)?;
    if !["SUBMITTING", "SUBMIT_UNKNOWN"].contains(&record.get_str("status").unwrap_or("")) {
        return Err(AppError::BillPending);
    }
    if record
        .get_datetime("sent_at")
        .or_else(|_| record.get_datetime("created_at"))
        .is_ok_and(|created| {
            DateTime::now().timestamp_millis() - created.timestamp_millis() < 120_000
        })
        && input.outcome == "not_created"
    {
        return Err(AppError::BillPending);
    }
    let mut updates = doc! {"status":"NOT_CREATED","active":false,"verification":"manual","resolved_by":&actor.name,"updated_at":DateTime::now()};
    if input.outcome == "created" {
        let order = input
            .order_id
            .filter(|value| {
                !value.trim().is_empty()
                    && value.len() <= 200
                    && !value.chars().any(char::is_control)
            })
            .ok_or(AppError::BadRequest)?;
        updates.insert("status", "CONFIRMED");
        updates.insert("alex_order_id", order);
        if let Some(number) = input.order_number {
            updates.insert(
                "alex_order_number",
                reference_value(&json!(number)).ok_or(AppError::BadRequest)?,
            );
        }
    }
    filter.insert(
        "status",
        record.get_str("status").map_err(|_| AppError::Data)?,
    );
    if state
        .collection("alex_submissions")
        .update_one(filter, doc! {"$set":updates})
        .await?
        .matched_count
        != 1
    {
        return Err(AppError::BillPending);
    }
    unlock(state, id, submission).await?;
    safe_submission(
        state
            .collection("alex_submissions")
            .find_one(scoped_id(state, id, submission))
            .await?
            .ok_or(AppError::Data)?,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn submission_response_keeps_missing_references_explicit_without_inventing_values() {
        let record = doc! {"status":"SUBMITTING"};
        let value = safe_submission(record.clone()).unwrap();
        assert_eq!(value.get("alex_order_id"), Some(&Value::Null));
        assert_eq!(value.get("alex_order_number"), Some(&Value::Null));
        let mut confirmed = record;
        confirmed.extend(doc! {"status":"CONFIRMED","alex_order_id":"id-123"});
        let value = safe_submission(confirmed.clone()).unwrap();
        assert_eq!(value["alex_order_id"], "id-123");
        assert_eq!(value.get("alex_order_number"), Some(&Value::Null));
        confirmed.insert("alex_order_number", "AR-2026-001");
        let value = safe_submission(confirmed).unwrap();
        assert_eq!(value["alex_order_id"], "id-123");
        assert_eq!(value["alex_order_number"], "AR-2026-001");
    }

    #[test]
    fn only_notes_saved_by_the_admin_prefill_the_invoice() {
        let id = ObjectId::new();
        let note = Some("ใช้งานชุด 28 พ.ย. 69");
        let mut draft = doc! {"_id":id,"result":{"evidence":[{"field":"notes","source_type":"admin_edit","source_ref":id.to_hex()}]}};
        assert_eq!(manual_draft_note(&draft, note), note);
        draft.insert("result", doc! {"evidence":[{"field":"notes","source_type":"admin_summary","source_ref":id.to_hex()}]});
        assert_eq!(
            manual_draft_note(&draft, Some("ems , flash; ต้องยืนยันวันคืน\n8:19")),
            None
        );
        draft.insert("result", doc! {"evidence":[{"field":"notes","source_type":"admin_edit","source_ref":ObjectId::new().to_hex()}]});
        assert_eq!(manual_draft_note(&draft, note), None);
    }

    #[test]
    fn quoted_fees_below_the_catalog_become_discounts() {
        assert_eq!(discount(None, Some(50_000), 64_000), 14_000);
        assert_eq!(discount(Some(10_000), Some(50_000), 64_000), 10_000);
        assert_eq!(discount(Some(0), Some(50_000), 64_000), 0);
        assert_eq!(discount(None, Some(50_000), 70_000), 20_000);
        assert_eq!(discount(Some(5_000), Some(50_000), 70_000), 5_000);
        assert_eq!(discount(Some(0), Some(50_000), 70_000), 0);
        assert_eq!(discount(None, Some(80_000), 70_000), 0);
        assert_eq!(discount(None, None, 70_000), 0);
    }

    #[test]
    fn order_references_are_distinct_and_bound_to_one_response_object() {
        let id = Some("id-123".into());
        let number = Some("AR-2026-001".into());
        assert_eq!(
            order_reference(
                200,
                &json!({"orderId":"id-123","orderNumber":"AR-2026-001","customerId":"customer"}),
                None,
                None
            ),
            (id.clone(), number.clone())
        );
        assert_eq!(
            order_reference(
                201,
                &json!({"data":{"OrderId":"id-123","OrderNumber":"AR-2026-001"}}),
                None,
                None
            ),
            (id.clone(), number.clone())
        );
        assert_eq!(
            order_reference(
                200,
                &json!({"data":{"id":"id-123","number":"AR-2026-001"}}),
                Some("/data/id"),
                Some("/data/number")
            ),
            (id.clone(), number)
        );
        assert_eq!(
            order_reference(
                200,
                &json!({"orderId":"id-123","data":{"orderNumber":"unrelated"}}),
                Some("/orderId"),
                None
            ),
            (id, None)
        );
        assert_eq!(
            order_reference(
                200,
                &json!({"orderId":42,"orderNumber":"0000819"}),
                None,
                None
            ),
            (Some("42".into()), Some("0000819".into()))
        );
        for (status, value) in [
            (500, json!({"orderId":"id-123","orderNumber":"failed"})),
            (200, json!({"success":false,"orderId":"id-123"})),
            (200, json!({"data":{"success":false,"orderId":"id-123"}})),
            (
                200,
                json!({"customer":{"id":"customer"},"productItemId":"product","number":"not-order"}),
            ),
            (200, json!({"orderId":"a","data":{"orderId":"b"}})),
            (200, json!({"orderId":0,"orderNumber":"bad\nvalue"})),
        ] {
            assert_eq!(
                order_reference(status, &value, None, None),
                (None, None),
                "{value}"
            );
        }
    }
}
