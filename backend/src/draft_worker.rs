use crate::{
    db::drafts,
    error::AppError,
    models::draft::{DraftResult, Evidence, PrepareInput, validate_evidence},
    state::AppState,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use mongodb::bson::{self, DateTime, Document, doc, oid::ObjectId};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use tokio::sync::watch;

const PROMPT: &str = include_str!("order_draft_prompt.txt");

pub async fn process(state: &AppState, job: &Document) -> Result<(), AppError> {
    let job_id = job.get_object_id("_id").map_err(|_| AppError::Data)?;
    let inquiry_id = job
        .get_object_id("inquiry_id")
        .map_err(|_| AppError::Data)?;
    let current = drafts::inquiry(state, inquiry_id).await?;
    // Recover a crash after the atomic pointer update without creating another version.
    if current
        .get_array("draft_history")
        .is_ok_and(|ids| ids.contains(&bson::Bson::ObjectId(job_id)))
    {
        if job.get_str("status").ok() == Some("DONE") {
            return Ok(());
        }
        return drafts::finish(state, job, "DONE", None).await;
    }
    let input: PrepareInput = bson::from_document(
        job.get_document("input")
            .map_err(|_| AppError::Data)?
            .clone(),
    )
    .map_err(|_| AppError::Data)?;
    let inquiry = drafts::lock(state, job, input.expected_version).await?;
    if job.get_i64("summary_revision").unwrap_or(0) != drafts::summary_revision(&inquiry) {
        return Err(AppError::ContextChanged);
    }
    let snapshot = drafts::snapshot(state, &inquiry).await?;
    if snapshot.pending {
        return Err(AppError::DraftWaiting);
    }
    if input
        .context_hash
        .as_ref()
        .is_some_and(|hash| hash != &snapshot.hash)
    {
        return Err(AppError::ContextChanged);
    }
    let mut analysis_filter = state.scope();
    analysis_filter.insert("inquiry_id", inquiry_id);
    let analysis = state
        .collection("analysis_jobs")
        .find_one(analysis_filter.clone())
        .await?;
    if analysis
        .as_ref()
        .and_then(|a| a.get_datetime("ready_at").ok())
        .is_some_and(|at| *at > DateTime::now())
    {
        return Err(AppError::DraftWaiting);
    }
    let previous = drafts::active(state, &inquiry)
        .await?
        .filter(|draft| !drafts::references_withdrawn(draft, &snapshot));
    let mut sources = BTreeMap::new();
    let mut images = BTreeSet::new();
    sources.insert(format!("admin_summary:{job_id}"), input.summary.clone());
    for event in &snapshot.events {
        let id = event
            .get_object_id("_id")
            .map_err(|_| AppError::Data)?
            .to_hex();
        if event.get_bool("withdrawn").unwrap_or(false) {
            continue;
        }
        if let Ok(payload) = event.get_document("payload") {
            let text = payload
                .get_document("message")
                .ok()
                .and_then(|m| m.get_str("text").ok())
                .or_else(|| {
                    payload
                        .get_document("postback")
                        .ok()
                        .and_then(|p| p.get_str("data").ok())
                })
                .unwrap_or("");
            sources.insert(
                format!("{}:{id}", crate::db::oa_sync::source_type(event)),
                text.into(),
            );
        }
    }
    if let Some(previous) = &previous {
        let result = previous
            .get_document("result")
            .map_err(|_| AppError::Data)?;
        sources.insert(
            format!(
                "previous_draft:{}",
                previous.get_object_id("_id").map_err(|_| AppError::Data)?
            ),
            serde_json::to_string(result).map_err(|_| AppError::Data)?,
        );
    }
    let now = DateTime::now();
    let mut summary = state.scope();
    summary.extend(doc! {"_id":job_id,"inquiry_id":inquiry_id,"text":&input.summary,"created_at":now,"updated_at":now});
    state
        .collection("admin_summaries")
        .update_one(doc! {"_id":job_id}, doc! {"$setOnInsert":summary})
        .upsert(true)
        .await?;
    let mut result = if let Some(fields) = &input.fields {
        let text = serde_json::to_string(fields).map_err(|_| AppError::Data)?;
        sources.insert(format!("admin_edit:{job_id}"), text.clone());
        let evidence = fields
            .values()
            .iter()
            .map(|(key, value)| Evidence {
                field: key.clone(),
                source_type: "admin_edit".into(),
                source_ref: job_id.to_hex(),
                quote: value.to_string(),
            })
            .collect();
        DraftResult {
            fields: fields.clone(),
            evidence,
            conflicts: vec![],
            questions: vec![],
        }
    } else {
        let mut parts = vec![json!({"type":"text","text":serde_json::to_string(&json!({
            "inquiry_ref":inquiry_id.to_hex(), "summary_ref":job_id.to_hex(), "admin_summary":input.summary,
            "reference_time_ms":job.get_datetime("created_at").map_err(|_|AppError::Data)?.timestamp_millis(),
            "business_timezone":"Asia/Bangkok", "events":snapshot.content,
            "previous_draft":previous, "selected_image_event_refs":input.image_event_ids,
            "source_texts":sources,
            "fields_schema_example":crate::models::draft::DraftFields::default()
        })).map_err(|_|AppError::Data)?})];
        let mut total_bytes = 0;
        for image_id in &input.image_event_ids {
            let id = ObjectId::parse_str(image_id).map_err(|_| AppError::BadRequest)?;
            let event = snapshot
                .events
                .iter()
                .find(|e| e.get_object_id("_id").ok() == Some(id))
                .ok_or(AppError::Validation(
                    "Image does not belong to this inquiry",
                ))?;
            if event.get_bool("withdrawn").unwrap_or(false) {
                return Err(AppError::ContextChanged);
            }
            let (mime, bytes) = drafts::visible_media(state, event)
                .await?
                .ok_or(AppError::Validation("Image is not ready"))?;
            total_bytes += bytes.len();
            if total_bytes > 8 * 1024 * 1024
                || !["image/jpeg", "image/png", "image/gif", "image/webp"].contains(&mime.as_str())
            {
                return Err(AppError::Validation(
                    "Select at most 8 MB of supported images",
                ));
            }
            let key = format!("line_event:{image_id}");
            images.insert(key.clone());
            sources.insert(key, "selected image".into());
            parts.push(json!({"type":"text","text":format!("Image evidence source: line_event:{image_id}")}));
            parts.push(json!({"type":"image_url","image_url":{"url":format!("data:{mime};base64,{}",STANDARD.encode(bytes))}}));
        }
        serde_json::from_value(ask_hermes_json(state, job_id, inquiry_id, PROMPT, parts).await?)
            .map_err(|_| AppError::HermesFailed)?
    };
    validate_evidence(&result, &sources, &images)?;
    let values = result.fields.values();
    let mut confirmed = BTreeSet::new();
    if let Some(previous) = &previous {
        let prior: DraftResult = bson::from_document(
            previous
                .get_document("result")
                .map_err(|_| AppError::Data)?
                .clone(),
        )
        .map_err(|_| AppError::Data)?;
        let prior_values = prior.fields.values();
        if let Ok(fields) = previous.get_array("confirmed_fields") {
            for key in fields.iter().filter_map(|v| v.as_str()) {
                if prior_values.get(key) == values.get(key) {
                    confirmed.insert(key.to_string());
                } else if !(input.fields.is_some()
                    && input
                        .confirmed_fields
                        .iter()
                        .any(|confirmed| confirmed == key))
                {
                    result.conflicts.push(format!("ข้อมูลที่เคยยืนยันถูกเปลี่ยน: {key}"));
                }
            }
        }
    }
    for key in &input.confirmed_fields {
        if !values.contains_key(key) {
            return Err(AppError::Validation(
                "Cannot confirm an empty or unknown field",
            ));
        }
        confirmed.insert(key.clone());
    }
    let fresh_inquiry = drafts::inquiry(state, inquiry_id).await?;
    let fresh = drafts::snapshot(state, &fresh_inquiry).await?;
    if fresh.pending
        || fresh.hash != snapshot.hash
        || drafts::summary_revision(&fresh_inquiry) != drafts::summary_revision(&inquiry)
    {
        return Err(AppError::ContextChanged);
    }
    // Fence an expired worker: another worker may have reclaimed this request.
    if state
        .collection("draft_requests")
        .find_one(drafts::lease(state, job)?)
        .await?
        .is_none()
    {
        return Err(AppError::DraftConflict);
    }
    let missing = result.fields.missing_fields();
    let workflow = if !result.conflicts.is_empty() {
        "NEEDS_REVIEW"
    } else if !missing.is_empty() {
        "NEEDS_INFORMATION"
    } else {
        "READY_FOR_REVIEW"
    };
    let mut draft = state.scope();
    draft.extend(
        doc! {"_id":job_id,"inquiry_id":inquiry_id,"draft_version":input.expected_version+1,
        "source_event_ids":snapshot.events.iter().filter(|event| !event.get_bool("withdrawn").unwrap_or(false)).filter_map(|event|event.get_object_id("_id").ok()).collect::<Vec<_>>(),
        "workflow_status":workflow,"context_hash":snapshot.hash,"admin_summary":&input.summary,
        "result":bson::to_document(&result).map_err(|_|AppError::Data)?,"missing_fields":missing,
        "confirmed_fields":confirmed.into_iter().collect::<Vec<_>>(),"source":"admin_assisted",
        "summary_revision":drafts::summary_revision(&inquiry),
        "pricing_verified":false,"product_verified":false,"availability_verified":false,
        "created_at":now,"updated_at":now},
    );
    drafts::commit(state, job, input.expected_version, draft).await?;
    if let Some(analysis) = analysis
        && let Ok(updated) = analysis.get_datetime("updated_at")
    {
        analysis_filter.insert("updated_at", *updated);
        state
            .collection("analysis_jobs")
            .update_one(
                analysis_filter,
                doc! {"$set":{"status":"PREPARED","draft_id":job_id}},
            )
            .await?;
    }
    Ok(())
}

pub(crate) async fn ask_hermes_json(
    state: &AppState,
    job: ObjectId,
    inquiry: ObjectId,
    prompt: &str,
    parts: Vec<Value>,
) -> Result<Value, AppError> {
    let endpoint = state
        .config
        .hermes_url
        .as_ref()
        .ok_or(AppError::HermesUnavailable)?;
    let key = state
        .config
        .hermes_api_key
        .as_ref()
        .ok_or(AppError::HermesUnavailable)?;
    let url = reqwest::Url::parse(endpoint).map_err(|_| AppError::HermesUnavailable)?;
    if !["http", "https"].contains(&url.scheme())
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(AppError::HermesUnavailable);
    }
    let namespace = drafts::hash(&json!([
        state.config.shop_id,
        state.config.channel_id,
        inquiry.to_hex(),
        job.to_hex()
    ]))?;
    // A system prompt/tool_choice alone does not disable Hermes' built-in tools.
    // Inspect the extraction profile before admitting any customer data for inference.
    let tools_url = url
        .as_str()
        .strip_suffix("chat/completions")
        .ok_or(AppError::HermesUnavailable)?
        .to_string()
        + "toolsets";
    let tools_response = state
        .http
        .get(tools_url)
        .bearer_auth(key)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|_| AppError::HermesFailed)?;
    if !tools_response.status().is_success() {
        return Err(AppError::HermesToolsEnabled);
    }
    let tools: Value = tools_response
        .json()
        .await
        .map_err(|_| AppError::HermesToolsEnabled)?;
    let toolsets = tools.as_array().ok_or(AppError::HermesToolsEnabled)?;
    if toolsets.iter().any(|toolset| {
        toolset["enabled"] != false
            && toolset["configured"] != false
            && toolset["tools"]
                .as_array()
                .is_none_or(|tools| !tools.is_empty())
    }) {
        return Err(AppError::HermesToolsEnabled);
    }
    let mut response=state.http.post(url).bearer_auth(key)
        .header("Idempotency-Key",format!("draft-{job}"))
        .header("X-Hermes-Session-Key",format!("alex-draft:{namespace}"))
        .timeout(std::time::Duration::from_secs(state.config.hermes_timeout_seconds))
        .json(&json!({"model":state.config.hermes_model,"stream":false,"tools":[],"tool_choice":"none",
            "messages":[{"role":"system","content":prompt},{"role":"user","content":parts}]}))
        .send().await.map_err(|_|AppError::HermesFailed)?;
    if !response.status().is_success() {
        return Err(AppError::HermesFailed);
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| AppError::HermesFailed)? {
        if bytes.len() + chunk.len() > 1024 * 1024 {
            return Err(AppError::HermesFailed);
        }
        bytes.extend(chunk);
    }
    let value: Value = serde_json::from_slice(&bytes).map_err(|_| AppError::HermesFailed)?;
    let choice = &value["choices"][0];
    if choice["finish_reason"] != "stop" || choice["hermes"]["error_code"].is_string() {
        return Err(AppError::HermesFailed);
    }
    let content = choice["message"]["content"]
        .as_str()
        .ok_or(AppError::HermesFailed)?
        .trim();
    let content = content
        .strip_prefix("```json")
        .or_else(|| content.strip_prefix("```"))
        .and_then(|s| s.strip_suffix("```"))
        .unwrap_or(content)
        .trim();
    serde_json::from_str(content).map_err(|_| AppError::HermesFailed)
}

pub async fn run(state: AppState, mut stop: watch::Receiver<bool>) {
    loop {
        if *stop.borrow() {
            break;
        }
        match drafts::claim(&state).await {
            Ok(Some(job)) => {
                if let Err(error) = process(&state, &job).await {
                    if matches!(error, AppError::DraftWaiting)
                        && job.get_datetime("created_at").is_ok_and(|at| {
                            DateTime::now().timestamp_millis() - at.timestamp_millis() < 300_000
                        })
                    {
                        if let Ok(filter) = drafts::lease(&state, &job) {
                            let _ = drafts::finish(&state, &job, "PENDING", None).await;
                            let mut next = filter;
                            next.remove("lease_token");
                            let _=state.collection("draft_requests").update_one(next,doc! {"$set":{"available_at":DateTime::from_millis(DateTime::now().timestamp_millis()+1000)}}).await;
                        }
                        continue;
                    }
                    let status = if matches!(error, AppError::ContextChanged) {
                        "STALE"
                    } else {
                        "FAILED"
                    };
                    if drafts::finish(&state, &job, status, Some(error.code()))
                        .await
                        .is_err()
                    {
                        tracing::error!("Could not finalize draft request; lease will recover it");
                    }
                }
            }
            Ok(None) => (),
            Err(_) => tracing::error!("Draft queue unavailable"),
        }
        tokio::select! {_=tokio::time::sleep(std::time::Duration::from_secs(1))=>(),_=stop.changed()=>break}
    }
}
