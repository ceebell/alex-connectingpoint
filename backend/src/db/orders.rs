use crate::{
    db::{drafts, events::duplicate},
    error::AppError,
    models::customer::{Actor, ApproveInput},
    state::AppState,
};
use mongodb::bson::{DateTime, Document, doc, oid::ObjectId};

pub async fn get(state: &AppState, inquiry: ObjectId) -> Result<Option<Document>, AppError> {
    let mut filter = state.scope();
    filter.insert("inquiry_id", inquiry);
    Ok(state.collection("orders").find_one(filter).await?)
}

fn same_draft(existing: Document, draft: ObjectId) -> Result<Document, AppError> {
    if existing.get_object_id("draft_id").ok() == Some(draft) {
        Ok(existing)
    } else {
        Err(AppError::OrderExists)
    }
}

pub async fn approve(
    state: &AppState,
    inquiry_id: ObjectId,
    input: ApproveInput,
    actor: &Actor,
) -> Result<Document, AppError> {
    input.validate()?;
    let draft_id = ObjectId::parse_str(&input.draft_id).map_err(|_| AppError::BadRequest)?;
    // Approval is idempotent for the same draft and refuses a second order.
    if let Some(existing) = get(state, inquiry_id).await? {
        return same_draft(existing, draft_id);
    }
    let inquiry = drafts::inquiry(state, inquiry_id).await?;
    if inquiry.get_str("status").ok() == Some("CLOSED")
        || inquiry.get_object_id("active_draft_id").ok() != Some(draft_id)
        || drafts::version(&inquiry) != input.draft_version
    {
        return Err(AppError::OrderNotReady);
    }
    let draft = drafts::active(state, &inquiry)
        .await?
        .ok_or(AppError::OrderNotReady)?;
    let snapshot = drafts::snapshot(state, &inquiry).await?;
    if snapshot.pending
        || snapshot.hash != input.context_hash
        || draft.get_str("context_hash").ok() != Some(snapshot.hash.as_str())
    {
        return Err(AppError::ContextChanged);
    }
    if drafts::references_withdrawn(&draft, &snapshot)
        || !drafts::summary_matches(&draft, &inquiry)
        || draft.get_str("workflow_status").ok() != Some("READY_FOR_REVIEW")
        || draft.get_i64("draft_version").ok() != Some(input.draft_version)
    {
        return Err(AppError::OrderNotReady);
    }
    let fields = draft
        .get_document("result")
        .and_then(|result| result.get_document("fields"))
        .map_err(|_| AppError::Data)?
        .clone();
    let now = DateTime::now();
    let mut order = state.scope();
    order.extend(doc! {"_id":ObjectId::new(),"inquiry_id":inquiry_id,
        "conversation_id":inquiry.get_object_id("conversation_id").ok(),"line_user_id":inquiry.get_str("line_user_id").ok(),
        "draft_id":draft_id,"draft_version":input.draft_version,"context_hash":&snapshot.hash,"fields":fields,
        "status":"CONFIRMED","approved_by":&actor.name,"approved_by_id":&actor.id,"approved_at":now,
        // Quoted values still need checking against Alex stock and pricing.
        "pricing_verified":false,"product_verified":false,"availability_verified":false,
        "created_at":now,"updated_at":now});
    match state.collection("orders").insert_one(order.clone()).await {
        Ok(_) => (),
        Err(error) if duplicate(&error) => {
            let existing = get(state, inquiry_id).await?.ok_or(AppError::Data)?;
            return same_draft(existing, draft_id);
        }
        Err(error) => return Err(error.into()),
    }
    let mut filter = state.scope();
    filter.insert("_id", inquiry_id);
    state
        .collection("inquiries")
        .update_one(
            filter,
            doc! {"$set":{"order_id":order.get_object_id("_id").map_err(|_| AppError::Data)?,"updated_at":now}},
        )
        .await?;
    Ok(order)
}
