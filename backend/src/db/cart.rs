use crate::{
    alex,
    db::{drafts, events::duplicate, products},
    error::AppError,
    models::{
        cart::{AddInput, UpdateInput},
        customer::Actor,
        product::{Candidate, valid_id, valid_quantity},
    },
    state::AppState,
};
use futures_util::TryStreamExt;
use mongodb::bson::{self, DateTime, Document, doc, oid::ObjectId};
use serde_json::{Value, json};

const MAX_LINES: usize = 20;

pub async fn items(state: &AppState, inquiry_id: ObjectId) -> Result<Vec<Document>, AppError> {
    let mut filter = state.scope();
    filter.insert("inquiry_id", inquiry_id);
    Ok(state
        .collection("cart_items")
        .find(filter)
        .sort(doc! {"created_at":1,"_id":1})
        .limit(MAX_LINES as i64)
        .await?
        .try_collect()
        .await?)
}

pub fn candidate(record: &Document) -> Result<Candidate, AppError> {
    bson::from_document(
        record
            .get_document("candidate")
            .map_err(|_| AppError::Data)?
            .clone(),
    )
    .map_err(|_| AppError::Data)
}

pub async fn list(state: &AppState, inquiry_id: ObjectId) -> Result<Value, AppError> {
    let views = items(state, inquiry_id)
        .await?
        .into_iter()
        .map(|mut record| {
            record.remove("added_by_id");
            serde_json::to_value(record).map_err(|_| AppError::Data)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(json!({"items":views}))
}

// Adds a catalog item from an inquiry search; adding the same item again raises its quantity.
pub async fn add(
    state: &AppState,
    inquiry_id: ObjectId,
    input: AddInput,
    actor: &Actor,
) -> Result<Value, AppError> {
    let quantity = input.quantity.unwrap_or(1);
    if !valid_id(&input.product_item_id) || !valid_quantity(quantity) {
        return Err(AppError::BadRequest);
    }
    let search_id = ObjectId::parse_str(&input.search_id).map_err(|_| AppError::BadRequest)?;
    let inquiry = drafts::inquiry(state, inquiry_id).await?;
    if inquiry.get_str("status").ok() == Some("CLOSED") {
        return Err(AppError::ProductConflict);
    }
    let record = products::search_record(state, inquiry_id, search_id).await?;
    let selected = products::search_candidates(&record)?
        .into_iter()
        .find(|candidate| candidate.product_item_id == input.product_item_id)
        .ok_or(AppError::ProductConflict)?;
    let branch = record
        .get_document("input")
        .and_then(|input| input.get_str("branch_id"))
        .map_err(|_| AppError::Data)?
        .to_string();
    // Re-read the exact catalog entry. IDs and prices never come from the browser alone.
    let session = alex::login(state).await?;
    alex::check_branch(&session, &branch)?;
    let mut latest = alex::lookup(
        state,
        &session,
        &branch,
        &selected.code,
        &selected.product_item_id,
    )
    .await?;
    if latest.code != selected.code
        || latest.name != selected.name
        || latest.color != selected.color
        || latest.size != selected.size
        || latest.rental_fee_satang != selected.rental_fee_satang
        || latest.deposit_satang != selected.deposit_satang
    {
        return Err(AppError::ProductConflict);
    }
    latest.image_url = latest.image_url.or(selected.image_url);
    latest.score = None;
    let existing = items(state, inquiry_id).await?;
    if existing
        .iter()
        .any(|item| item.get_str("branch_id").ok() != Some(branch.as_str()))
    {
        return Err(AppError::Validation(
            "Cart items must come from one Alex branch",
        ));
    }
    let now = DateTime::now();
    let collection = state.collection("cart_items");
    if let Some(current) = existing
        .iter()
        .find(|item| item.get_str("product_item_id").ok() == Some(latest.product_item_id.as_str()))
    {
        let mut filter = state.scope();
        filter.extend(
            doc! {"_id":current.get_object_id("_id").map_err(|_| AppError::Data)?,
            "revision":current.get_i64("revision").map_err(|_| AppError::Data)?},
        );
        let total = (current.get_i64("quantity").unwrap_or(1) + quantity).min(999);
        let updated = collection
            .update_one(filter, doc! {"$set":{"quantity":total,"candidate":bson::to_document(&latest).map_err(|_| AppError::Data)?,
                "updated_at":now},"$inc":{"revision":1_i64}})
            .await?;
        if updated.matched_count == 0 {
            return Err(AppError::ProductConflict);
        }
    } else {
        if existing.len() >= MAX_LINES {
            return Err(AppError::Validation("A cart holds at most 20 items"));
        }
        let mut line = state.scope();
        line.extend(doc! {"_id":ObjectId::new(),"inquiry_id":inquiry_id,"product_item_id":&latest.product_item_id,
            "branch_id":&branch,"candidate":bson::to_document(&latest).map_err(|_| AppError::Data)?,"quantity":quantity,
            "revision":1_i64,"search_id":search_id,"added_by":&actor.name,"added_by_id":&actor.id,"created_at":now,"updated_at":now});
        match collection.insert_one(line).await {
            Ok(_) => (),
            Err(error) if duplicate(&error) => return Err(AppError::ProductConflict),
            Err(error) => return Err(error.into()),
        }
    }
    list(state, inquiry_id).await
}

pub async fn update(
    state: &AppState,
    inquiry_id: ObjectId,
    input: UpdateInput,
) -> Result<Value, AppError> {
    let id = ObjectId::parse_str(&input.item_id).map_err(|_| AppError::BadRequest)?;
    if input.expected_revision < 1
        || input.expected_revision == i64::MAX
        || input
            .quantity
            .is_some_and(|quantity| !valid_quantity(quantity))
    {
        return Err(AppError::BadRequest);
    }
    let mut filter = state.scope();
    filter.extend(doc! {"_id":id,"inquiry_id":inquiry_id,"revision":input.expected_revision});
    let collection = state.collection("cart_items");
    let changed = match input.quantity {
        None => collection.delete_one(filter).await?.deleted_count,
        Some(quantity) => {
            collection
                .update_one(
                    filter,
                    doc! {"$set":{"quantity":quantity,"updated_at":DateTime::now()},"$inc":{"revision":1_i64}},
                )
                .await?
                .matched_count
        }
    };
    if changed == 0 {
        return Err(AppError::ProductConflict);
    }
    list(state, inquiry_id).await
}
