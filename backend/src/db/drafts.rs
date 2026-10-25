use crate::{
    db::{events, media},
    error::AppError,
    models::draft::PrepareInput,
    state::AppState,
};
use futures_util::TryStreamExt;
use mongodb::{
    bson::{self, DateTime, Document, doc, oid::ObjectId},
    options::ReturnDocument,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub struct Snapshot {
    pub events: Vec<Document>,
    pub content: Value,
    pub hash: String,
    pub pending: bool,
}

pub fn hash(value: &Value) -> Result<String, AppError> {
    let bytes = serde_json::to_vec(value).map_err(|_| AppError::Data)?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

pub fn version(inquiry: &Document) -> i64 {
    inquiry.get_i64("draft_version").unwrap_or(0)
}

pub async fn inquiry(state: &AppState, id: ObjectId) -> Result<Document, AppError> {
    let mut filter = state.scope();
    filter.insert("_id", id);
    state
        .collection("inquiries")
        .find_one(filter)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn snapshot(state: &AppState, inquiry: &Document) -> Result<Snapshot, AppError> {
    let id = inquiry.get_object_id("_id").map_err(|_| AppError::Data)?;
    let mut filter = state.scope();
    let mut alternatives = vec![doc! {"inquiry_id":id}];
    if let Ok(conversation) = inquiry.get_object_id("conversation_id") {
        let mut scope = state.scope();
        scope.insert("_id", conversation);
        if let Some(record) = state.collection("conversations").find_one(scope).await?
            && let Ok(key) = record.get_str("conversation_key")
        {
            // Also detect messages durably received but not yet assigned by Step 1.
            alternatives.push(doc! {"conversation_key":key, "inquiry_id":{"$exists":false}});
        }
    }
    filter.insert("$or", alternatives);
    let mut records: Vec<Document> = state
        .collection("line_events")
        .find(filter)
        .sort(doc! {"event_at":1, "event_id":1})
        .limit(501)
        .await?
        .try_collect()
        .await?;
    let mut oa_filter = state.scope();
    oa_filter.insert("inquiry_id", id);
    records.extend(
        state
            .collection("oa_messages")
            .find(oa_filter)
            .sort(doc! {"event_at":1,"event_id":1})
            .limit(501)
            .await?
            .try_collect::<Vec<_>>()
            .await?,
    );
    crate::db::oa_sync::sort_messages(&mut records);
    if records.len() > 500 {
        return Err(AppError::Validation(
            "Inquiry exceeds the 500-event preparation limit",
        ));
    }
    events::redact_withdrawn(state, &mut records).await?;
    let mut pending = false;
    let mut content = Vec::new();
    for record in &records {
        let status = record.get_str("status").unwrap_or("PENDING");
        pending |= matches!(status, "PENDING" | "PROCESSING");
        // Unassigned messages in an ambiguous conversation must never become AI
        // context for an arbitrarily selected inquiry. Wait for explicit assignment.
        pending |= record.get_object_id("inquiry_id").ok() != Some(id)
            && matches!(status, "NEEDS_ADMIN" | "FAILED");
        content.push(json!({
            "event_ref":record.get_object_id("_id").map_err(|_| AppError::Data)?.to_hex(),
            "event_at":record.get_datetime("event_at").map_err(|_| AppError::Data)?.timestamp_millis(),
            "sender_role":record.get_str("sender_role").unwrap_or("customer"),
            "source_type":crate::db::oa_sync::source_type(record),
            "status":status, "withdrawn":record.get_bool("withdrawn").unwrap_or(false),
            "payload":record.get_document("payload").ok(),
            "media_ref":record.get_object_id("media_id").ok().map(|id| id.to_hex()),
            "media_status":record.get_str("media_status").ok()
        }));
    }
    let content = Value::Array(content);
    Ok(Snapshot {
        hash: hash(&content)?,
        content,
        events: records,
        pending,
    })
}

pub async fn active(state: &AppState, inquiry: &Document) -> Result<Option<Document>, AppError> {
    let Ok(id) = inquiry.get_object_id("active_draft_id") else {
        return Ok(None);
    };
    let mut filter = state.scope();
    filter.insert("_id", id);
    state
        .collection("order_drafts")
        .find_one(filter)
        .await
        .map_err(Into::into)
}

pub fn references_withdrawn(draft: &Document, snapshot: &Snapshot) -> bool {
    let refs = draft
        .get_array("source_event_ids")
        .cloned()
        .unwrap_or_default();
    snapshot.events.iter().any(|event| {
        event.get_bool("withdrawn").unwrap_or(false)
            && event
                .get_object_id("_id")
                .is_ok_and(|id| refs.contains(&bson::Bson::ObjectId(id)))
    })
}

pub async fn enqueue(
    state: &AppState,
    id: ObjectId,
    input: PrepareInput,
    manual: bool,
) -> Result<Document, AppError> {
    input.validate()?;
    let input_hash = hash(&json!({"manual":manual, "input":input}))?;
    let mut identity = state.scope();
    identity.extend(doc! {"inquiry_id":id, "request_id":&input.request_id});
    if let Some(existing) = state
        .collection("draft_requests")
        .find_one(identity.clone())
        .await?
    {
        if existing.get_str("input_hash").ok() != Some(&input_hash) {
            return Err(AppError::DraftConflict);
        }
        return Ok(existing);
    }
    let inquiry = inquiry(state, id).await?;
    if version(&inquiry) != input.expected_version
        || inquiry.get_str("status").ok() == Some("CLOSED")
    {
        return Err(AppError::DraftConflict);
    }
    if !manual && (state.config.hermes_url.is_none() || state.config.hermes_api_key.is_none()) {
        return Err(AppError::HermesUnavailable);
    }
    let now = DateTime::now();
    let mut record = identity.clone();
    record.extend(doc! {"_id":ObjectId::new(), "input_hash":input_hash,
    "input":bson::to_document(&input).map_err(|_| AppError::Data)?, "manual":manual,
    "status":"PENDING", "attempts":0_i32, "available_at":now, "created_at":now, "updated_at":now});
    match state
        .collection("draft_requests")
        .insert_one(record.clone())
        .await
    {
        Ok(_) => Ok(record),
        Err(error) if events::duplicate(&error) => {
            let existing = state
                .collection("draft_requests")
                .find_one(identity)
                .await?
                .ok_or(AppError::Data)?;
            if existing.get_str("input_hash").ok() != record.get_str("input_hash").ok() {
                return Err(AppError::DraftConflict);
            }
            Ok(existing)
        }
        Err(error) => Err(error.into()),
    }
}

pub async fn claim(state: &AppState) -> Result<Option<Document>, AppError> {
    let now = DateTime::now();
    let mut filter = state.scope();
    filter.extend(doc! {"status":{"$in":["PENDING", "PROCESSING"]}, "available_at":{"$lte":now}});
    Ok(state.collection("draft_requests").find_one_and_update(filter, doc! {"$set":{
        "status":"PROCESSING", "updated_at":now, "lease_token":ObjectId::new(),
        "available_at":DateTime::from_millis(now.timestamp_millis()+300_000)}, "$inc":{"attempts":1}})
        .sort(doc! {"created_at":1}).return_document(ReturnDocument::After).await?)
}

pub fn lease(state: &AppState, job: &Document) -> Result<Document, AppError> {
    let mut filter = state.scope();
    filter.extend(
        doc! {"_id":job.get_object_id("_id").map_err(|_| AppError::Data)?,
        "lease_token":job.get_object_id("lease_token").map_err(|_| AppError::Data)?},
    );
    Ok(filter)
}

pub async fn finish(
    state: &AppState,
    job: &Document,
    status: &str,
    error: Option<&str>,
) -> Result<(), AppError> {
    let mut fields = doc! {"status":status, "updated_at":DateTime::now()};
    if let Some(error) = error {
        fields.insert("error_code", error);
    }
    state
        .collection("draft_requests")
        .update_one(
            lease(state, job)?,
            doc! {"$set":fields,"$unset":{"lease_token":""}},
        )
        .await?;
    let mut filter = state.scope();
    filter.extend(
        doc! {"_id":job.get_object_id("inquiry_id").map_err(|_| AppError::Data)?,
        "draft_job_id":job.get_object_id("_id").map_err(|_| AppError::Data)?,
        "draft_lease_token":job.get_object_id("lease_token").map_err(|_| AppError::Data)?},
    );
    state
        .collection("inquiries")
        .update_one(
            filter,
            doc! {"$unset":{"draft_job_id":"", "draft_lock_until":"", "draft_lease_token":""}},
        )
        .await?;
    Ok(())
}

pub async fn lock(state: &AppState, job: &Document, expected: i64) -> Result<Document, AppError> {
    let id = job
        .get_object_id("inquiry_id")
        .map_err(|_| AppError::Data)?;
    let job_id = job.get_object_id("_id").map_err(|_| AppError::Data)?;
    let mut filter = state.scope();
    let version_filter = if expected == 0 {
        doc! {"$or":[{"draft_version":0_i64},{"draft_version":{"$exists":false}}]}
    } else {
        doc! {"draft_version":expected}
    };
    filter.extend(doc! {"_id":id, "status":{"$ne":"CLOSED"}, "$and":[version_filter,
        {"$or":[{"draft_job_id":job_id}, {"draft_job_id":{"$exists":false}}, {"draft_lock_until":{"$lte":DateTime::now()}}]}]});
    state
        .collection("inquiries")
        .find_one_and_update(
            filter,
            doc! {"$set":{"draft_job_id":job_id,"draft_lease_token":job.get_object_id("lease_token").map_err(|_| AppError::Data)?,
            "draft_lock_until":DateTime::from_millis(DateTime::now().timestamp_millis()+300_000)}},
        )
        .return_document(ReturnDocument::After)
        .await?
        .ok_or(AppError::DraftConflict)
}

pub async fn commit(
    state: &AppState,
    job: &Document,
    expected: i64,
    draft: Document,
) -> Result<(), AppError> {
    let id = job
        .get_object_id("inquiry_id")
        .map_err(|_| AppError::Data)?;
    let job_id = job.get_object_id("_id").map_err(|_| AppError::Data)?;
    let mut draft_filter = state.scope();
    draft_filter.insert("_id", job_id);
    let expected_hash = draft
        .get_str("context_hash")
        .map_err(|_| AppError::Data)?
        .to_string();
    state
        .collection("order_drafts")
        .update_one(draft_filter.clone(), doc! {"$setOnInsert":draft})
        .upsert(true)
        .await?;
    let stored = state
        .collection("order_drafts")
        .find_one(draft_filter)
        .await?
        .ok_or(AppError::Data)?;
    if stored.get_str("context_hash").ok() != Some(expected_hash.as_str()) {
        return Err(AppError::ContextChanged);
    }
    let workflow = stored
        .get_str("workflow_status")
        .map_err(|_| AppError::Data)?;
    let mut filter = state.scope();
    filter.extend(doc! {"_id":id,"status":{"$ne":"CLOSED"},"draft_job_id":job_id,"draft_lease_token":job.get_object_id("lease_token").map_err(|_|AppError::Data)?});
    if expected == 0 {
        filter.insert(
            "$or",
            vec![
                doc! {"draft_version":0_i64},
                doc! {"draft_version":{"$exists":false}},
            ],
        );
    } else {
        filter.insert("draft_version", expected);
    }
    let result = state.collection("inquiries").update_one(filter,doc! {
        "$set":{"active_draft_id":job_id,"draft_version":expected+1,"status":workflow,"updated_at":DateTime::now()},
        "$push":{"draft_history":job_id}, "$unset":{"draft_job_id":"", "draft_lock_until":"", "draft_lease_token":"", "approval":"", "quote":""}
    }).await?;
    if result.matched_count != 1 {
        return Err(AppError::DraftConflict);
    }
    finish(state, job, "DONE", None).await
}

pub async fn safe_drafts(state: &AppState, inquiry: &Document) -> Result<Vec<Value>, AppError> {
    let snapshot = snapshot(state, inquiry).await?;
    let ids = inquiry
        .get_array("draft_history")
        .cloned()
        .unwrap_or_default();
    let mut filter = state.scope();
    filter.insert("_id", doc! {"$in":ids});
    let records: Vec<Document> = state
        .collection("order_drafts")
        .find(filter)
        .sort(doc! {"draft_version":-1})
        .limit(50)
        .await?
        .try_collect()
        .await?;
    let mut result = Vec::new();
    for mut record in records {
        let stale = record.get_str("context_hash").ok() != Some(&snapshot.hash) || snapshot.pending;
        // Withdrawn evidence must not leak via an old AI or manual revision.
        let redacted = references_withdrawn(&record, &snapshot);
        if redacted {
            for field in ["result", "admin_summary", "confirmed_fields"] {
                record.remove(field);
            }
        }
        let mut value = serde_json::to_value(record).map_err(|_| AppError::Data)?;
        value["stale"] = json!(stale);
        value["redacted"] = json!(redacted);
        result.push(value);
    }
    Ok(result)
}

pub async fn visible_media(
    state: &AppState,
    event: &Document,
) -> Result<Option<(String, Vec<u8>)>, AppError> {
    let Ok(id) = event.get_object_id("media_id") else {
        return Ok(None);
    };
    match media::read(state, id).await {
        Ok(value) => Ok(Some(value)),
        Err(AppError::NotFound) => Err(AppError::Validation(
            "Selected image has expired or was withdrawn",
        )),
        Err(error) => Err(error),
    }
}
