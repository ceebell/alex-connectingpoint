use super::*;
use crate::{
    config::AlexConfig,
    db::{drafts, products},
    draft_worker,
    models::draft::{DraftFields, PrepareInput},
};
use std::sync::atomic::{AtomicUsize, Ordering};

fn catalog_config(base: &str) -> AlexConfig {
    AlexConfig {
        api_url: base.into(),
        username: "fixture-user".into(),
        password: "fixture-password".into(),
        shop_info_id: "alex-shop".into(),
        branch_ids: vec!["branch-a".into()],
        product_url_prefix: Some("https://catalog.test/products/".into()),
        image_origins: vec![],
    }
}

#[test]
fn product_urls_are_parsed_without_fetching_arbitrary_urls() {
    let cfg = catalog_config("https://catalog.test");
    assert_eq!(
        crate::alex::product_code_from_url(&cfg, "https://catalog.test/products/DR-012").unwrap(),
        "DR-012"
    );
    assert_eq!(
        crate::alex::product_code_from_url(&cfg, "https://catalog.test/products/view?code=DR-012")
            .unwrap(),
        "DR-012"
    );
    for value in [
        "https://catalog.test.evil/products/DR-012",
        "https://evil.test/products/DR-012",
        "https://catalog.test@evil.test/products/DR-012",
        "https://catalog.test/products/../../DR-012",
        "https://catalog.test/products/view?code=A&productCode=B",
        "file:///products/DR-012",
        "https://catalog.test/products/DR-012#other",
        "https://catalog.test/products/%2Fetc%2Fpasswd",
    ] {
        assert!(
            crate::alex::product_code_from_url(&cfg, value).is_err(),
            "{value}"
        );
    }
}

async fn mcp_request(
    state: &AppState,
    method: &str,
    body: Value,
    origin: Option<&str>,
) -> axum::response::Response {
    let mut builder = Request::builder()
        .uri("/api/admin/alex/mcp")
        .method(method)
        .header(
            "authorization",
            "Bearer fixture-mcp-token-at-least-32-characters",
        )
        .header("content-type", "application/json");
    if let Some(origin) = origin {
        builder = builder.header("origin", origin);
    }
    routes::router(state.clone())
        .unwrap()
        .oneshot(builder.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap()
}

#[tokio::test]
async fn mcp_is_authenticated_read_only_and_supports_json_rpc_lifecycle() {
    let mut cfg = config();
    cfg.alex_mcp_token = Some("fixture-mcp-token-at-least-32-characters".into());
    let client = mongodb::Client::with_uri_str(&cfg.mongodb_uri)
        .await
        .unwrap();
    let state = AppState {
        db: client.database(&cfg.mongodb_db),
        config: Arc::new(cfg),
        http: reqwest::Client::new(),
        login_attempts: Arc::new(tokio::sync::Mutex::new((0, std::time::Instant::now()))),
    };
    let body = json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"fixture","version":"1"}}});
    assert_eq!(
        request(&state, "/api/admin/alex/mcp", "POST", body.clone(), true)
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        mcp_request(&state, "POST", body.clone(), Some("https://evil.test"))
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    let response = mcp_request(&state, "POST", body, None).await;
    let result: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 10000).await.unwrap()).unwrap();
    assert_eq!(result["result"]["protocolVersion"], "2025-11-25");
    let response = mcp_request(
        &state,
        "POST",
        json!({"jsonrpc":"2.0","id":"list","method":"tools/list"}),
        None,
    )
    .await;
    let result: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 20000).await.unwrap()).unwrap();
    let tools = result["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 2);
    assert!(
        tools
            .iter()
            .all(|tool| tool["annotations"]["readOnlyHint"] == true)
    );
    assert_eq!(
        mcp_request(
            &state,
            "POST",
            json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
            None
        )
        .await
        .status(),
        StatusCode::ACCEPTED
    );
    assert_eq!(
        mcp_request(&state, "GET", Value::Null, None).await.status(),
        StatusCode::METHOD_NOT_ALLOWED
    );
    let response=mcp_request(&state,"POST",json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"confirm_product","arguments":{}}}),None).await;
    let result: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 1000).await.unwrap()).unwrap();
    assert_eq!(result["error"]["code"], -32602);
}

#[derive(Clone, Default)]
struct CatalogMock {
    mode: Arc<AtomicUsize>,
    calls: Arc<tokio::sync::Mutex<Vec<Value>>>,
}

async fn login_mock(
    axum::extract::State(mock): axum::extract::State<CatalogMock>,
    Json(body): Json<Value>,
) -> Json<Value> {
    assert_eq!(
        body,
        json!({"username":"fixture-user","password":"fixture-password"})
    );
    Json(
        json!({"shopInfoId":if mock.mode.load(Ordering::SeqCst)==1{"wrong-shop"}else{"alex-shop"},"token":"fixture-alex-token",
        "userBranch":[{"branchId":"branch-a","branchName":"สาขา A"},{"branchId":"branch-b","branchName":"สาขา B"}]}),
    )
}

async fn catalog_mock(
    axum::extract::State(mock): axum::extract::State<CatalogMock>,
    headers: axum::http::HeaderMap,
    Json(body): Json<Value>,
) -> axum::response::Response {
    assert_eq!(headers["authorization"], "Bearer fixture-alex-token");
    mock.calls.lock().await.push(body.clone());
    let mode = mock.mode.load(Ordering::SeqCst);
    if mode == 3 {
        return Json(json!({"itemList":"invalid"})).into_response();
    }
    if mode == 5 {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    let item = |id: &str, code: &str, color: &str, size: &str| {
        json!({"productItemId":id,"code":code,"name":"ชุดทดสอบ","color":color,"size":size,
        "rentalPrice":if mode==4{1600}else{1500},"bail":"2000.01","mainImage":if mode==8{""}else{"/photo.png"}})
    };
    if body["keyword"] == "page-test" {
        let items: Vec<_> = if body["pageNumber"] == 1 {
            (0..15)
                .map(|n| item(&format!("page-{n}"), &format!("PAGE-{n}"), "R", "M"))
                .collect()
        } else {
            vec![item("page-last", "PAGE-LAST", "R", "M")]
        };
        return Json(json!({"itemNumber":16,"itemList":items})).into_response();
    }
    let mut items = vec![
        item("item-1", "DR-012", "R", "M"),
        item("item-2", "DR-012", "B", "L"),
        item("item-extra", "DR-012-A", "R", "M"),
    ];
    if mode == 2 {
        items[0]["branchId"] = json!("outside-branch");
    }
    Json(json!({"itemNumber":3,"itemList":items})).into_response()
}

const PNG: &str =
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+yZf8AAAAASUVORK5CYII=";
async fn photo_mock() -> axum::response::Response {
    (
        [(axum::http::header::CONTENT_TYPE, "image/png")],
        STANDARD.decode(PNG).unwrap(),
    )
        .into_response()
}
async fn vision_mock(
    axum::extract::State(mock): axum::extract::State<CatalogMock>,
    Json(body): Json<Value>,
) -> Json<Value> {
    assert_eq!(body["tool_choice"], "none");
    let mode = mock.mode.load(Ordering::SeqCst);
    let result = json!({"read_product_code":if mode==7{json!("DR-012")}else{Value::Null},
        "candidates":[{"product_item_id":if mode==6{"fabricated-id"}else{"item-1"},"score":0.9}]});
    Json(json!({"choices":[{"finish_reason":"stop","message":{"content":result.to_string()}}]}))
}

use axum::{Json, response::IntoResponse};

async fn manual(state: &AppState, id: ObjectId, version: i64, fields: DraftFields) {
    drafts::enqueue(
        state,
        id,
        PrepareInput {
            request_id: format!("catalog-draft-{version}"),
            expected_version: version,
            summary: String::new(),
            image_event_ids: vec![],
            fields: Some(fields),
            confirmed_fields: vec![],
            context_hash: None,
        },
        true,
    )
    .await
    .unwrap();
    let job = drafts::claim(state).await.unwrap().unwrap();
    draft_worker::process(state, &job).await.unwrap();
}

#[tokio::test]
#[ignore = "requires disposable MongoDB; run cargo test -- --ignored"]
async fn product_matching_uses_real_ids_scopes_candidates_and_fences_old_drafts() {
    use futures_util::io::AsyncWriteExt;
    let mock = CatalogMock::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = axum::Router::new()
        .route("/api/user/login", axum::routing::post(login_mock))
        .route("/api/v2/GetProductItems", axum::routing::post(catalog_mock))
        .route("/photo.png", axum::routing::get(photo_mock))
        .route(
            "/v1/toolsets",
            axum::routing::get(|| async { Json(json!([])) }),
        )
        .route("/v1/chat/completions", axum::routing::post(vision_mock))
        .with_state(mock.clone());
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let mut cfg = config();
    cfg.mongodb_uri = std::env::var("TEST_MONGODB_URI").unwrap_or(cfg.mongodb_uri);
    cfg.burst_ms = 0;
    cfg.alex = Some(catalog_config(&format!("http://{address}")));
    cfg.hermes_url = Some(format!("http://{address}/v1/chat/completions"));
    cfg.hermes_api_key = Some("fixture-hermes-key".into());
    let state = AppState::connect(cfg).await.unwrap();
    crate::db::indexes::ensure(&state).await.unwrap();
    events::persist(&state, &event("catalog-start", "catalog-customer", 1))
        .await
        .unwrap();
    let incoming = events::claim(&state).await.unwrap().unwrap();
    worker::process(&state, &incoming).await.unwrap();
    let inquiry = all(&state, "inquiries").await[0].clone();
    let id = inquiry.get_object_id("_id").unwrap();
    let fields:DraftFields=serde_json::from_value(json!({"customer_name":"คุณเอ","branch":"A","items":[
        {"product_code":"DR-012","color":"R","size":"M","quantity":1},{"product_code":"DR-012","color":"B","size":"L","quantity":1}]})).unwrap();
    manual(&state, id, 0, fields.clone()).await;
    let snapshot = drafts::snapshot(&state, &drafts::inquiry(&state, id).await.unwrap())
        .await
        .unwrap();
    let search_path = format!("/api/inquiries/{id}/products/search");
    let confirm_path = format!("/api/inquiries/{id}/products/confirm");
    let input = json!({"draft_version":1,"context_hash":snapshot.hash,"item_index":0,"branch_id":"branch-a","mode":"code","query":"DR-012"});
    let (status, result) = request(&state, &search_path, "POST", input.clone(), true).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(result["candidates"].as_array().unwrap().len(), 2);
    assert_eq!(result["candidates"][0]["rental_fee_satang"], 150000);
    assert_eq!(result["candidates"][0]["deposit_satang"], 200001);
    assert!(
        result["candidates"][0]["image_url"]
            .as_str()
            .unwrap()
            .starts_with("/api/inquiries/")
    );
    let body = mock.calls.lock().await[0].clone();
    assert_eq!(body["pageSize"], 15);
    assert_eq!(body["showNotAvailable"], false);
    let invalid = json!({"search_id":result["search_id"],"product_item_id":"fabricated-id","expected_revision":0});
    assert_eq!(
        request(&state, &confirm_path, "POST", invalid, true)
            .await
            .0,
        StatusCode::CONFLICT
    );
    let mut confirmation =
        json!({"search_id":result["search_id"],"product_item_id":"item-2","expected_revision":0});
    assert_eq!(
        request(&state, &confirm_path, "POST", confirmation.clone(), true)
            .await
            .1["error"]["code"],
        "PRODUCT_OPTIONS_DIFFER"
    );
    confirmation["product_item_id"] = json!("item-1");
    mock.mode.store(4, Ordering::SeqCst);
    assert_eq!(
        request(&state, &confirm_path, "POST", confirmation.clone(), true)
            .await
            .0,
        StatusCode::CONFLICT
    );
    mock.mode.store(0, Ordering::SeqCst);
    assert_eq!(
        request(&state, &confirm_path, "POST", confirmation.clone(), true)
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        products::list(&state, id).await.unwrap()["product_verified"],
        false
    );
    assert_eq!(
        request(&state, &confirm_path, "POST", confirmation, true)
            .await
            .0,
        StatusCode::CONFLICT
    );
    let mut second = input.clone();
    second["item_index"] = json!(1);
    let result2 = request(&state, &search_path, "POST", second, true).await.1;
    assert_eq!(request(&state,&confirm_path,"POST",json!({"search_id":result2["search_id"],"product_item_id":"item-2","expected_revision":0}),true).await.0,StatusCode::OK);
    let verified = products::list(&state, id).await.unwrap();
    assert_eq!(verified["product_verified"], true);
    assert_eq!(verified["availability_verified"], false);
    assert!(all(&state, "orders").await.is_empty());
    let mut other = state.clone();
    let mut other_cfg = (*state.config).clone();
    other_cfg.shop_id = "another-shop".into();
    other.config = Arc::new(other_cfg);
    assert_eq!(
        request(&other, &search_path, "POST", input.clone(), true)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    let mut denied = input.clone();
    denied["branch_id"] = json!("branch-b");
    assert_eq!(
        request(&state, &search_path, "POST", denied, true).await.0,
        StatusCode::FORBIDDEN
    );
    for (mode, status) in [
        (1, StatusCode::FORBIDDEN),
        (2, StatusCode::FORBIDDEN),
        (3, StatusCode::BAD_GATEWAY),
        (5, StatusCode::BAD_GATEWAY),
    ] {
        mock.mode.store(mode, Ordering::SeqCst);
        assert_eq!(
            request(&state, &search_path, "POST", input.clone(), true)
                .await
                .0,
            status
        );
    }
    mock.mode.store(0, Ordering::SeqCst);
    let mut page = input.clone();
    page["mode"] = json!("text");
    page["query"] = json!("page-test");
    assert_eq!(
        request(&state, &search_path, "POST", page.clone(), true)
            .await
            .1["has_more"],
        true
    );
    page["page"] = json!(2);
    assert_eq!(
        request(&state, &search_path, "POST", page, true).await.1["has_more"],
        false
    );
    // A new draft invalidates old selections even if the free-text fields are unchanged.
    manual(&state, id, 1, fields.clone()).await;
    assert_eq!(
        products::list(&state, id).await.unwrap()["matches"],
        json!([])
    );
    let image_event = ObjectId::new();
    let now = DateTime::now();
    let bucket = state.db.gridfs_bucket(None);
    let mut upload=bucket.open_upload_stream("fixture.png").metadata(doc! {"shop_id":&state.config.shop_id,"channel_id":&state.config.channel_id,
        "message_id":"catalog-photo","content_type":"image/png","expires_at":DateTime::from_millis(now.timestamp_millis()+60_000)}).await.unwrap();
    let media_id = upload.id().as_object_id().unwrap();
    upload
        .write_all(&STANDARD.decode(PNG).unwrap())
        .await
        .unwrap();
    upload.close().await.unwrap();
    let mut image_record = state.scope();
    image_record.extend(doc! {"_id":image_event,"inquiry_id":id,"event_id":"catalog-image","event_at":now,"created_at":now,"updated_at":now,
        "status":"DONE","media_id":media_id,"message_id":"catalog-photo","payload":{"type":"message","message":{"type":"image","id":"catalog-photo"}}});
    state
        .collection("line_events")
        .insert_one(image_record)
        .await
        .unwrap();
    manual(&state, id, 2, fields).await;
    let hash = drafts::snapshot(&state, &drafts::inquiry(&state, id).await.unwrap())
        .await
        .unwrap()
        .hash;
    let image = json!({"draft_version":3,"context_hash":hash,"item_index":0,"branch_id":"branch-a","mode":"image","image_event_id":image_event.to_hex()});
    let result = request(&state, &search_path, "POST", image.clone(), true).await;
    assert_eq!(result.0, StatusCode::OK);
    assert_eq!(result.1["method"], "image_similarity");
    assert_eq!(result.1["candidates"][0]["product_item_id"], "item-1");
    mock.mode.store(6, Ordering::SeqCst);
    assert_eq!(
        request(&state, &search_path, "POST", image.clone(), true)
            .await
            .1["error"]["code"],
        "HERMES_FAILED"
    );
    mock.mode.store(7, Ordering::SeqCst);
    assert_eq!(
        request(&state, &search_path, "POST", image.clone(), true)
            .await
            .1["method"],
        "image_code"
    );
    mock.mode.store(8, Ordering::SeqCst);
    assert_eq!(
        request(&state, &search_path, "POST", image.clone(), true)
            .await
            .1["error"]["code"],
        "CATALOG_IMAGES_UNAVAILABLE"
    );
    media::tombstone(&state, "catalog-photo").await.unwrap();
    assert_eq!(
        request(&state, &search_path, "POST", image, true).await.1["error"]["code"],
        "CONTEXT_CHANGED"
    );
    state.db.drop().await.unwrap();
    server.abort();
}
