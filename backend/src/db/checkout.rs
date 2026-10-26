use crate::{
    alex,
    db::{cart, drafts, products},
    error::AppError,
    models::{
        cart::CheckInput as CartCheckInput,
        checkout::{
            CheckInput, Group, Line, PAYMENT_METHODS, ResolveInput, ReviewInput, SubmitInput,
            UNPAID,
        },
        customer::{Actor, ProfileInput, normalize_phone},
        draft::{DraftFields, RECEIVING_METHODS, valid_date},
    },
    state::AppState,
};
use futures_util::TryStreamExt;
use mongodb::bson::{self, DateTime, Document, doc, oid::ObjectId};
use serde_json::{Value, json};
use std::collections::BTreeSet;

struct Source {
    inquiry: Document,
    draft: Document,
    fields: DraftFields,
    source_hash: String,
    groups: Vec<Group>,
    missing: Vec<Value>,
    has_conflicts: bool,
}

// One inquiry is one Alex order: cart lines plus the order-level dates, notes and discounts.
async fn source(state: &AppState, id: ObjectId) -> Result<Source, AppError> {
    let (inquiry, draft, fields, snapshot) = products::current(state, id).await?;
    let draft_id = draft.get_object_id("_id").map_err(|_| AppError::Data)?;
    let records = cart::items(state, id).await?;
    let source_hash =
        drafts::hash(&json!({"draft":draft_id.to_hex(),"context":snapshot.hash,"cart":records}))?;
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
        lines,
        unpriced,
        branches,
    } = cart_lines(&records)?;
    missing.extend(
        unpriced
            .into_iter()
            .map(|code| json!({"field":"catalog_price","code":code})),
    );
    if branches.len() > 1 {
        missing.push(json!({"field":"mixed_branches"}));
    }
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
            quoted_shipping_fee_satang: fields.amounts.shipping_fee_satang,
            rental_discount_satang: fields.amounts.rental_discount_satang.unwrap_or(0),
            deposit_discount_satang: fields.amounts.deposit_discount_satang.unwrap_or(0),
        });
    }
    let has_conflicts = draft
        .get_document("result")
        .ok()
        .and_then(|r| r.get_array("conflicts").ok())
        .is_some_and(|c| !c.is_empty());
    Ok(Source {
        inquiry,
        draft,
        fields,
        source_hash,
        groups,
        missing,
        has_conflicts,
    })
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

fn safe_submission(mut record: Document) -> Result<Value, AppError> {
    for key in ["payload", "alex_response", "submitted_by_id", "source_hash"] {
        record.remove(key);
    }
    serde_json::to_value(record).map_err(|_| AppError::Data)
}

pub async fn list(state: &AppState, id: ObjectId) -> Result<Value, AppError> {
    let inquiry = drafts::inquiry(state, id).await?;
    if let Ok(submission) = inquiry.get_object_id("alex_submission_id")
        && let Some(record) = recover(state, id, submission).await?
        && ["CONFIRMED", "NOT_CREATED"].contains(&record.get_str("status").unwrap_or(""))
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
    let submissions: Vec<Document> = state
        .collection("alex_submissions")
        .find(filter)
        .sort(doc! {"created_at":-1})
        .limit(50)
        .await?
        .try_collect()
        .await?;
    let submissions = submissions
        .into_iter()
        .map(safe_submission)
        .collect::<Result<Vec<_>, _>>()?;
    let source = match source(state, id).await {
        Ok(source) => source,
        Err(error) => {
            return Ok(
                json!({"source_hash":Value::Null,"draft_version":drafts::version(&inquiry),
        "groups":[],"missing":[],"has_conflicts":true,"submissions":submissions,"customer":{"name":Value::Null,"phone":Value::Null,"address":Value::Null},
        "notes":Value::Null,"context_error":error.code(),"order_response_configured":state.config.alex.as_ref().is_some_and(|cfg|cfg.order_id_pointer.is_some())}),
            );
        }
    };
    Ok(
        json!({"source_hash":source.source_hash,"draft_version":drafts::version(&source.inquiry),"groups":source.groups,"missing":source.missing,
        "has_conflicts":source.has_conflicts,"submissions":submissions,"customer":{"name":source.fields.customer_name,"phone":source.fields.customer_phone,"address":source.fields.customer_address},
        "notes":source.fields.notes,"order_response_configured":state.config.alex.as_ref().is_some_and(|cfg|cfg.order_id_pointer.is_some())}),
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
    let group = choose(&source, &input.group_key)?;
    let session = alex::login(state).await?;
    let results = check(state, &session, &group).await?;
    if self::source(state, id).await?.source_hash != source.source_hash {
        return Err(AppError::ContextChanged);
    }
    let status = if results.iter().all(|value| value["state"] == "AVAILABLE") {
        "AVAILABLE"
    } else if results.iter().any(|value| value["state"] == "UNAVAILABLE") {
        "UNAVAILABLE"
    } else {
        "UNKNOWN"
    };
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
    };
    let session = alex::login(state).await?;
    let results = check(state, &session, &group).await?;
    let status = if results.iter().all(|value| value["state"] == "AVAILABLE") {
        "AVAILABLE"
    } else if results.iter().any(|value| value["state"] == "UNAVAILABLE") {
        "UNAVAILABLE"
    } else {
        "UNKNOWN"
    };
    Ok(json!({"state":status,"items":results,"checked_at":DateTime::now(),"reserves_stock":false}))
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
    let group = choose(&source, &input.group_key)?;
    validate_input(&input, &group)?;
    let session = alex::login(state).await?;
    let results = check(state, &session, &group).await?;
    if results.iter().any(|result| result["state"] != "AVAILABLE") {
        return Err(AppError::BillNotReady);
    }
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
    if group.rental_discount_satang > rental || group.deposit_discount_satang > deposit {
        return Err(AppError::Validation(
            "Discounts cannot exceed the catalog rental or deposit totals",
        ));
    }
    let rental_fee = rental - group.rental_discount_satang;
    let deposit_fee = deposit - group.deposit_discount_satang;
    // Alex has no shipping field, so a shipping fee always goes into the order note.
    let mut notes = input.notes.trim().to_string();
    if input.shipping_fee_satang > 0 {
        if !notes.is_empty() {
            notes.push('\n');
        }
        notes.push_str(&format!(
            "ค่าจัดส่ง {} บาท (ไม่รวมในยอดค่าเช่า)",
            baht(input.shipping_fee_satang)
        ));
    }
    let mut detail = json!({"branchId":group.branch_id,"customerId":"","customerName":input.customer_name,"customerAddress":input.customer_address,"customerPhone":input.customer_phone,
        "lineId":"","howToKnowShop":"","accountNumber":"","accountName":"","bankName":"","startDate":group.pickup_date,"endDate":group.return_date,
        "receivingMethod":input.receiving_method,"remark1":notes,"rentalPaymentMethod":input.rental_payment_method,"bailPaymentMethod":input.bail_payment_method,
        "pointDiscount":0,"voucherDiscount":0,"rentalDiscount":baht(group.rental_discount_satang),"bailDiscount":baht(group.deposit_discount_satang),
        "useFreeItemPromotion":input.use_free_item_promotion,"useDiscountPromotion":input.use_discount_promotion});
    if let Some(birth) = &input.birth_date {
        detail["birthDate"] = json!(birth);
    }
    let payload = json!({"orderDetail":detail,"orderItems":group.lines.iter().map(|line|json!({"productItemId":line.product_item_id,"reservedQuantity":line.quantity})).collect::<Vec<_>>()});
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
        "created_at":now,"updated_at":now,"expires_at":DateTime::from_millis(now.timestamp_millis()+300_000)});
    state.collection("bill_reviews").insert_one(record).await?;
    Ok(
        json!({"review_id":review_id.to_hex(),"group":group,"customer":{"name":input.customer_name,"phone":input.customer_phone,"address":input.customer_address},
        "catalog_rental_satang":rental,"catalog_deposit_satang":deposit,"rental_fee_satang":rental_fee,"deposit_satang":deposit_fee,"shipping_fee_satang":input.shipping_fee_satang,
        "rental_discount_satang":group.rental_discount_satang,"deposit_discount_satang":group.deposit_discount_satang,
        "receiving_method":input.receiving_method,"rental_payment_method":input.rental_payment_method,"bail_payment_method":input.bail_payment_method,"notes":input.notes,"expires_at":record_expiry(now)}),
    )
}

fn record_expiry(now: DateTime) -> DateTime {
    DateTime::from_millis(now.timestamp_millis() + 300_000)
}

fn order_id(state: &AppState, status: u16, value: &Value) -> Option<String> {
    if !(200..300).contains(&status) || value.get("success") == Some(&json!(false)) {
        return None;
    }
    let pointer = state.config.alex.as_ref()?.order_id_pointer.as_ref()?;
    let id = if pointer == "$" {
        value
    } else {
        value.pointer(pointer)?
    };
    let id = if let Some(value) = id.as_str() {
        value.to_string()
    } else if let Some(value) = id.as_u64().filter(|n| *n > 0) {
        value.to_string()
    } else {
        return None;
    };
    (!id.trim().is_empty() && id.len() <= 200 && !id.chars().any(char::is_control)).then_some(id)
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
    let now = DateTime::now();
    let started = inquiry
        .get_datetime("alex_submission_started_at")
        .copied()
        .unwrap_or(now);
    let mut record = identity.clone();
    record.extend(doc! {"reservation_key":review.get_str("reservation_key").map_err(|_|AppError::Data)?,"active":true,"status":"SUBMIT_UNKNOWN",
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
                return Ok(Some(existing));
            }
            let mut filter = state.scope();
            filter.extend(doc! {"inquiry_id":id,"reservation_key":review.get_str("reservation_key").map_err(|_|AppError::Data)?,"active":true});
            let existing = state
                .collection("alex_submissions")
                .find_one(filter)
                .await?
                .ok_or(AppError::Data)?;
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
        return safe_submission(existing);
    }
    let mut filter = identity.clone();
    filter.insert("expires_at", doc! {"$gt":DateTime::now()});
    let review = state
        .collection("bill_reviews")
        .find_one(filter)
        .await?
        .ok_or(AppError::BillNotReady)?;
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
    let mut overlapping = state.scope();
    overlapping.extend(doc! {"inquiry_id":id,"active":true,"group.branch_id":&group.branch_id,
        "group.pickup_date":{"$lte":&group.return_date},"group.return_date":{"$gte":&group.pickup_date},
        "group.lines.product_item_id":{"$in":group.lines.iter().map(|line|line.product_item_id.clone()).collect::<Vec<_>>()}});
    if state
        .collection("alex_submissions")
        .find_one(overlapping)
        .await?
        .is_some()
    {
        unlock(state, id, review_id).await?;
        return Err(AppError::BillPending);
    }
    let now = DateTime::now();
    let mut record = identity.clone();
    record.extend(doc! {"reservation_key":review.get_str("reservation_key").map_err(|_|AppError::Data)?,"active":true,
        "status":"SUBMITTING","group":review.get("group").cloned().ok_or(AppError::Data)?,"source_hash":&source.source_hash,
        "payload":review.get("payload").cloned().ok_or(AppError::Data)?,"submitted_by":&actor.name,"submitted_by_id":&actor.id,"created_at":now,"updated_at":now});
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
        if results.iter().any(|value| value["state"] != "AVAILABLE") {
            return Err(AppError::BillNotReady);
        }
        if self::source(state, id).await?.source_hash != source.source_hash {
            return Err(AppError::ContextChanged);
        }
        Ok::<_, AppError>(session)
    }
    .await;
    let session = match preflight {
        Ok(session) => session,
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
    if state
        .collection("alex_submissions")
        .update_one(
            sending,
            doc! {"$set":{"sent_at":sent_at,"updated_at":sent_at}},
        )
        .await?
        .matched_count
        != 1
    {
        return Err(AppError::BillPending);
    }
    let outcome = alex::create_order(state, &session, &payload).await;
    let mut updates = doc! {"status":"SUBMIT_UNKNOWN","updated_at":DateTime::now()};
    if let Ok((status, mut response)) = outcome {
        if let Some(order) = order_id(state, status, &response) {
            updates.insert("status", "CONFIRMED");
            updates.insert("alex_order_id", order);
            updates.insert("verification", "api_response");
        }
        sanitize(&mut response);
        updates.insert("http_status", i32::from(status));
        updates.insert(
            "alex_response",
            bson::to_bson(&response).map_err(|_| AppError::Data)?,
        );
    }
    let confirmed = updates.get_str("status").ok() == Some("CONFIRMED");
    let mut finishing = identity.clone();
    finishing.insert("status", "SUBMITTING");
    let updated = state
        .collection("alex_submissions")
        .update_one(finishing, doc! {"$set":updates})
        .await?;
    if confirmed && updated.matched_count == 1 {
        unlock(state, id, review_id).await?;
    }
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
        updates.insert("active", true);
        updates.insert("alex_order_id", order);
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
