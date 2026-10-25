use super::*;
use crate::{
    db::drafts,
    draft_worker,
    models::draft::{DraftFields, DraftItem, PrepareInput, Rental},
};

#[tokio::test]
async fn profile_and_order_routes_check_auth_and_input_before_database_access() {
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
    let profile = "/api/line/users/Uvalid/profile";
    assert_eq!(
        request(
            &state,
            profile,
            "POST",
            json!({"expected_version":0}),
            false
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    for body in [json!({"expected_version":0,"extra":1}), json!({})] {
        assert_eq!(
            request(&state, profile, "POST", body, true).await.0,
            StatusCode::BAD_REQUEST
        );
    }
    assert_eq!(
        request(
            &state,
            "/api/line/users/bad.id/profile",
            "POST",
            json!({"expected_version":0}),
            true
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let (status, error) = request(
        &state,
        profile,
        "POST",
        json!({"expected_version":0,"phone":"call me"}),
        true,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(error["error"]["code"], "VALIDATION_ERROR");
    let order = format!("/api/inquiries/{}/order", ObjectId::new().to_hex());
    assert_eq!(
        request(&state, &order, "POST", json!({"draft_id":"x"}), true)
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    let short_hash =
        json!({"draft_id":ObjectId::new().to_hex(),"draft_version":1,"context_hash":"abc"});
    assert_eq!(
        request(&state, &order, "POST", short_hash, true).await.0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

async fn manual_draft(state: &AppState, id: ObjectId, version: i64, fields: DraftFields) {
    let input = PrepareInput {
        request_id: format!("manual-{version}"),
        expected_version: version,
        summary: String::new(),
        image_event_ids: vec![],
        fields: Some(fields),
        confirmed_fields: vec![],
        context_hash: None,
    };
    drafts::enqueue(state, id, input, true).await.unwrap();
    let job = drafts::claim(state).await.unwrap().unwrap();
    draft_worker::process(state, &job).await.unwrap();
}

#[tokio::test]
#[ignore = "requires disposable MongoDB; run cargo test -- --ignored"]
async fn mongo_customer_profiles_and_order_approval() {
    let mut cfg = config();
    cfg.mongodb_uri = std::env::var("TEST_MONGODB_URI").unwrap_or(cfg.mongodb_uri);
    cfg.burst_ms = 0;
    let state = AppState::connect(cfg).await.unwrap();
    crate::db::indexes::ensure(&state).await.unwrap();
    events::persist(&state, &event("order-start", "Ucustomer", 1))
        .await
        .unwrap();
    drain(&state).await;
    let inquiry = all(&state, "inquiries").await.remove(0);
    let id = inquiry.get_object_id("_id").unwrap();

    // Profiles: only known LINE users, optimistic versions, trimmed values.
    let path = "/api/line/users/Ucustomer/profile";
    let (status, empty) = request(&state, path, "GET", json!(null), true).await;
    assert_eq!(status, StatusCode::OK);
    assert!(empty["profile"].is_null());
    let first = json!({"expected_version":0,"display_name":" คุณเอ ","phone":"081-234-5678","notes":"ลูกค้าประจำ"});
    let (status, saved) = request(&state, path, "POST", first.clone(), true).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(saved["profile"]["version"], 1);
    assert_eq!(saved["profile"]["display_name"], "คุณเอ");
    assert_eq!(saved["profile"]["updated_by"], "internal-api");
    let (status, error) = request(&state, path, "POST", first, true).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error["error"]["code"], "RECORD_CONFLICT");
    let (status, saved) = request(
        &state,
        path,
        "POST",
        json!({"expected_version":1,"address":"กรุงเทพฯ"}),
        true,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(saved["profile"]["version"], 2);
    assert!(saved["profile"]["phone"].is_null());
    assert_eq!(
        request(
            &state,
            path,
            "POST",
            json!({"expected_version":1,"notes":"stale tab"}),
            true
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        request(
            &state,
            "/api/line/users/Unknown/profile",
            "POST",
            json!({"expected_version":0}),
            true
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(all(&state, "customer_profiles").await.len(), 1);

    // Orders: only the latest complete draft on the current context.
    let order = format!("/api/inquiries/{}/order", id.to_hex());
    let (status, error) = request(&state, &order, "POST", json!({"draft_id":ObjectId::new().to_hex(),"draft_version":1,"context_hash":"0".repeat(64)}), true).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error["error"]["code"], "ORDER_NOT_READY");
    let partial = DraftFields {
        customer_name: Some("คุณเอ".into()),
        ..Default::default()
    };
    manual_draft(&state, id, 0, partial).await;
    let complete = DraftFields {
        customer_name: Some("คุณเอ".into()),
        branch: Some("สาขา A".into()),
        items: vec![DraftItem {
            product_code: Some("DR-012".into()),
            color: Some("แดง".into()),
            size: Some("M".into()),
            quantity: Some(1),
        }],
        rental: Rental {
            usage_date: None,
            pickup_date: Some("2026-11-15".into()),
            return_date: Some("2026-11-18".into()),
            fulfillment_method: Some("store_pickup".into()),
        },
        ..Default::default()
    };
    let drafts_path = format!("/api/inquiries/{}/drafts", id.to_hex());
    let (_, list) = request(&state, &drafts_path, "GET", json!(null), true).await;
    let partial_id = list["drafts"][0]["_id"]["$oid"]
        .as_str()
        .unwrap()
        .to_string();
    let hash = list["context_hash"].as_str().unwrap().to_string();
    let (status, error) = request(
        &state,
        &order,
        "POST",
        json!({"draft_id":partial_id,"draft_version":1,"context_hash":hash}),
        true,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error["error"]["code"], "ORDER_NOT_READY");
    manual_draft(&state, id, 1, complete).await;
    let (_, list) = request(&state, &drafts_path, "GET", json!(null), true).await;
    assert_eq!(list["drafts"][0]["workflow_status"], "READY_FOR_REVIEW");
    let draft_id = list["drafts"][0]["_id"]["$oid"]
        .as_str()
        .unwrap()
        .to_string();
    let wrong = json!({"draft_id":draft_id,"draft_version":2,"context_hash":"f".repeat(64)});
    let (status, error) = request(&state, &order, "POST", wrong, true).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error["error"]["code"], "CONTEXT_CHANGED");
    let approve = json!({"draft_id":draft_id,"draft_version":2,"context_hash":hash});
    let (status, created) = request(&state, &order, "POST", approve.clone(), true).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["order"]["status"], "CONFIRMED");
    assert_eq!(created["order"]["approved_by"], "internal-api");
    assert_eq!(created["order"]["line_user_id"], "Ucustomer");
    assert_eq!(
        created["order"]["fields"]["items"][0]["product_code"],
        "DR-012"
    );
    assert_eq!(created["order"]["pricing_verified"], false);
    let (status, again) = request(&state, &order, "POST", approve, true).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(again["order"]["_id"], created["order"]["_id"]);
    let (status, error) = request(
        &state,
        &order,
        "POST",
        json!({"draft_id":partial_id,"draft_version":1,"context_hash":hash}),
        true,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error["error"]["code"], "ORDER_EXISTS");
    assert_eq!(all(&state, "orders").await.len(), 1);
    let (_, fetched) = request(&state, &order, "GET", json!(null), true).await;
    assert_eq!(fetched["order"]["_id"], created["order"]["_id"]);
    let inquiry = drafts::inquiry(&state, id).await.unwrap();
    assert_eq!(
        inquiry.get_object_id("order_id").unwrap().to_hex(),
        created["order"]["_id"]["$oid"].as_str().unwrap()
    );
    state.db.drop().await.unwrap();
}
