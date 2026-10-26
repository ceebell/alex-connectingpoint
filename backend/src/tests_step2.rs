use super::*;
use crate::{
    db::drafts,
    draft_worker,
    models::draft::{DraftFields, PrepareInput},
};
use std::sync::atomic::{AtomicUsize, Ordering};

#[tokio::test]
async fn browser_login_uses_signed_http_only_sessions_and_requires_csrf_header() {
    let mut cfg = config();
    cfg.admin_cookie_secure = true;
    let client = mongodb::Client::with_uri_str(&cfg.mongodb_uri)
        .await
        .unwrap();
    let state = AppState {
        db: client.database(&cfg.mongodb_db),
        config: Arc::new(cfg),
        http: reqwest::Client::new(),
        login_attempts: Arc::new(tokio::sync::Mutex::new((0, std::time::Instant::now()))),
    };
    let app = routes::router(state.clone()).unwrap();
    let body = json!({"username":"admin","password":"test-only-password"}).to_string();
    let denied = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/admin/login")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(body.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
    let invalid = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/admin/login")
                .method("POST")
                .header("content-type", "application/json")
                .header("x-admin-request", "1")
                .body(Body::from("{invalid"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);
    let error: Value =
        serde_json::from_slice(&to_bytes(invalid.into_body(), 1000).await.unwrap()).unwrap();
    assert_eq!(error["error"]["code"], "BAD_REQUEST");
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/admin/login")
                .method("POST")
                .header("content-type", "application/json")
                .header("x-admin-request", "1")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let cookie = response.headers()["set-cookie"]
        .to_str()
        .unwrap()
        .to_string();
    assert!(
        cookie.contains("HttpOnly")
            && cookie.contains("SameSite=Strict")
            && cookie.contains("Secure")
    );
    assert!(!cookie.contains("test-only-password") && !cookie.contains("test-only-admin"));
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/admin/session")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/admin/session")
                .method("DELETE")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/session")
                .header(
                    "cookie",
                    cookie.replace("alex_admin=", "alex_admin=invalid"),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[derive(Clone, Default)]
struct MockHermes {
    mode: Arc<AtomicUsize>,
    calls: Arc<tokio::sync::Mutex<Vec<Value>>>,
    started: Arc<tokio::sync::Notify>,
    resume: Arc<tokio::sync::Notify>,
}

async fn hermes_mock(
    axum::extract::State(mock): axum::extract::State<MockHermes>,
    axum::Json(body): axum::Json<Value>,
) -> axum::Json<Value> {
    mock.calls.lock().await.push(body.clone());
    let data: Value =
        serde_json::from_str(body["messages"][1]["content"][0]["text"].as_str().unwrap()).unwrap();
    let mode = mock.mode.load(Ordering::SeqCst);
    if mode == 3 {
        mock.started.notify_one();
        mock.resume.notified().await;
    }
    let summary = data["admin_summary"].as_str().unwrap();
    let result = json!({"fields":{"customer_name":"คุณเอ","rental":{"pickup_date":if mode==1{"2569-11-15"}else{"2026-11-15"}}},
        "evidence":[{"field":"customer_name","source_type":"admin_summary","source_ref":if mode==2{"wrong-inquiry"}else{data["summary_ref"].as_str().unwrap()},"quote":"คุณเอ"},
            {"field":"rental.pickup_date","source_type":"admin_summary","source_ref":data["summary_ref"],"quote":summary}],"conflicts":[],"questions":["ขอวันคืนและสาขาเพิ่มเติมค่ะ"]});
    axum::Json(
        json!({"choices":[{"finish_reason":"stop","message":{"content":result.to_string()}}]}),
    )
}

async fn run_next(state: &AppState) -> Result<(), crate::error::AppError> {
    let job = drafts::claim(state).await.unwrap().unwrap();
    draft_worker::process(state, &job).await
}

async fn hermes_tools(
    axum::extract::State(mock): axum::extract::State<MockHermes>,
) -> axum::Json<Value> {
    axum::Json(if mock.mode.load(Ordering::SeqCst) == 4 {
        json!([{"enabled":true,"configured":true,"tools":["terminal"]}])
    } else {
        json!([])
    })
}

#[tokio::test]
#[ignore = "requires disposable MongoDB; run cargo test -- --ignored"]
async fn admin_summaries_survive_without_ai_and_reject_stale_or_concurrent_edits() {
    let mut cfg = config();
    cfg.mongodb_uri = std::env::var("TEST_MONGODB_URI").unwrap_or(cfg.mongodb_uri);
    cfg.burst_ms = 0;
    let state = AppState::connect(cfg).await.unwrap();
    crate::db::indexes::ensure(&state).await.unwrap();
    events::persist(&state, &event("summary-start", "summary-customer", 1))
        .await
        .unwrap();
    let incoming = events::claim(&state).await.unwrap().unwrap();
    worker::process(&state, &incoming).await.unwrap();
    let inquiry = all(&state, "inquiries").await[0].clone();
    let id = inquiry.get_object_id("_id").unwrap();
    let path = format!("/api/inquiries/{id}/summary");
    let context = drafts::snapshot(&state, &inquiry).await.unwrap();
    let input = json!({"expected_revision":0,"text":"คุณเอ รับ 15 พ.ย. 2569",
        "image_event_ids":[],"context_hash":context.hash});
    assert_eq!(
        request(&state, &path, "POST", input.clone(), false).await.0,
        StatusCode::UNAUTHORIZED
    );
    let (status, saved) = request(&state, &path, "POST", input.clone(), true).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(saved["revision"], 1);
    assert_eq!(saved["text"], input["text"]);
    assert_eq!(saved["stale"], false);
    assert!(saved["updated_by"].is_string());
    assert!(saved.get("source_event_ids").is_none());
    assert!(all(&state, "draft_requests").await.is_empty());
    assert!(all(&state, "order_drafts").await.is_empty());
    assert!(all(&state, "orders").await.is_empty());
    let (status, reloaded) = request(
        &state,
        &format!("/api/inquiries/{id}/drafts"),
        "GET",
        Value::Null,
        true,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(reloaded["summary"]["text"], input["text"]);
    let fields = serde_json::from_value(json!({"customer_name":"คุณเอ","branch":"A",
        "items":[{"product_code":"DR-012","color":"แดง","size":"M","quantity":1}],
        "rental":{"pickup_date":"2026-11-15","return_date":"2026-11-18","fulfillment_method":"store_pickup"}})).unwrap();
    drafts::enqueue(
        &state,
        id,
        PrepareInput {
            request_id: "manual-from-saved-summary".into(),
            expected_version: 0,
            summary: input["text"].as_str().unwrap().into(),
            image_event_ids: vec![],
            fields: Some(fields),
            confirmed_fields: vec![],
            context_hash: Some(context.hash.clone()),
        },
        true,
    )
    .await
    .unwrap();
    run_next(&state).await.unwrap();
    let ready = drafts::inquiry(&state, id).await.unwrap();
    assert_eq!(
        drafts::safe_drafts(&state, &ready).await.unwrap()[0]["stale"],
        false
    );
    // Compare-and-set permits exactly one writer at a given summary revision.
    let mut edit = input.clone();
    edit["expected_revision"] = json!(1);
    edit["text"] = json!("คุณเอ เปลี่ยนเป็นสีแดง");
    let mut second = edit.clone();
    second["text"] = json!("คุณเอ เปลี่ยนเป็นสีน้ำเงิน");
    let (a, b) = tokio::join!(
        request(&state, &path, "POST", edit, true),
        request(&state, &path, "POST", second, true)
    );
    assert!(
        (a.0 == StatusCode::OK && b.1["error"]["code"] == "SUMMARY_CONFLICT")
            || (b.0 == StatusCode::OK && a.1["error"]["code"] == "SUMMARY_CONFLICT")
    );
    let changed = drafts::inquiry(&state, id).await.unwrap();
    assert_eq!(
        drafts::safe_drafts(&state, &changed).await.unwrap()[0]["stale"],
        true
    );
    let approval = json!({"draft_id":ready.get_object_id("active_draft_id").unwrap().to_hex(),"draft_version":1,"context_hash":context.hash});
    assert_eq!(
        request(
            &state,
            &format!("/api/inquiries/{id}/order"),
            "POST",
            approval,
            true
        )
        .await
        .1["error"]["code"],
        "ORDER_NOT_READY"
    );
    let mut invalid = input.clone();
    invalid["expected_revision"] = json!(2);
    invalid["image_event_ids"] = json!([ObjectId::new().to_hex()]);
    assert_eq!(
        request(&state, &path, "POST", invalid, true).await.0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let mut other = state.clone();
    let mut other_cfg = (*state.config).clone();
    other_cfg.shop_id = "another-shop".into();
    other.config = Arc::new(other_cfg);
    assert_eq!(
        request(&other, &path, "POST", input.clone(), true).await.0,
        StatusCode::NOT_FOUND
    );
    events::persist(&state, &event("summary-new", "summary-customer", 2))
        .await
        .unwrap();
    let mut edit = input.clone();
    edit["expected_revision"] = json!(2);
    assert_eq!(
        request(&state, &path, "POST", edit.clone(), true).await.1["error"]["code"],
        "CONTEXT_CHANGED"
    );
    let incoming = events::claim(&state).await.unwrap().unwrap();
    worker::process(&state, &incoming).await.unwrap();
    let current = drafts::inquiry(&state, id).await.unwrap();
    let snapshot = drafts::snapshot(&state, &current).await.unwrap();
    assert_eq!(
        drafts::safe_summary(&current, &snapshot).unwrap().unwrap()["stale"],
        true
    );
    // Even a configured AI request must not silently adopt context newer than the admin saw.
    let mut configured = state.clone();
    let mut cfg = (*state.config).clone();
    cfg.hermes_url = Some("http://127.0.0.1:9/v1/chat/completions".into());
    cfg.hermes_api_key = Some("test-only-hermes-key".into());
    configured.config = Arc::new(cfg);
    let prepare = json!({"request_id":"old-context","expected_version":1,"summary":input["text"],"context_hash":context.hash});
    let mut prepare = prepare;
    prepare["summary"] = json!(
        current
            .get_document("working_summary")
            .unwrap()
            .get_str("text")
            .unwrap()
    );
    assert_eq!(
        request(
            &configured,
            &format!("/api/inquiries/{id}/prepare"),
            "POST",
            prepare,
            true
        )
        .await
        .1["error"]["code"],
        "CONTEXT_CHANGED"
    );
    // Withdrawn chat cannot be exposed through a separately saved summary.
    media::tombstone(&state, incoming.get_str("message_id").unwrap())
        .await
        .unwrap();
    // The saved summary predates this second message; withdraw its first source as well.
    let first = all(&state, "line_events")
        .await
        .into_iter()
        .find(|event| event.get_str("event_id") == Ok("summary-start"))
        .unwrap();
    media::tombstone(&state, first.get_str("message_id").unwrap())
        .await
        .unwrap();
    let current = drafts::inquiry(&state, id).await.unwrap();
    let snapshot = drafts::snapshot(&state, &current).await.unwrap();
    let redacted = drafts::safe_summary(&current, &snapshot).unwrap().unwrap();
    assert_eq!(redacted["redacted"], true);
    assert!(redacted.get("text").is_none() && redacted.get("image_event_ids").is_none());
    edit["context_hash"] = json!(snapshot.hash);
    edit["text"] = json!("สรุปใหม่หลังตรวจข้อความที่ยกเลิก");
    let (status, fresh) = request(&state, &path, "POST", edit, true).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fresh["revision"], 3);
    assert_eq!(fresh["redacted"], false);
    state.db.drop().await.unwrap();
}

#[tokio::test]
#[ignore = "requires disposable MongoDB; run cargo test -- --ignored"]
async fn drafts_are_versioned_grounded_isolated_and_recoverable() {
    let mock = MockHermes::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = axum::Router::new()
        .route("/v1/toolsets", axum::routing::get(hermes_tools))
        .route("/v1/chat/completions", axum::routing::post(hermes_mock))
        .with_state(mock.clone());
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let mut cfg = config();
    cfg.mongodb_uri = std::env::var("TEST_MONGODB_URI").unwrap_or(cfg.mongodb_uri);
    cfg.hermes_url = Some(format!("http://{address}/v1/chat/completions"));
    cfg.hermes_api_key = Some("test-only-hermes-key".into());
    cfg.burst_ms = 0;
    let state = AppState::connect(cfg).await.unwrap();
    crate::db::indexes::ensure(&state).await.unwrap();
    events::persist(&state, &event("draft-start", "draft-customer", 1))
        .await
        .unwrap();
    let incoming = events::claim(&state).await.unwrap().unwrap();
    worker::process(&state, &incoming).await.unwrap();
    let inquiry = all(&state, "inquiries").await[0].clone();
    let id = inquiry.get_object_id("_id").unwrap();
    let input = PrepareInput {
        request_id: "request-1".into(),
        expected_version: 0,
        summary: "คุณเอ รับ 15 พ.ย. 2569".into(),
        image_event_ids: vec![],
        fields: None,
        confirmed_fields: vec![],
        context_hash: None,
    };
    let job = drafts::enqueue(&state, id, input.clone(), false)
        .await
        .unwrap();
    assert_eq!(
        drafts::enqueue(&state, id, input.clone(), false)
            .await
            .unwrap()
            .get_object_id("_id")
            .unwrap(),
        job.get_object_id("_id").unwrap()
    );
    let mut changed = input.clone();
    changed.summary = "different".into();
    assert!(matches!(
        drafts::enqueue(&state, id, changed, false).await,
        Err(crate::error::AppError::DraftConflict)
    ));
    run_next(&state).await.unwrap();
    let inquiry = drafts::inquiry(&state, id).await.unwrap();
    assert_eq!(drafts::version(&inquiry), 1);
    let saved = drafts::safe_drafts(&state, &inquiry).await.unwrap();
    assert_eq!(
        saved[0]["result"]["fields"]["rental"]["pickup_date"],
        "2026-11-15"
    );
    assert_eq!(saved[0]["stale"], false);
    // Products come from the cart and delivery defaults to Flash, so only the return date is missing.
    assert_eq!(saved[0]["missing_fields"], json!(["rental.return_date"]));
    assert_eq!(
        saved[0]["result"]["fields"]["rental"]["delivery_provider"],
        "Flash"
    );
    // Restart recovery sees the committed pointer and does not call Hermes twice.
    let done = state
        .collection("draft_requests")
        .find_one(doc! {"_id":job.get_object_id("_id").unwrap()})
        .await
        .unwrap()
        .unwrap();
    draft_worker::process(&state, &done).await.unwrap();
    assert_eq!(mock.calls.lock().await.len(), 1);
    let mut manual = input.clone();
    manual.request_id = "manual-2".into();
    manual.expected_version = 1;
    let fields = DraftFields {
        customer_name: Some("คุณเอ".into()),
        ..Default::default()
    };
    manual.fields = Some(fields);
    manual.confirmed_fields = vec!["customer_name".into()];
    drafts::enqueue(&state, id, manual, true).await.unwrap();
    run_next(&state).await.unwrap();
    assert_eq!(
        drafts::version(&drafts::inquiry(&state, id).await.unwrap()),
        2
    );
    assert_eq!(mock.calls.lock().await.len(), 1);
    // Incorrect dates and fabricated provenance never create a new draft.
    for mode in [1, 2, 4] {
        mock.mode.store(mode, Ordering::SeqCst);
        let mut invalid = input.clone();
        invalid.request_id = format!("invalid-{mode}");
        invalid.expected_version = 2;
        drafts::enqueue(&state, id, invalid, false).await.unwrap();
        let job = drafts::claim(&state).await.unwrap().unwrap();
        assert!(matches!(
            draft_worker::process(&state, &job).await,
            Err(crate::error::AppError::Validation(_) | crate::error::AppError::HermesToolsEnabled)
        ));
        drafts::finish(&state, &job, "FAILED", Some("VALIDATION_ERROR"))
            .await
            .unwrap();
        assert_eq!(
            drafts::version(&drafts::inquiry(&state, id).await.unwrap()),
            2
        );
    }
    // Saving a newer admin summary during inference also fences the old result.
    mock.mode.store(3, Ordering::SeqCst);
    let mut summary_race = input.clone();
    summary_race.request_id = "summary-race".into();
    summary_race.expected_version = 2;
    drafts::enqueue(&state, id, summary_race, false)
        .await
        .unwrap();
    let job = drafts::claim(&state).await.unwrap().unwrap();
    let cloned = state.clone();
    let task_job = job.clone();
    let processing = tokio::spawn(async move { draft_worker::process(&cloned, &task_job).await });
    mock.started.notified().await;
    let current = drafts::inquiry(&state, id).await.unwrap();
    let snapshot = drafts::snapshot(&state, &current).await.unwrap();
    drafts::save_summary(
        &state,
        id,
        crate::models::draft::SummaryInput {
            expected_revision: 0,
            text: input.summary.clone(),
            image_event_ids: vec![],
            context_hash: snapshot.hash,
        },
        &crate::models::customer::Actor {
            id: "test-admin".into(),
            name: "Admin".into(),
        },
    )
    .await
    .unwrap();
    mock.resume.notify_one();
    assert!(matches!(
        processing.await.unwrap(),
        Err(crate::error::AppError::ContextChanged)
    ));
    drafts::finish(&state, &job, "STALE", Some("CONTEXT_CHANGED"))
        .await
        .unwrap();
    assert_eq!(
        drafts::version(&drafts::inquiry(&state, id).await.unwrap()),
        2
    );
    // A new event received during inference fences the old context before commit.
    mock.mode.store(3, Ordering::SeqCst);
    let mut race = input.clone();
    race.request_id = "race-3".into();
    race.expected_version = 2;
    drafts::enqueue(&state, id, race, false).await.unwrap();
    let job = drafts::claim(&state).await.unwrap().unwrap();
    let cloned = state.clone();
    let task_job = job.clone();
    let processing = tokio::spawn(async move { draft_worker::process(&cloned, &task_job).await });
    mock.started.notified().await;
    events::persist(&state, &event("new-during-ai", "draft-customer", 2))
        .await
        .unwrap();
    mock.resume.notify_one();
    assert!(matches!(
        processing.await.unwrap(),
        Err(crate::error::AppError::ContextChanged)
    ));
    drafts::finish(&state, &job, "STALE", Some("CONTEXT_CHANGED"))
        .await
        .unwrap();
    let incoming = events::claim(&state).await.unwrap().unwrap();
    worker::process(&state, &incoming).await.unwrap();
    // Cross-shop IDs cannot enqueue/read another shop's inquiry.
    let mut other_cfg = (*state.config).clone();
    other_cfg.shop_id = "other-shop".into();
    let mut other = state.clone();
    other.config = Arc::new(other_cfg);
    assert!(drafts::inquiry(&other, id).await.is_err());
    let ambiguous_id = ObjectId::new();
    let key = event("ambiguous", "draft-customer", 3)
        .conversation_key()
        .unwrap();
    state.collection("line_events").insert_one(doc! {"_id":ambiguous_id,"shop_id":&state.config.shop_id,"channel_id":&state.config.channel_id,
        "conversation_key":key,"event_id":"ambiguous-unassigned","event_at":DateTime::now(),"created_at":DateTime::now(),"updated_at":DateTime::now(),"status":"NEEDS_ADMIN"}).await.unwrap();
    assert!(
        drafts::snapshot(&state, &drafts::inquiry(&state, id).await.unwrap())
            .await
            .unwrap()
            .pending
    );
    state
        .collection("line_events")
        .delete_one(doc! {"_id":ambiguous_id})
        .await
        .unwrap();
    // Withdrawn evidence hides historical text; fresh revisions remain usable.
    let first = all(&state, "line_events")
        .await
        .into_iter()
        .find(|r| r.get_str("event_id") == Ok("draft-start"))
        .unwrap();
    media::tombstone(&state, first.get_str("message_id").unwrap())
        .await
        .unwrap();
    let inquiry = drafts::inquiry(&state, id).await.unwrap();
    let saved = drafts::safe_drafts(&state, &inquiry).await.unwrap();
    assert!(
        saved
            .iter()
            .all(|r| r["redacted"] == true && r.get("result").is_none())
    );
    let mut fresh = input;
    fresh.request_id = "fresh-3".into();
    fresh.expected_version = 2;
    fresh.fields = Some(DraftFields::default());
    drafts::enqueue(&state, id, fresh, true).await.unwrap();
    run_next(&state).await.unwrap();
    let inquiry = drafts::inquiry(&state, id).await.unwrap();
    assert_eq!(
        drafts::safe_drafts(&state, &inquiry).await.unwrap()[0]["redacted"],
        false
    );
    state.db.drop().await.unwrap();
    server.abort();
}
