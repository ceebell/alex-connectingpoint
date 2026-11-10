use crate::models::customer_notes::AliasInput;
use crate::{
    db::events::duplicate,
    error::AppError,
    models::customer::{Actor, ProfileInput},
    state::AppState,
};
use futures_util::TryStreamExt;
use mongodb::{
    bson::{Bson, DateTime, Document, doc, oid::ObjectId},
    options::ReturnDocument,
};
use std::collections::HashMap;

pub async fn ensure_user(state: &AppState, user_id: &str) -> Result<(), AppError> {
    if state
        .collection("line_users")
        .find_one(identity(state, user_id))
        .await?
        .is_none()
    {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub async fn alias(state: &AppState, user_id: &str) -> Result<Option<Document>, AppError> {
    Ok(state
        .collection("customer_aliases")
        .find_one(identity(state, user_id))
        .await?)
}

pub async fn aliases(
    state: &AppState,
    ids: Vec<String>,
) -> Result<HashMap<String, Document>, AppError> {
    let mut filter = state.scope();
    filter.insert("line_user_id", doc! {"$in":ids});
    let records: Vec<Document> = state
        .collection("customer_aliases")
        .find(filter)
        .await?
        .try_collect()
        .await?;
    Ok(records
        .into_iter()
        .filter_map(|record| Some((record.get_str("line_user_id").ok()?.to_string(), record)))
        .collect())
}

pub async fn save_alias(
    state: &AppState,
    user_id: &str,
    input: AliasInput,
    actor: &Actor,
) -> Result<Document, AppError> {
    ensure_user(state, user_id).await?;
    let now = DateTime::now();
    let fields = doc! {"custom_name":text(&input.custom_name),"updated_at":now,"updated_by":&actor.name,"updated_by_id":&actor.id};
    if input.expected_version == 0 {
        let mut record = identity(state, user_id);
        record.extend(fields);
        record.extend(doc! {"_id":ObjectId::new(),"created_at":now,"version":1_i64});
        return match state
            .collection("customer_aliases")
            .insert_one(record.clone())
            .await
        {
            Ok(_) => Ok(record),
            Err(error) if duplicate(&error) => Err(AppError::RecordConflict),
            Err(error) => Err(error.into()),
        };
    }
    let mut filter = identity(state, user_id);
    filter.insert("version", input.expected_version);
    state
        .collection("customer_aliases")
        .find_one_and_update(filter, doc! {"$set":fields,"$inc":{"version":1_i64}})
        .return_document(ReturnDocument::After)
        .await?
        .ok_or(AppError::RecordConflict)
}

fn identity(state: &AppState, user_id: &str) -> Document {
    let mut filter = state.scope();
    filter.insert("line_user_id", user_id);
    filter
}

fn text(value: &Option<String>) -> Bson {
    value
        .as_ref()
        .map_or(Bson::Null, |v| Bson::String(v.clone()))
}

pub async fn get(state: &AppState, user_id: &str) -> Result<Option<Document>, AppError> {
    Ok(state
        .collection("customer_profiles")
        .find_one(identity(state, user_id))
        .await?)
}

pub async fn save(
    state: &AppState,
    user_id: &str,
    input: ProfileInput,
    actor: &Actor,
) -> Result<Document, AppError> {
    // Profiles exist only for LINE users actually seen through the signed webhook.
    if state
        .collection("line_users")
        .find_one(identity(state, user_id))
        .await?
        .is_none()
    {
        return Err(AppError::NotFound);
    }
    let now = DateTime::now();
    let fields = doc! {"display_name":text(&input.display_name),"phone":text(&input.phone),
    "address":text(&input.address),"notes":text(&input.notes),"updated_by":&actor.name,"updated_by_id":&actor.id,"updated_at":now};
    if input.expected_version == 0 {
        let mut record = identity(state, user_id);
        record.extend(fields);
        record.extend(doc! {"_id":ObjectId::new(),"version":1_i64,"created_at":now});
        return match state
            .collection("customer_profiles")
            .insert_one(record.clone())
            .await
        {
            Ok(_) => Ok(record),
            Err(error) if duplicate(&error) => Err(AppError::RecordConflict),
            Err(error) => Err(error.into()),
        };
    }
    let mut filter = identity(state, user_id);
    filter.insert("version", input.expected_version);
    state
        .collection("customer_profiles")
        .find_one_and_update(filter, doc! {"$set":fields,"$inc":{"version":1_i64}})
        .return_document(ReturnDocument::After)
        .await?
        .ok_or(AppError::RecordConflict)
}
