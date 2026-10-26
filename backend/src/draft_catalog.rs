use crate::{
    alex,
    db::drafts,
    error::AppError,
    models::{
        draft::{Amounts, DraftItem, DraftResult, Evidence},
        product::{Candidate, SearchInput, SearchMode},
    },
    state::AppState,
};
use mongodb::bson::{self, DateTime, Document, doc, oid::ObjectId};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use tokio::time::{Duration, Instant, timeout_at};

fn question(result: &mut DraftResult, message: String) {
    if result.questions.len() < 20 && !result.questions.contains(&message) {
        result.questions.push(message);
    }
}

fn evidence(result: &mut DraftResult, field: &str, reference: &str, value: Value) {
    result.evidence.retain(|entry| entry.field != field);
    result.evidence.push(Evidence {
        field: field.into(),
        source_type: "alex_catalog".into(),
        source_ref: reference.into(),
        quote: value.to_string(),
    });
}

fn identity_matches(item: &DraftItem, candidate: &Candidate) -> bool {
    if let Some(code) = &item.product_code {
        return code.trim().eq_ignore_ascii_case(&candidate.code);
    }
    item.product_name.as_ref().is_some_and(|name| {
        let name = name.trim().to_lowercase();
        let catalog = candidate.name.to_lowercase();
        !name.is_empty()
            && catalog.match_indices(&name).any(|(start, _)| {
                catalog[..start]
                    .chars()
                    .next_back()
                    .is_none_or(|c| !c.is_alphanumeric())
                    && catalog[start + name.len()..]
                        .chars()
                        .next()
                        .is_none_or(|c| !c.is_alphanumeric())
            })
    })
}

fn options_match(item: &DraftItem, candidate: &Candidate) -> bool {
    [
        (&item.color, &candidate.color),
        (&item.size, &candidate.size),
    ]
    .iter()
    .all(|(wanted, actual)| {
        wanted.as_ref().is_none_or(|wanted| {
            actual
                .as_ref()
                .is_some_and(|actual| wanted.trim().eq_ignore_ascii_case(actual.trim()))
        })
    })
}

async fn candidates(
    state: &AppState,
    session: &alex::Session,
    item: &DraftItem,
    input: &mut SearchInput,
) -> Result<(Vec<Candidate>, bool), AppError> {
    let mut found = BTreeMap::new();
    let mut more;
    loop {
        let (page, has_more) = alex::search_page(state, session, input).await?;
        for candidate in page
            .into_iter()
            .filter(|candidate| identity_matches(item, candidate))
        {
            found.insert(candidate.product_item_id.clone(), candidate);
        }
        more = has_more;
        if !more || input.page >= 5 {
            break;
        }
        input.page += 1;
    }
    // Offer the actual catalog options when OCR color/size differs instead of inventing a match.
    if found.is_empty() && !more && (input.color.is_some() || input.size.is_some()) {
        input.color = None;
        input.size = None;
        input.page = 1;
        loop {
            let (page, has_more) = alex::search_page(state, session, input).await?;
            for candidate in page
                .into_iter()
                .filter(|candidate| identity_matches(item, candidate))
            {
                found.insert(candidate.product_item_id.clone(), candidate);
            }
            more = has_more;
            if !more || input.page >= 5 {
                break;
            }
            input.page += 1;
        }
    }
    Ok((found.into_values().collect(), more))
}

fn fill(result: &mut DraftResult, index: usize, candidate: &Candidate) -> Vec<(String, Value)> {
    let common = result.fields.amounts.clone();
    let item = &mut result.fields.items[index];
    let mut filled = vec![];
    for (key, target, value) in [
        (
            "product_code",
            &mut item.product_code,
            Some(candidate.code.clone()),
        ),
        (
            "product_name",
            &mut item.product_name,
            Some(candidate.name.clone()),
        ),
        ("color", &mut item.color, candidate.color.clone()),
        ("size", &mut item.size, candidate.size.clone()),
    ] {
        if let Some(value) = value {
            *target = Some(value.clone());
            filled.push((format!("items.{index}.{key}"), json!(value)));
        }
    }
    let amounts = item.amounts.get_or_insert_with(Amounts::default);
    for (key, target, common, catalog) in [
        (
            "rental_fee_satang",
            &mut amounts.rental_fee_satang,
            common.rental_fee_satang,
            candidate.rental_fee_satang,
        ),
        (
            "deposit_satang",
            &mut amounts.deposit_satang,
            common.deposit_satang,
            candidate.deposit_satang,
        ),
    ] {
        if target.is_none()
            && common.is_none()
            && let Some(value) = catalog
        {
            *target = Some(value);
            filled.push((format!("items.{index}.amounts.{key}"), json!(value)));
        }
    }
    filled
}

pub async fn enrich(
    state: &AppState,
    job: &Document,
    inquiry: &Document,
    context_hash: &str,
    result: &mut DraftResult,
    sources: &mut BTreeMap<String, String>,
) -> Result<Vec<Document>, AppError> {
    if result.fields.items.is_empty() {
        return Ok(vec![]);
    }
    let deadline = Instant::now() + Duration::from_secs(60);
    let session = match timeout_at(deadline, alex::login(state)).await {
        Ok(Ok(session)) => session,
        failure => {
            let status = if matches!(failure, Ok(Err(AppError::AlexUnavailable))) {
                "NOT_CONFIGURED"
            } else {
                "FAILED"
            };
            question(
                result,
                "ยังค้นสินค้าใน Alex ไม่สำเร็จ กรุณาตรวจการเชื่อมต่อแล้วค้นอีกครั้งในแท็บเลือกชุด".into(),
            );
            return Ok((0..result.fields.items.len())
                .map(|index| doc! {"item_index":index as i64,"status":status})
                .collect());
        }
    };
    let branch = if let Some(wanted) = &result.fields.branch {
        session
            .branches
            .iter()
            .find(|branch| branch.id == *wanted || branch.name.eq_ignore_ascii_case(wanted.trim()))
    } else if session.branches.len() == 1 {
        session.branches.first()
    } else {
        None
    };
    let Some(branch) = branch else {
        question(result, "กรุณาเลือกสาขา Alex ที่ต้องการค้นสินค้าในแท็บเลือกชุด".into());
        return Ok((0..result.fields.items.len())
            .map(|index| doc! {"item_index":index as i64,"status":"BRANCH_REQUIRED"})
            .collect());
    };
    let branch_json = json!({"id":branch.id,"name":branch.name}).to_string();
    sources.insert(format!("alex_catalog:branch-{}", branch.id), branch_json);
    result.fields.branch = Some(branch.name.clone());
    evidence(
        result,
        "branch",
        &format!("branch-{}", branch.id),
        json!(branch.name),
    );
    let draft_id = job.get_object_id("_id").map_err(|_| AppError::Data)?;
    let inquiry_id = inquiry.get_object_id("_id").map_err(|_| AppError::Data)?;
    let mut lookups = vec![];
    for index in 0..result.fields.items.len() {
        let item = result.fields.items[index].clone();
        let query = item
            .product_code
            .clone()
            .or_else(|| item.product_name.clone());
        if query.as_ref().is_none_or(|query| query.trim().is_empty()) {
            question(
                result,
                format!("กรุณาระบุชื่อหรือรหัสสินค้ารายการ {} เพื่อค้นใน Alex", index + 1),
            );
            lookups.push(doc! {"item_index":index as i64,"status":"MISSING_QUERY"});
            continue;
        }
        let mut input = SearchInput {
            draft_version: drafts::version(inquiry) + 1,
            context_hash: context_hash.into(),
            item_index: index,
            branch_id: branch.id.clone(),
            mode: if item.product_code.is_some() {
                SearchMode::Code
            } else {
                SearchMode::Text
            },
            query,
            color: item.color.clone(),
            size: item.size.clone(),
            image_event_id: None,
            category: None,
            texture: None,
            page: 1,
        };
        let (candidates, more) =
            match timeout_at(deadline, candidates(state, &session, &item, &mut input)).await {
                Ok(Ok(value)) => value,
                _ => {
                    question(
                        result,
                        format!(
                            "ค้นสินค้ารายการ {} ใน Alex ไม่สำเร็จ กรุณาค้นอีกครั้งในแท็บเลือกชุด",
                            index + 1
                        ),
                    );
                    lookups.push(doc! {"item_index":index as i64,"status":"FAILED"});
                    continue;
                }
            };
        let selected = if !more && candidates.len() == 1 && options_match(&item, &candidates[0]) {
            candidates.first()
        } else {
            None
        };
        let status = if selected.is_some() {
            "MATCHED"
        } else if candidates.is_empty() && !more {
            "NOT_FOUND"
        } else {
            "NEEDS_SELECTION"
        };
        let search_id = ObjectId::new();
        let now = DateTime::now();
        let mut record = state.scope();
        record.extend(doc! {"_id":search_id,"inquiry_id":inquiry_id,"draft_id":draft_id,
            "draft_version":input.draft_version,"summary_revision":drafts::summary_revision(inquiry),
            "context_hash":context_hash,"input":bson::to_document(&input).map_err(|_|AppError::Data)?,
            "method":"admin_input","has_more":more,"candidates":bson::to_bson(&candidates).map_err(|_|AppError::Data)?,
            "created_at":now,"updated_at":now,"expires_at":DateTime::from_millis(now.timestamp_millis()+1_800_000)});
        state
            .collection("product_searches")
            .insert_one(record)
            .await?;
        let mut lookup =
            doc! {"item_index":index as i64,"status":status,"search_id":search_id.to_hex()};
        if let Some(candidate) = selected {
            sources.insert(
                format!("alex_catalog:{}", candidate.product_item_id),
                serde_json::to_string(candidate).map_err(|_| AppError::Data)?,
            );
            for (field, value) in fill(result, index, candidate) {
                evidence(result, &field, &candidate.product_item_id, value);
            }
            lookup.insert("selected_product_item_id", &candidate.product_item_id);
        } else if status == "NOT_FOUND" {
            question(
                result,
                format!(
                    "ไม่พบสินค้ารายการ {} ใน Alex กรุณาตรวจชื่อ สี และไซซ์ในแท็บเลือกชุด",
                    index + 1
                ),
            );
        } else {
            question(
                result,
                format!(
                    "กรุณาเลือกสินค้ารายการ {} จากผลค้นหา Alex ในแท็บเลือกชุด",
                    index + 1
                ),
            );
        }
        lookups.push(lookup);
    }
    Ok(lookups)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_names_must_match_whole_words_before_autofill() {
        let item = DraftItem {
            product_name: Some("nina".into()),
            ..Default::default()
        };
        let mut candidate = Candidate {
            product_item_id: "id".into(),
            code: "NINA-1".into(),
            name: "Nina dress".into(),
            color: None,
            size: None,
            rental_fee_satang: None,
            deposit_satang: None,
            image_url: None,
            score: None,
        };
        assert!(identity_matches(&item, &candidate));
        candidate.name = "Ninata dress".into();
        assert!(!identity_matches(&item, &candidate));
        candidate.name = "Different dress".into();
        assert!(!identity_matches(&item, &candidate));
    }
}
