use crate::{
    alex,
    db::{drafts, events::duplicate},
    error::AppError,
    models::{
        customer::Actor,
        draft::DraftFields,
        product::{Candidate, ConfirmInput, SearchInput, SearchMode, valid_id},
    },
    state::AppState,
};
use futures_util::TryStreamExt;
use mongodb::{
    bson::{self, DateTime, Document, doc, oid::ObjectId},
    options::ReturnDocument,
};
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

fn candidate_view(
    candidate: &Candidate,
    inquiry: ObjectId,
    search: ObjectId,
) -> Result<Value, AppError> {
    let mut value = serde_json::to_value(candidate).map_err(|_| AppError::Data)?;
    value["image_url"] = json!(candidate.image_url.as_ref().map(|_| format!(
        "/api/inquiries/{inquiry}/product-searches/{search}/images/{}",
        candidate.product_item_id
    )));
    Ok(value)
}

pub async fn search(
    state: &AppState,
    inquiry_id: ObjectId,
    input: SearchInput,
) -> Result<Value, AppError> {
    input.validate()?;
    let (inquiry, draft, fields, snapshot) = current(state, inquiry_id).await?;
    if drafts::version(&inquiry) != input.draft_version
        || snapshot.hash != input.context_hash
        || input.item_index >= fields.items.len()
    {
        return Err(AppError::ProductConflict);
    }
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
    let (fresh, _, _, fresh_snapshot) = current(state, inquiry_id).await?;
    if drafts::version(&fresh) != input.draft_version || fresh_snapshot.hash != snapshot.hash {
        return Err(AppError::ContextChanged);
    }
    let now = DateTime::now();
    let id = ObjectId::new();
    let mut record = state.scope();
    record.extend(doc! {"_id":id,"inquiry_id":inquiry_id,"draft_id":draft.get_object_id("_id").map_err(|_| AppError::Data)?,
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
        .map(|candidate| candidate_view(candidate, inquiry_id, id))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(
        json!({"search_id":id.to_hex(),"branch_id":input.branch_id,"item_index":input.item_index,
        "mode":input.mode,"method":method,"page":input.page,"has_more":has_more,"candidates":candidates,
        "requires_confirmation":true,"availability_verified":false,"pricing_verified":false}),
    )
}

async fn search_record(
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

fn search_candidates(record: &Document) -> Result<Vec<Candidate>, AppError> {
    bson::from_bson(record.get("candidates").cloned().ok_or(AppError::Data)?)
        .map_err(|_| AppError::Data)
}

pub async fn confirm(
    state: &AppState,
    inquiry_id: ObjectId,
    input: ConfirmInput,
    actor: &Actor,
) -> Result<Value, AppError> {
    if input.expected_revision < 0
        || input.expected_revision == i64::MAX
        || !valid_id(&input.product_item_id)
    {
        return Err(AppError::BadRequest);
    }
    let search_id = ObjectId::parse_str(&input.search_id).map_err(|_| AppError::BadRequest)?;
    let record = search_record(state, inquiry_id, search_id).await?;
    let (inquiry, draft, fields, snapshot) = current(state, inquiry_id).await?;
    let draft_id = draft.get_object_id("_id").map_err(|_| AppError::Data)?;
    if record.get_object_id("draft_id").ok() != Some(draft_id)
        || record.get_str("context_hash").ok() != Some(&snapshot.hash)
        || !drafts::summary_matches(&record, &inquiry)
    {
        return Err(AppError::ProductConflict);
    }
    let mut query: SearchInput = bson::from_document(
        record
            .get_document("input")
            .map_err(|_| AppError::Data)?
            .clone(),
    )
    .map_err(|_| AppError::Data)?;
    if query.item_index >= fields.items.len() {
        return Err(AppError::ProductConflict);
    }
    let selected = search_candidates(&record)?
        .into_iter()
        .find(|candidate| candidate.product_item_id == input.product_item_id)
        .ok_or(AppError::ProductConflict)?;
    let item = &fields.items[query.item_index];
    let differences: Vec<_> = [
        ("product_code", &item.product_code, Some(&selected.code)),
        ("color", &item.color, selected.color.as_ref()),
        ("size", &item.size, selected.size.as_ref()),
        ("product_name", &item.product_name, Some(&selected.name)),
    ]
    .into_iter()
    .filter(|(_, draft, catalog)| {
        draft
            .as_ref()
            .is_some_and(|value| catalog.is_none_or(|catalog| !value.eq_ignore_ascii_case(catalog)))
    })
    .map(|(field, _, _)| field)
    .collect();
    if !differences.is_empty() && !input.acknowledge_difference {
        return Err(AppError::ProductOptionsMismatch);
    }
    // Re-read the exact catalog entry at confirmation time. IDs never come from the model or browser alone.
    let session = alex::login(state).await?;
    query.mode = SearchMode::Code;
    query.query = Some(selected.code.clone());
    query.page = 1;
    let mut verified = None;
    loop {
        let (candidates, more) = alex::search_page(state, &session, &query).await?;
        if let Some(candidate) = candidates
            .into_iter()
            .find(|candidate| candidate.product_item_id == selected.product_item_id)
        {
            verified = Some(candidate);
            break;
        }
        if !more || query.page >= 100 {
            break;
        }
        query.page += 1;
    }
    let mut verified = verified.ok_or(AppError::ProductConflict)?;
    // A price or option change requires a fresh explicit choice.
    verified.image_url = selected.image_url.clone();
    verified.score = selected.score;
    if verified != selected {
        return Err(AppError::ProductConflict);
    }
    let (fresh, _, _, fresh_snapshot) = current(state, inquiry_id).await?;
    if drafts::version(&fresh) != drafts::version(&inquiry) || fresh_snapshot.hash != snapshot.hash
    {
        return Err(AppError::ContextChanged);
    }
    let mut identity = state.scope();
    identity.extend(
        doc! {"inquiry_id":inquiry_id,"draft_id":draft_id,"item_index":query.item_index as i64},
    );
    let now = DateTime::now();
    let values = doc! {"search_id":search_id,"candidate":bson::to_document(&selected).map_err(|_| AppError::Data)?,
    "branch_id":&query.branch_id,"draft_version":query.draft_version,"context_hash":snapshot.hash,
    "summary_revision":drafts::summary_revision(&inquiry),"confirmed_by":&actor.name,"confirmed_by_id":&actor.id,
    "draft_item":bson::to_document(item).map_err(|_| AppError::Data)?,"differences":differences,"differences_acknowledged":input.acknowledge_difference,
    "confirmed_at":now,"updated_at":now,"method":record.get_str("method").map_err(|_| AppError::Data)?,
    "image_event_id":record.get_document("input").ok().and_then(|input| input.get_str("image_event_id").ok())};
    let saved = if input.expected_revision == 0 {
        let mut record = identity.clone();
        record.extend(values);
        record.extend(doc! {"_id":ObjectId::new(),"revision":1_i64,"created_at":now});
        match state
            .collection("product_matches")
            .insert_one(record.clone())
            .await
        {
            Ok(_) => record,
            Err(error) if duplicate(&error) => return Err(AppError::ProductConflict),
            Err(error) => return Err(error.into()),
        }
    } else {
        identity.insert("revision", input.expected_revision);
        state
            .collection("product_matches")
            .find_one_and_update(identity, doc! {"$set":values,"$inc":{"revision":1_i64}})
            .return_document(ReturnDocument::After)
            .await?
            .ok_or(AppError::ProductConflict)?
    };
    match_view(saved, inquiry_id)
}

fn match_view(mut record: Document, inquiry: ObjectId) -> Result<Value, AppError> {
    let candidate: Candidate = bson::from_document(
        record
            .get_document("candidate")
            .map_err(|_| AppError::Data)?
            .clone(),
    )
    .map_err(|_| AppError::Data)?;
    let search = record
        .get_object_id("search_id")
        .map_err(|_| AppError::Data)?;
    record.remove("confirmed_by_id");
    let mut result = serde_json::to_value(record).map_err(|_| AppError::Data)?;
    result["candidate"] = candidate_view(&candidate, inquiry, search)?;
    Ok(result)
}

pub async fn list(state: &AppState, inquiry_id: ObjectId) -> Result<Value, AppError> {
    let (inquiry, draft, fields, _) = current(state, inquiry_id).await?;
    let mut filter = state.scope();
    filter.extend(doc! {"inquiry_id":inquiry_id,"draft_id":draft.get_object_id("_id").map_err(|_| AppError::Data)?});
    let records: Vec<Document> = state
        .collection("product_matches")
        .find(filter)
        .sort(doc! {"item_index":1})
        .limit(20)
        .await?
        .try_collect()
        .await?;
    let matches = records
        .into_iter()
        .map(|record| match_view(record, inquiry_id))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(
        json!({"draft_version":drafts::version(&inquiry),"matches":matches,
        "product_verified":!fields.items.is_empty() && matches.len()==fields.items.len(),
        "availability_verified":false,"pricing_verified":false}),
    )
}

pub async fn image(
    state: &AppState,
    inquiry: ObjectId,
    search: ObjectId,
    product_id: &str,
) -> Result<(String, Vec<u8>), AppError> {
    let record = search_record(state, inquiry, search).await?;
    let (_, draft, _, snapshot) = current(state, inquiry).await?;
    if record.get_object_id("draft_id").ok() != draft.get_object_id("_id").ok()
        || record.get_str("context_hash").ok() != Some(&snapshot.hash)
    {
        return Err(AppError::ContextChanged);
    }
    let selected = search_candidates(&record)?
        .into_iter()
        .find(|candidate| candidate.product_item_id == product_id)
        .ok_or(AppError::NotFound)?;
    let session = alex::login(state).await?;
    let branch = record
        .get_document("input")
        .map_err(|_| AppError::Data)?
        .get_str("branch_id")
        .map_err(|_| AppError::Data)?;
    alex::check_branch(&session, branch)?;
    alex::image(
        state,
        &session,
        selected.image_url.as_deref().ok_or(AppError::NotFound)?,
    )
    .await
}
