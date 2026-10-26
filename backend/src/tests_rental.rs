use super::*;
use crate::{
    customer_extract,
    db::{checkout, drafts},
    draft_worker,
    models::draft::{DraftFields, PrepareInput, SummaryInput},
};
use axum::{Json, response::IntoResponse};
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn image_only_requests_accept_five_and_keep_per_item_dates() {
    let mut input = PrepareInput {
        request_id: "photos-only".into(),
        expected_version: 0,
        summary: String::new(),
        image_event_ids: (0..5).map(|_| ObjectId::new().to_hex()).collect(),
        fields: None,
        confirmed_fields: vec![],
        context_hash: None,
    };
    assert!(input.validate().is_ok());
    input.image_event_ids.push(ObjectId::new().to_hex());
    assert!(input.validate().is_err());
    input.image_event_ids.clear();
    assert!(input.validate().is_err());
    let fields:DraftFields=serde_json::from_value(json!({"customer_name":"คุณเอ","branch":"A","items":[
        {"product_name":"nina","color":"dark brown","size":"M","quantity":1,"rental":{"pickup_date":"2026-11-04","usage_date":"2026-11-07","return_date":"2026-11-08","fulfillment_method":"delivery","return_shipping_deadline":"16:00"}},
        {"product_name":"Haatiya","color":"Blush Pink","size":"M","quantity":1,"rental":{"pickup_date":"2026-11-12","usage_date":"2026-11-14","return_date":"2026-11-15","fulfillment_method":"store_pickup"}}]})).unwrap();
    assert!(fields.validate().is_ok());
    assert!(fields.missing_fields().is_empty());
    assert_eq!(
        fields.items[1]
            .effective_rental(&fields.rental)
            .pickup_date
            .as_deref(),
        Some("2026-11-12")
    );
    let mut invalid = fields;
    invalid.items[0]
        .rental
        .as_mut()
        .unwrap()
        .return_shipping_deadline = Some("+1:20".into());
    assert!(invalid.validate().is_err());
}

#[test]
fn customer_extraction_requires_verbatim_customer_evidence() {
    let source = "คุณเอ โทร 0812345678 ที่อยู่ 99 ถนนสุขุมวิท กทม. 10110";
    let sources = std::collections::BTreeMap::from([("e1".into(), source.into())]);
    let valid = json!({"fields":{"display_name":"คุณเอ","phone":"0812345678","address":"99 ถนนสุขุมวิท กทม. 10110"},
        "evidence":[{"field":"display_name","source_type":"line_event","source_ref":"e1","quote":source},
        {"field":"phone","source_type":"line_event","source_ref":"e1","quote":source},{"field":"address","source_type":"line_event","source_ref":"e1","quote":source}],"conflicts":[],"questions":[]});
    assert!(
        customer_extract::validate(&serde_json::from_value(valid.clone()).unwrap(), &sources)
            .is_ok()
    );
    let mut invented = valid.clone();
    invented["fields"]["address"] = json!("99 ถนนสุขุมวิท กรุงเทพมหานคร 99999");
    assert!(
        customer_extract::validate(&serde_json::from_value(invented).unwrap(), &sources).is_err()
    );
    let mut shop = valid;
    shop["evidence"][0]["source_ref"] = json!("shop-message");
    assert!(customer_extract::validate(&serde_json::from_value(shop).unwrap(), &sources).is_err());
}

#[tokio::test]
#[ignore = "requires disposable MongoDB; run cargo test -- --ignored"]
async fn abandoned_submission_locks_recover_without_sending_and_keep_history() {
    let mut cfg = config();
    cfg.mongodb_uri = std::env::var("TEST_MONGODB_URI").unwrap_or(cfg.mongodb_uri);
    cfg.burst_ms = 0;
    let state = AppState::connect(cfg).await.unwrap();
    crate::db::indexes::ensure(&state).await.unwrap();
    text_event(&state, "orphan-start", "ตรวจบิลค้างหลังรีสตาร์ต", 1).await;
    let id = all(&state, "inquiries").await[0]
        .get_object_id("_id")
        .unwrap();
    let review = ObjectId::new();
    let created = DateTime::from_millis(DateTime::now().timestamp_millis() - 180_000);
    let mut record = state.scope();
    record.extend(doc! {"_id":review,"inquiry_id":id,"reservation_key":"orphan-test","group":{"key":"g","branch_id":"a","pickup_date":"2026-11-12","return_date":"2026-11-15","fulfillment_method":"store_pickup","delivery_provider":mongodb::bson::Bson::Null,"lines":[],"notes":""},"created_at":created,"updated_at":created});
    state
        .collection("bill_reviews")
        .insert_one(record)
        .await
        .unwrap();
    state
        .collection("inquiries")
        .update_one(
            doc! {"_id":id},
            doc! {"$set":{"alex_submission_id":review,"alex_submission_started_at":created}},
        )
        .await
        .unwrap();
    let recovered = checkout::list(&state, id).await.unwrap();
    assert!(recovered["source_hash"].is_null());
    assert_eq!(recovered["submissions"][0]["status"], "SUBMIT_UNKNOWN");
    // GET must be idempotent, and recovery must never invoke a write API.
    assert_eq!(
        checkout::list(&state, id).await.unwrap()["submissions"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let base = format!("/api/inquiries/{id}/checkout/resolve");
    let denied = request(
        &state,
        &base,
        "POST",
        json!({"submission_id":review.to_hex(),"outcome":"not_created","verified_in_alex":true}),
        false,
    )
    .await;
    assert_eq!(denied.0, StatusCode::UNAUTHORIZED);
    let resolved = request(
        &state,
        &base,
        "POST",
        json!({"submission_id":review.to_hex(),"outcome":"not_created","verified_in_alex":true}),
        true,
    )
    .await;
    assert_eq!(resolved.1["status"], "NOT_CREATED");
    assert!(
        !drafts::inquiry(&state, id)
            .await
            .unwrap()
            .contains_key("alex_submission_id")
    );
    assert_eq!(
        checkout::list(&state, id).await.unwrap()["submissions"][0]["verification"],
        "manual"
    );
    state.db.drop().await.unwrap();
}

#[derive(Clone, Default)]
struct AiMock {
    mode: Arc<AtomicUsize>,
    calls: Arc<tokio::sync::Mutex<Vec<Value>>>,
    started: Arc<tokio::sync::Notify>,
    resume: Arc<tokio::sync::Notify>,
}
async fn ai_mock(
    axum::extract::State(mock): axum::extract::State<AiMock>,
    Json(body): Json<Value>,
) -> Json<Value> {
    mock.calls.lock().await.push(body.clone());
    assert_eq!(body["tool_choice"], "none");
    let data: Value =
        serde_json::from_str(body["messages"][1]["content"][0]["text"].as_str().unwrap()).unwrap();
    let mode = mock.mode.load(Ordering::SeqCst);
    let result = if let Some(messages) = data["messages"].as_array() {
        assert!(
            messages
                .iter()
                .all(|message| !message["text"].as_str().unwrap().contains("SHOPONLY"))
        );
        if mode == 4 {
            mock.started.notify_one();
            mock.resume.notified().await;
        }
        let message = messages
            .iter()
            .find(|message| message["text"].as_str().unwrap().contains("0812345678"))
            .unwrap();
        let mut result = json!({"fields":{"display_name":"คุณเอ","phone":"0812345678","address":if mode==2{Value::Null}else{json!("99 ถนนสุขุมวิท กทม. 10110")}},
            "evidence":[],"conflicts":if mode==2{json!([{"field":"address","message":"มีที่อยู่มากกว่าหนึ่งแห่ง กรุณายืนยัน"}])}else{json!([])},"questions":[]});
        for field in ["display_name", "phone", "address"] {
            if !result["fields"][field].is_null() {
                result["evidence"].as_array_mut().unwrap().push(json!({"field":field,"source_type":"line_event","source_ref":if mode==1{json!("wrong-source")}else{message["event_ref"].clone()},"quote":message["text"]}));
            }
        }
        result
    } else {
        let refs = data["selected_image_event_refs"].as_array().unwrap();
        let fields = json!({"items":[{"product_name":"nina","color":"dark brown","size":"M","amounts":{"rental_fee_satang":55000,"shipping_fee_satang":4000},"notes":"เข็มกลัดดอกไม้; รับชุด 5,6 ต้องยืนยันวันที่และปี"},
            {"product_name":"Haatiya","color":"Blush Pink","size":"M","notes":"ใช้ 14/11 รับ 12/11 คืน 15/11 ต้องยืนยันปีและราคาที่ตกลง"}]});
        let fields: DraftFields = serde_json::from_value(fields).unwrap();
        let mut evidence = Vec::new();
        for key in fields.values().keys() {
            let index = if key.starts_with("items.1.") { 1 } else { 0 };
            evidence.push(json!({"field":key,"source_type":"admin_attachment","source_ref":refs[index],"quote":"ข้อความบนรูปสรุป"}));
        }
        json!({"fields":fields,"evidence":evidence,"conflicts":[],"questions":["วันที่ในภาพเป็นปีใด และวันรับชุด nina เป็นวันที่ 5 หรือ 6 คะ"]})
    };
    Json(json!({"choices":[{"finish_reason":"stop","message":{"content":result.to_string()}}]}))
}

async fn text_event(state: &AppState, id: &str, text: &str, timestamp: i64) {
    let event=serde_json::from_value(json!({"type":"message","webhookEventId":id,"timestamp":timestamp,"source":{"type":"user","userId":"U0123456789abcdef0123456789abcdef"},"message":{"id":format!("100{timestamp}"),"type":"text","text":text}})).unwrap();
    events::persist(state, &event).await.unwrap();
    let job = events::claim(state).await.unwrap().unwrap();
    worker::process(state, &job).await.unwrap();
}

#[tokio::test]
#[ignore = "requires disposable MongoDB; run cargo test -- --ignored"]
async fn private_photos_and_customer_autofill_are_scoped_grounded_and_not_sent_to_line() {
    let mock = AiMock::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = axum::Router::new()
        .route(
            "/v1/toolsets",
            axum::routing::get(|| async { Json(json!([])) }),
        )
        .route("/v1/chat/completions", axum::routing::post(ai_mock))
        .with_state(mock.clone());
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let mut cfg = config();
    cfg.mongodb_uri = std::env::var("TEST_MONGODB_URI").unwrap_or(cfg.mongodb_uri);
    cfg.burst_ms = 0;
    cfg.hermes_url = Some(format!("http://{address}/v1/chat/completions"));
    cfg.hermes_api_key = Some("fixture-key".into());
    let state = AppState::connect(cfg).await.unwrap();
    crate::db::indexes::ensure(&state).await.unwrap();
    text_event(
        &state,
        "customer-contact",
        "คุณเอ โทร 0812345678 ที่อยู่ 99 ถนนสุขุมวิท กทม. 10110",
        1,
    )
    .await;
    let inquiry = all(&state, "inquiries").await[0].clone();
    let id = inquiry.get_object_id("_id").unwrap();
    let now = DateTime::now();
    let mut shop = state.scope();
    shop.extend(doc! {"_id":ObjectId::new(),"inquiry_id":id,"source":"alex_push","status":"DONE","event_id":"shop-only","event_at":now,
        "created_at":now,"updated_at":now,"payload":{"type":"message","message":{"type":"text","text":"SHOPONLY ชื่อลูกค้าปลอม 0999999999"}}});
    state
        .collection("outbound_messages")
        .insert_one(shop)
        .await
        .unwrap();
    let extract_path = format!("/api/inquiries/{id}/customer-extract");
    let (status, contact) = request(&state, &extract_path, "POST", Value::Null, true).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(contact["fields"]["phone"], "0812345678");
    assert!(all(&state, "customer_profiles").await.is_empty());
    mock.mode.store(1, Ordering::SeqCst);
    assert_eq!(
        request(&state, &extract_path, "POST", Value::Null, true)
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    mock.mode.store(2, Ordering::SeqCst);
    assert_eq!(
        request(&state, &extract_path, "POST", Value::Null, true)
            .await
            .1["fields"]["address"],
        Value::Null
    );
    mock.mode.store(4, Ordering::SeqCst);
    let cloned = state.clone();
    let task = tokio::spawn(async move { customer_extract::extract(&cloned, id).await });
    mock.started.notified().await;
    text_event(&state, "contact-change", "ขอแก้เบอร์โทรภายหลังค่ะ", 2).await;
    mock.resume.notify_one();
    assert!(matches!(
        task.await.unwrap(),
        Err(crate::error::AppError::ContextChanged)
    ));
    mock.mode.store(0, Ordering::SeqCst);
    let png=STANDARD.decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+yZf8AAAAASUVORK5CYII=").unwrap();
    let upload_path = format!("/api/inquiries/{id}/draft-images");
    let mut ids = Vec::new();
    let mut first = None;
    for n in 0..5 {
        let mut bytes = png.clone();
        bytes.push(n);
        let body = json!({"name":format!("summary-{n}.png"),"image_base64":STANDARD.encode(bytes)});
        let (status, result) = request(&state, &upload_path, "POST", body.clone(), true).await;
        assert_eq!(status, StatusCode::OK);
        ids.push(result["attachment"]["id"].as_str().unwrap().to_string());
        if n == 0 {
            first = Some((body, result));
        }
    }
    let (body, first) = first.unwrap();
    let duplicate = request(&state, &upload_path, "POST", body, true).await.1;
    assert_eq!(duplicate["attachment"]["id"], first["attachment"]["id"]);
    assert_eq!(all(&state, "inquiry_attachments").await.len(), 5);
    assert!(all(&state, "outbound_media").await.is_empty());
    let private_url = first["attachment"]["url"].as_str().unwrap();
    let response = routes::router(state.clone())
        .unwrap()
        .oneshot(
            Request::builder()
                .uri(private_url)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let hash = drafts::snapshot(&state, &drafts::inquiry(&state, id).await.unwrap())
        .await
        .unwrap()
        .hash;
    let mut summary =
        json!({"expected_revision":0,"text":"","image_event_ids":ids,"context_hash":hash});
    assert_eq!(
        request(
            &state,
            &format!("/api/inquiries/{id}/summary"),
            "POST",
            summary.clone(),
            true
        )
        .await
        .0,
        StatusCode::OK
    );
    let mut prepare = PrepareInput {
        request_id: "five-images-only".into(),
        expected_version: 0,
        summary: String::new(),
        image_event_ids: ids.clone(),
        fields: None,
        confirmed_fields: vec![],
        context_hash: Some(hash),
    };
    drafts::enqueue(&state, id, prepare.clone(), false)
        .await
        .unwrap();
    let job = drafts::claim(&state).await.unwrap().unwrap();
    draft_worker::process(&state, &job).await.unwrap();
    let saved = drafts::safe_drafts(&state, &drafts::inquiry(&state, id).await.unwrap())
        .await
        .unwrap();
    assert_eq!(
        saved[0]["result"]["fields"]["items"][0]["product_name"],
        "nina"
    );
    assert_eq!(
        saved[0]["result"]["fields"]["items"][1]["product_name"],
        "Haatiya"
    );
    assert!(saved[0]["result"]["fields"]["items"][0]["product_code"].is_null());
    assert!(saved[0]["result"]["fields"]["rental"]["pickup_date"].is_null());
    assert_eq!(
        saved[0]["result"]["fields"]["items"][0]["amounts"]["shipping_fee_satang"],
        4000
    );
    let calls = mock.calls.lock().await;
    let call = calls.last().unwrap();
    assert_eq!(
        call["messages"][1]["content"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|part| part["type"] == "image_url")
            .count(),
        5
    );
    drop(calls);
    prepare.image_event_ids.push(ObjectId::new().to_hex());
    assert!(prepare.validate().is_err());
    summary["image_event_ids"] = json!(prepare.image_event_ids);
    assert!(
        serde_json::from_value::<SummaryInput>(summary)
            .unwrap()
            .validate()
            .is_err()
    );
    assert!(all(&state, "alex_submissions").await.is_empty());
    state.db.drop().await.unwrap();
    server.abort();
}

#[derive(Clone)]
struct OrderMock {
    catalog: super::product_tests::CatalogMock,
    orders: Arc<AtomicUsize>,
    payloads: Arc<tokio::sync::Mutex<Vec<Value>>>,
}
async fn queue_mock(
    axum::extract::State(mock): axum::extract::State<OrderMock>,
    Json(body): Json<Value>,
) -> Json<Value> {
    assert_eq!(body.as_array().unwrap().len(), 1);
    let line = &body[0];
    assert_eq!(line["branchId"], "branch-a");
    assert_eq!(line["qty"], 1);
    let mode = mock.catalog.mode.load(Ordering::SeqCst);
    Json(if mode == 21 {
        json!([{"result":9}])
    } else {
        json!([{"result":if mode==20{0}else{1},"productItemId":line["productItemId"],"pickupDate":format!("{}T00:00:00",line["pickupDate"].as_str().unwrap()),"returnDate":format!("{}T00:00:00",line["returnDate"].as_str().unwrap())}])
    })
}
async fn order_mock(
    axum::extract::State(mock): axum::extract::State<OrderMock>,
    Json(body): Json<Value>,
) -> axum::response::Response {
    mock.orders.fetch_add(1, Ordering::SeqCst);
    mock.payloads.lock().await.push(body);
    if mock.catalog.mode.load(Ordering::SeqCst) == 23 {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error":"timeout after committing"})),
        )
            .into_response()
    } else {
        Json(json!({"success":true,"orderId":"alex-bill-1","token":"must-be-removed"}))
            .into_response()
    }
}

#[tokio::test]
#[ignore = "requires disposable MongoDB; run cargo test -- --ignored"]
async fn checkout_splits_dates_checks_stock_and_never_retries_unknown_orders() {
    use super::product_tests::{CatalogMock, catalog_config, catalog_mock, login_mock, manual};
    let catalog = CatalogMock::default();
    let mock = OrderMock {
        catalog: catalog.clone(),
        orders: Arc::new(AtomicUsize::new(0)),
        payloads: Arc::new(tokio::sync::Mutex::new(vec![])),
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let reads = axum::Router::new()
        .route("/api/user/login", axum::routing::post(login_mock))
        .route("/api/v2/GetProductItems", axum::routing::post(catalog_mock))
        .with_state(catalog.clone());
    let writes = axum::Router::new()
        .route("/api/orderdetail/checkq", axum::routing::post(queue_mock))
        .route("/api/v2/AddRentalOrder", axum::routing::post(order_mock))
        .with_state(mock.clone());
    let server =
        tokio::spawn(async move { axum::serve(listener, reads.merge(writes)).await.unwrap() });
    let mut cfg = config();
    cfg.mongodb_uri = std::env::var("TEST_MONGODB_URI").unwrap_or(cfg.mongodb_uri);
    cfg.burst_ms = 0;
    cfg.alex = Some(catalog_config(&format!("http://{address}")));
    let state = AppState::connect(cfg).await.unwrap();
    crate::db::indexes::ensure(&state).await.unwrap();
    text_event(
        &state,
        "bill-start",
        "คุณเอ โทร 0812345678 ที่อยู่ 99 ถนนสุขุมวิท กทม. 10110",
        1,
    )
    .await;
    let id = all(&state, "inquiries").await[0]
        .get_object_id("_id")
        .unwrap();
    let fields:DraftFields=serde_json::from_value(json!({"customer_name":"คุณเอ","customer_phone":"0812345678","customer_address":"99 ถนนสุขุมวิท กทม. 10110","branch":"A","items":[
        {"product_name":"nina","product_code":"DR-012","color":"R","size":"M","quantity":1,"rental":{"pickup_date":"2026-11-04","return_date":"2026-11-08","fulfillment_method":"delivery","delivery_provider":"Flash"},"amounts":{"rental_fee_satang":55000,"deposit_satang":0,"shipping_fee_satang":4000}},
        {"product_name":"Haatiya","product_code":"DR-012","color":"B","size":"L","quantity":1,"rental":{"pickup_date":"2026-11-12","return_date":"2026-11-15","fulfillment_method":"store_pickup"},"amounts":{"rental_fee_satang":100000,"deposit_satang":200001,"shipping_fee_satang":0}}]})).unwrap();
    manual(&state, id, 0, fields).await;
    let hash = drafts::snapshot(&state, &drafts::inquiry(&state, id).await.unwrap())
        .await
        .unwrap()
        .hash;
    for (index, product) in [(0, "item-1"), (1, "item-2")] {
        let result=request(&state,&format!("/api/inquiries/{id}/products/search"),"POST",json!({"draft_version":1,"context_hash":hash,"item_index":index,"branch_id":"branch-a","mode":"code","query":"DR-012"}),true).await.1;
        assert_eq!(request(&state,&format!("/api/inquiries/{id}/products/confirm"),"POST",json!({"search_id":result["search_id"],"product_item_id":product,"expected_revision":0,"acknowledge_difference":true}),true).await.0,StatusCode::OK);
    }
    let base = format!("/api/inquiries/{id}/checkout");
    let data = checkout::list(&state, id).await.unwrap();
    assert_eq!(data["groups"].as_array().unwrap().len(), 2);
    let group = data["groups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|group| group["pickup_date"] == "2026-11-04")
        .unwrap();
    let input = json!({"source_hash":data["source_hash"],"group_key":group["key"]});
    catalog.mode.store(20, Ordering::SeqCst);
    assert_eq!(
        request(
            &state,
            &format!("{base}/availability"),
            "POST",
            input.clone(),
            true
        )
        .await
        .1["state"],
        "UNAVAILABLE"
    );
    catalog.mode.store(21, Ordering::SeqCst);
    assert_eq!(
        request(
            &state,
            &format!("{base}/availability"),
            "POST",
            input.clone(),
            true
        )
        .await
        .1["state"],
        "UNKNOWN"
    );
    catalog.mode.store(0, Ordering::SeqCst);
    assert_eq!(
        request(&state, &format!("{base}/availability"), "POST", input, true)
            .await
            .1["state"],
        "AVAILABLE"
    );
    let mut review = json!({"source_hash":data["source_hash"],"group_key":group["key"],"customer_name":"คุณเอ","customer_phone":"0812345678","customer_address":"99 ถนนสุขุมวิท กทม. 10110",
        "rental_fee_satang":55000,"deposit_satang":0,"shipping_fee_satang":4000,"receiving_method":"Flash","rental_payment_method":"โอนเงิน","bail_payment_method":"โอนเงิน","notes":"เข็มกลัดดอกไม้"});
    assert_eq!(
        request(
            &state,
            &format!("{base}/review"),
            "POST",
            review.clone(),
            true
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    review["shipping_note_only"] = json!(true);
    let (status, preview) = request(
        &state,
        &format!("{base}/review"),
        "POST",
        review.clone(),
        true,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(preview["rental_discount_satang"], 95000);
    assert_eq!(mock.orders.load(Ordering::SeqCst), 0);
    let submit = json!({"review_id":preview["review_id"],"confirmed":true});
    let result = request(
        &state,
        &format!("{base}/submit"),
        "POST",
        submit.clone(),
        true,
    )
    .await;
    assert_eq!(result.0, StatusCode::OK);
    assert_eq!(result.1["status"], "CONFIRMED");
    assert_eq!(
        request(&state, &format!("{base}/submit"), "POST", submit, true)
            .await
            .1["alex_order_id"],
        "alex-bill-1"
    );
    assert_eq!(mock.orders.load(Ordering::SeqCst), 1);
    let payload = mock.payloads.lock().await[0].clone();
    assert_eq!(payload["orderDetail"]["rentalDiscount"], "950.00");
    assert_eq!(payload["orderItems"][0]["productItemId"], "item-1");
    assert!(payload["orderDetail"].get("birthDate").is_none());
    assert!(
        payload["orderDetail"]["remark1"]
            .as_str()
            .unwrap()
            .contains("40.00")
    );
    let record = all(&state, "alex_submissions").await[0].clone();
    assert!(
        record
            .get_document("alex_response")
            .unwrap()
            .get("token")
            .is_none()
    );
    let another = request(
        &state,
        &format!("{base}/review"),
        "POST",
        review.clone(),
        true,
    )
    .await
    .1;
    assert_eq!(
        request(
            &state,
            &format!("{base}/submit"),
            "POST",
            json!({"review_id":another["review_id"],"confirmed":true}),
            true
        )
        .await
        .1["error"]["code"],
        "BILL_PENDING"
    );
    assert_eq!(mock.orders.load(Ordering::SeqCst), 1);
    let second = data["groups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|group| group["pickup_date"] == "2026-11-12")
        .unwrap();
    review["group_key"] = second["key"].clone();
    review["shipping_fee_satang"] = json!(0);
    review["receiving_method"] = json!("รับหน้าร้าน");
    let preview = request(&state, &format!("{base}/review"), "POST", review, true)
        .await
        .1;
    catalog.mode.store(23, Ordering::SeqCst);
    let submit = json!({"review_id":preview["review_id"],"confirmed":true});
    let unknown = request(
        &state,
        &format!("{base}/submit"),
        "POST",
        submit.clone(),
        true,
    )
    .await
    .1;
    assert_eq!(unknown["status"], "SUBMIT_UNKNOWN");
    assert_eq!(
        request(&state, &format!("{base}/submit"), "POST", submit, true)
            .await
            .1["status"],
        "SUBMIT_UNKNOWN"
    );
    assert_eq!(mock.orders.load(Ordering::SeqCst), 2);
    let resolve = json!({"submission_id":unknown["_id"]["$oid"],"outcome":"not_created","verified_in_alex":true});
    assert_eq!(
        request(&state, &format!("{base}/resolve"), "POST", resolve, true)
            .await
            .0,
        StatusCode::CONFLICT
    );
    let resolve = json!({"submission_id":unknown["_id"]["$oid"],"outcome":"created","order_id":"alex-bill-2","verified_in_alex":true});
    assert_eq!(
        request(&state, &format!("{base}/resolve"), "POST", resolve, true)
            .await
            .1["status"],
        "CONFIRMED"
    );
    assert_eq!(mock.orders.load(Ordering::SeqCst), 2);
    text_event(&state, "after-bill", "ขอแก้ข้อมูลภายหลัง", 3).await;
    let stale = checkout::list(&state, id).await.unwrap();
    assert!(stale["source_hash"].is_null());
    assert_eq!(stale["submissions"].as_array().unwrap().len(), 2);
    state.db.drop().await.unwrap();
    server.abort();
}
