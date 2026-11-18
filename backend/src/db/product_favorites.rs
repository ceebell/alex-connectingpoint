use crate::{
    alex,
    db::{cart, drafts, events::duplicate, products, team},
    error::AppError,
    models::{cart::AddInput, customer::Actor, product::valid_id, product_favorites::SaveInput},
    state::AppState,
};
use futures_util::TryStreamExt;
use mongodb::bson::{self, DateTime, Document, doc, oid::ObjectId};
use serde_json::{Value, json};

fn branch(state: &AppState, id: &str) -> Result<(), AppError> {
    if !valid_id(id) {
        return Err(AppError::BadRequest);
    }
    let config = state
        .config
        .alex
        .as_ref()
        .ok_or(AppError::AlexUnavailable)?;
    if !config.branch_ids.is_empty() && !config.branch_ids.iter().any(|allowed| allowed == id) {
        return Err(AppError::AlexScope);
    }
    team::check_branch(state, id)
}

pub async fn list(state: &AppState, inquiry: ObjectId, branch_id: &str) -> Result<Value, AppError> {
    drafts::inquiry(state, inquiry).await?;
    branch(state, branch_id)?;
    let mut filter = state.scope();
    filter.insert("branch_id", branch_id);
    let records: Vec<Document> = state
        .collection("product_favorites")
        .find(filter)
        .sort(doc! {"created_at":-1,"_id":-1})
        .await?
        .try_collect()
        .await?;
    Ok(json!({"items":records.iter().map(view).collect::<Result<Vec<_>, _>>()?}))
}

fn view(record: &Document) -> Result<Value, AppError> {
    Ok(
        json!({"id":record.get_object_id("_id").map_err(|_|AppError::Data)?.to_hex(),
        "branch_id":record.get_str("branch_id").map_err(|_|AppError::Data)?,
        "candidate":record.get_document("candidate").map_err(|_|AppError::Data)?,
        "created_at":record.get_datetime("created_at").map_err(|_|AppError::Data)?.try_to_rfc3339_string().map_err(|_|AppError::Data)?}),
    )
}

pub async fn save(
    state: &AppState,
    inquiry: ObjectId,
    input: SaveInput,
    actor: &Actor,
) -> Result<Value, AppError> {
    drafts::inquiry(state, inquiry).await?;
    if !valid_id(&input.product_item_id) {
        return Err(AppError::BadRequest);
    }
    let search_id = ObjectId::parse_str(&input.search_id).map_err(|_| AppError::BadRequest)?;
    let search = products::search_record(state, inquiry, search_id).await?;
    let branch_id = search
        .get_document("input")
        .and_then(|input| input.get_str("branch_id"))
        .map_err(|_| AppError::Data)?;
    branch(state, branch_id)?;
    let candidate = products::search_candidates(&search)?
        .into_iter()
        .find(|candidate| candidate.product_item_id == input.product_item_id)
        .ok_or(AppError::ProductConflict)?;
    let mut filter = state.scope();
    filter.extend(doc! {"branch_id":branch_id,"product_item_id":&candidate.product_item_id});
    let now = DateTime::now();
    let update = doc! {"$set":{"candidate":bson::to_document(&candidate).map_err(|_|AppError::Data)?,"updated_at":now},
    "$setOnInsert":{"_id":ObjectId::new(),"created_at":now,"created_by_id":&actor.id,"created_by":&actor.name}};
    match state
        .collection("product_favorites")
        .update_one(filter.clone(), update)
        .upsert(true)
        .await
    {
        Ok(_) => (),
        Err(error) if duplicate(&error) => (),
        Err(error) => return Err(error.into()),
    }
    view(
        &state
            .collection("product_favorites")
            .find_one(filter)
            .await?
            .ok_or(AppError::ProductConflict)?,
    )
}

async fn record(state: &AppState, inquiry: ObjectId, id: ObjectId) -> Result<Document, AppError> {
    drafts::inquiry(state, inquiry).await?;
    let mut filter = state.scope();
    filter.insert("_id", id);
    let record = state
        .collection("product_favorites")
        .find_one(filter)
        .await?
        .ok_or(AppError::NotFound)?;
    branch(
        state,
        record.get_str("branch_id").map_err(|_| AppError::Data)?,
    )?;
    Ok(record)
}

pub async fn remove(state: &AppState, inquiry: ObjectId, id: ObjectId) -> Result<Value, AppError> {
    let record = record(state, inquiry, id).await?;
    let mut filter = state.scope();
    filter.extend(
        doc! {"_id":id,"branch_id":record.get_str("branch_id").map_err(|_|AppError::Data)?},
    );
    state
        .collection("product_favorites")
        .delete_one(filter)
        .await?;
    Ok(json!({"removed":true}))
}

pub async fn add_to_cart(
    state: &AppState,
    inquiry: ObjectId,
    id: ObjectId,
    actor: &Actor,
) -> Result<Value, AppError> {
    let record = record(state, inquiry, id).await?;
    let branch_id = record.get_str("branch_id").map_err(|_| AppError::Data)?;
    let candidate = bson::from_document::<crate::models::product::Candidate>(
        record
            .get_document("candidate")
            .map_err(|_| AppError::Data)?
            .clone(),
    )
    .map_err(|_| AppError::Data)?;
    let session = alex::login(state).await?;
    alex::check_branch(&session, branch_id)?;
    // Lookup requires the exact product ID, code and branch, never a similar item.
    let latest = alex::lookup(
        state,
        &session,
        branch_id,
        &candidate.code,
        &candidate.product_item_id,
    )
    .await?;
    let latest_document = bson::to_document(&latest).map_err(|_| AppError::Data)?;
    let now = DateTime::now();
    let search_id = ObjectId::new();
    let mut search = state.scope();
    search.extend(doc! {"_id":search_id,"inquiry_id":inquiry,"input":{"branch_id":branch_id},
        "candidates":[latest_document.clone()],"method":"favorite",
        "created_at":now,"updated_at":now,"expires_at":DateTime::from_millis(now.timestamp_millis()+1_800_000)});
    state
        .collection("product_searches")
        .insert_one(search)
        .await?;
    let cart = cart::add(
        state,
        inquiry,
        AddInput {
            search_id: search_id.to_hex(),
            product_item_id: latest.product_item_id.clone(),
            quantity: Some(1),
        },
        actor,
    )
    .await?;
    // A failed display snapshot refresh must not turn a successful cart mutation into an error.
    let mut filter = state.scope();
    filter.insert("_id", id);
    if let Err(error) = state
        .collection("product_favorites")
        .update_one(
            filter,
            doc! {"$set":{"candidate":latest_document,"updated_at":now}},
        )
        .await
    {
        tracing::warn!(error=%error, "Could not refresh favorite display snapshot");
    }
    Ok(cart)
}
