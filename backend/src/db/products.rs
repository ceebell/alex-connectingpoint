use crate::{
    alex,
    db::drafts,
    error::AppError,
    models::{
        draft::DraftFields,
        product::{Candidate, SearchInput, SearchMode},
    },
    state::AppState,
};
use mongodb::bson::{self, DateTime, Document, doc, oid::ObjectId};
use serde_json::{Value, json};

pub async fn current(
    state: &AppState,
    id: ObjectId,
) -> Result<(Document, Document, DraftFields, drafts::Snapshot), AppError> {
    let inquiry = drafts::inquiry(state, id).await?;
    if inquiry.get_str("status").ok() == Some("CLOSED") {
        return Err(AppError::ProductConflict);
    }
    let draft = drafts::active(state, &inquiry)
        .await?
        .ok_or(AppError::ProductConflict)?;
    let snapshot = drafts::snapshot(state, &inquiry).await?;
    if snapshot.pending
        || draft.get_str("context_hash").ok() != Some(&snapshot.hash)
        || !drafts::summary_matches(&draft, &inquiry)
        || drafts::references_withdrawn(&draft, &snapshot)
    {
        return Err(AppError::ContextChanged);
    }
    let fields = bson::from_document(
        draft
            .get_document("result")
            .and_then(|result| result.get_document("fields"))
            .map_err(|_| AppError::Data)?
            .clone(),
    )
    .map_err(|_| AppError::Data)?;
    Ok((inquiry, draft, fields, snapshot))
}

// image_url is the public Alex catalog link, already checked against the allowed origins.
fn candidate_view(candidate: &Candidate) -> Result<Value, AppError> {
    serde_json::to_value(candidate).map_err(|_| AppError::Data)
}

pub async fn search(
    state: &AppState,
    inquiry_id: ObjectId,
    input: SearchInput,
) -> Result<Value, AppError> {
    input.validate()?;
    let inquiry = drafts::inquiry(state, inquiry_id).await?;
    if inquiry.get_str("status").ok() == Some("CLOSED") {
        return Err(AppError::ProductConflict);
    }
    // Image search needs the chat snapshot; cart searches are not bound to a draft version.
    let snapshot = drafts::snapshot(state, &inquiry).await?;
    let draft_id = drafts::active(state, &inquiry)
        .await?
        .and_then(|draft| draft.get_object_id("_id").ok());
    let session = alex::login(state).await?;
    let (candidates, has_more, method) = if input.mode == SearchMode::Image {
        crate::product_vision::search(state, &session, &snapshot, &input).await?
    } else {
        let (candidates, more) = alex::search_page(state, &session, &input).await?;
        (
            candidates,
            more,
            serde_json::to_value(&input.mode)
                .map_err(|_| AppError::Data)?
                .as_str()
                .ok_or(AppError::Data)?
                .to_string(),
        )
    };
    let now = DateTime::now();
    let id = ObjectId::new();
    let mut record = state.scope();
    record.extend(doc! {"_id":id,"inquiry_id":inquiry_id,"draft_id":draft_id,
        "draft_version":input.draft_version,"summary_revision":drafts::summary_revision(&inquiry),
        "context_hash":&snapshot.hash,"input":bson::to_document(&input).map_err(|_| AppError::Data)?,"method":&method,
        "candidates":bson::to_bson(&candidates).map_err(|_| AppError::Data)?,
        "created_at":now,"updated_at":now,"expires_at":DateTime::from_millis(now.timestamp_millis()+1_800_000)});
    state
        .collection("product_searches")
        .insert_one(record)
        .await?;
    let candidates = candidates
        .iter()
        .map(candidate_view)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(
        json!({"search_id":id.to_hex(),"branch_id":input.branch_id,"item_index":input.item_index,
        "mode":input.mode,"method":method,"page":input.page,"has_more":has_more,"candidates":candidates,
        "requires_confirmation":true,"availability_verified":false,"pricing_verified":false}),
    )
}

pub(crate) async fn search_record(
    state: &AppState,
    inquiry: ObjectId,
    search: ObjectId,
) -> Result<Document, AppError> {
    let mut filter = state.scope();
    filter.extend(doc! {"_id":search,"inquiry_id":inquiry,"expires_at":{"$gt":DateTime::now()}});
    state
        .collection("product_searches")
        .find_one(filter)
        .await?
        .ok_or(AppError::ProductConflict)
}

pub(crate) fn search_candidates(record: &Document) -> Result<Vec<Candidate>, AppError> {
    bson::from_bson(record.get("candidates").cloned().ok_or(AppError::Data)?)
        .map_err(|_| AppError::Data)
}

// Catalog suggestions the AI lookup found for the latest draft's requested items.
pub async fn list(state: &AppState, inquiry_id: ObjectId) -> Result<Value, AppError> {
    let (inquiry, draft, _, _) = current(state, inquiry_id).await?;
    let mut suggestions = Vec::new();
    if let Ok(lookups) = draft.get_array("catalog_lookup") {
        for lookup in lookups.iter().filter_map(|value| value.as_document()) {
            let mut value = serde_json::to_value(lookup).map_err(|_| AppError::Data)?;
            if let Ok(id) = lookup.get_str("search_id") {
                let id = ObjectId::parse_str(id).map_err(|_| AppError::Data)?;
                match search_record(state, inquiry_id, id).await {
                    Ok(record)
                        if record.get_object_id("draft_id").ok()
                            == draft.get_object_id("_id").ok() =>
                    {
                        let input: SearchInput = bson::from_document(
                            record
                                .get_document("input")
                                .map_err(|_| AppError::Data)?
                                .clone(),
                        )
                        .map_err(|_| AppError::Data)?;
                        let candidates = search_candidates(&record)?
                            .iter()
                            .map(candidate_view)
                            .collect::<Result<Vec<_>, _>>()?;
                        value["search"] = json!({"search_id":id.to_hex(),"item_index":input.item_index,"branch_id":input.branch_id,
                            "mode":input.mode,"query":input.query,"color":input.color,"size":input.size,
                            "method":"admin_input","page":input.page,"has_more":record.get_bool("has_more").unwrap_or(false),"candidates":candidates});
                    }
                    Ok(_) | Err(AppError::ProductConflict) => {
                        value["status"] = json!("EXPIRED");
                    }
                    Err(error) => return Err(error),
                }
            }
            suggestions.push(value);
        }
    }
    Ok(json!({"draft_version":drafts::version(&inquiry),"suggestions":suggestions}))
}
