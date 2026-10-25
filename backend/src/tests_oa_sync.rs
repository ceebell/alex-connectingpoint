use super::*;
use crate::db::{drafts, oa_sync};
use crate::models::oa_sync::OaBatch;

#[tokio::test]
async fn oa_connector_auth_and_dto_checks_happen_before_database_access() {
    let cfg = config();
    let client = mongodb::Client::with_uri_str(&cfg.mongodb_uri)
        .await
        .unwrap();
    let state = AppState {
        db: client.database(&cfg.mongodb_db),
        config: Arc::new(cfg),
        http: reqwest::Client::new(),
        login_attempts: Arc::new(tokio::sync::Mutex::new((0, std::time::Instant::now()))),
    };
    let path = "/api/connectors/line-oa/messages";
    assert_eq!(
        request(&state, path, "POST", json!({}), false).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        request(&state, path, "POST", json!({}), true).await.0,
        StatusCode::BAD_REQUEST
    );
    let invalid = json!({"bot_id":"wrong-bot","chat_id":"chat","anchor_message_ids":["known"],"messages":[{"id":"msg","text":"fixture","sent_at_ms":1000}]});
    assert_eq!(
        request(&state, path, "POST", invalid, true).await.0,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
#[ignore = "requires disposable MongoDB; run cargo test -- --ignored"]
async fn oa_sync_uses_webhook_anchors_deduplicates_and_preserves_context_pagination() {
    let mut cfg = config();
    cfg.mongodb_uri = std::env::var("TEST_MONGODB_URI").unwrap_or(cfg.mongodb_uri);
    let state = AppState::connect(cfg).await.unwrap();
    crate::db::indexes::ensure(&state).await.unwrap();
    events::persist(&state, &event("anchor-before", "oa-customer", 1000))
        .await
        .unwrap();
    events::persist(&state, &event("anchor-after", "oa-customer", 3000))
        .await
        .unwrap();
    drain(&state).await;
    let inquiry = all(&state, "inquiries").await.remove(0);
    let id = inquiry.get_object_id("_id").unwrap();
    let old = drafts::snapshot(&state, &inquiry).await.unwrap();
    let path = "/api/connectors/line-oa/messages";
    let body = json!({"bot_id":"test-destination","chat_id":"oa-chat","anchor_message_ids":["1001000"],"messages":[{"id":"outgoing-1","text":"Shop reply fixture","sent_at_ms":2000}]});
    let (status, saved) = request(&state, path, "POST", body.clone(), true).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(saved["inserted"], 1);
    assert_eq!(
        request(&state, path, "POST", body.clone(), true).await.1["duplicates"],
        1
    );
    assert_eq!(all(&state, "oa_messages").await.len(), 1);
    assert_eq!(all(&state, "line_events").await.len(), 2);
    let mut conflict = body.clone();
    conflict["messages"][0]["text"] = json!("changed");
    assert_eq!(
        request(&state, path, "POST", conflict, true).await.1["error"]["code"],
        "OA_MESSAGE_CONFLICT"
    );
    let mut missing = body.clone();
    missing["anchor_message_ids"] = json!(["not-in-webhook"]);
    assert_eq!(
        request(&state, path, "POST", missing, true).await.1["error"]["code"],
        "OA_UNLINKED"
    );
    let mut early = body.clone();
    early["messages"][0]["sent_at_ms"] = json!(0);
    assert_eq!(
        request(&state, path, "POST", early, true).await.0,
        StatusCode::CONFLICT
    );
    let mut other = state.clone();
    let mut other_cfg = (*state.config).clone();
    other_cfg.shop_id = "other-shop".into();
    other.config = Arc::new(other_cfg);
    assert_eq!(
        request(&other, path, "POST", body, true).await.1["error"]["code"],
        "OA_UNLINKED"
    );
    let current = drafts::snapshot(&state, &inquiry).await.unwrap();
    assert_ne!(old.hash, current.hash);
    assert_eq!(current.content[1]["sender_role"], "admin");
    assert_eq!(current.content[1]["source_type"], "oa_message");
    let route = format!("/api/inquiries/{id}/context");
    let (_, context) = request(&state, &route, "GET", Value::Null, true).await;
    let messages: Vec<&Value> = context["batches"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|b| b.as_array().unwrap())
        .collect();
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[1]["source"], "oa_manager_browser");
    assert!(messages[1].get("content_hash").is_none());
    let records:Vec<Document>=(4..=201).map(|second|{let mut d=state.scope();let now=DateTime::now();d.extend(doc! {"_id":ObjectId::new(),"inquiry_id":id,"event_id":format!("page-{second}"),"event_at":DateTime::from_millis(second*1000),"status":"DONE","created_at":now,"updated_at":now});d}).collect();
    state
        .collection("line_events")
        .insert_many(records)
        .await
        .unwrap();
    let boundary:OaBatch=serde_json::from_value(json!({"bot_id":"test-destination","chat_id":"oa-chat","anchor_message_ids":["1001000"],"messages":[{"id":"boundary","text":"Boundary reply","sent_at_ms":199500}]})).unwrap();
    oa_sync::ingest(&state, boundary).await.unwrap();
    let (_, page) = request(&state, &route, "GET", Value::Null, true).await;
    let items: Vec<&Value> = page["batches"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|b| b.as_array().unwrap())
        .collect();
    assert_eq!(items.len(), 200);
    assert_eq!(items.last().unwrap()["source"], "oa_manager_browser");
    let cursor = page["next"].as_str().unwrap();
    let (_, last) = request(
        &state,
        &format!("{route}?after={cursor}"),
        "GET",
        Value::Null,
        true,
    )
    .await;
    assert!(last["next"].is_null());
    assert_eq!(
        last["batches"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| b.as_array().unwrap().len())
            .sum::<usize>(),
        2
    );
    let (_, status) = request(&state, "/api/line/oa-sync/status", "GET", Value::Null, true).await;
    assert_eq!(status["state"], "NOT_CONFIGURED");
    request(
        &state,
        "/api/connectors/line-oa/status",
        "POST",
        json!({"state":"NEEDS_LOGIN","synced":0,"pending":0}),
        true,
    )
    .await;
    assert_eq!(
        request(&state, "/api/line/oa-sync/status", "GET", Value::Null, true)
            .await
            .1["state"],
        "NEEDS_LOGIN"
    );
    state.db.drop().await.unwrap();
}
