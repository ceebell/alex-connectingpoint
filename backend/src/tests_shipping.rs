use super::*;
use crate::db::shipping;
use axum::{Json, extract::State, response::IntoResponse};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Default)]
struct ShippingMock {
    tools_enabled: Arc<std::sync::atomic::AtomicBool>,
    reads: Arc<AtomicUsize>,
    pushes: Arc<tokio::sync::Mutex<Vec<(String, Value)>>>,
    accepted: Arc<tokio::sync::Mutex<std::collections::BTreeSet<String>>>,
}
async fn toolsets(State(mock): State<ShippingMock>) -> Json<Value> {
    Json(if mock.tools_enabled.load(Ordering::SeqCst) {
        json!([{"enabled":true,"configured":true,"tools":["terminal"]}])
    } else {
        json!([])
    })
}
fn extraction() -> Value {
    json!({"carrier":"THAILAND_POST","receipt_number":"627275","receipt_date":"2026-11-16","total_satang":4800,
    "parcels":[{"row":1,"recipient_name":"กัญญาณัฐ","tracking_number":"JD 3334 5879 0 TH","postal_code":"10150","service":"EMS","shipping_fee_satang":2400,"raw_text":"ผู้รับ กัญญาณัฐ JD 3334 5879 0 TH ฿21.00 ฿3.00"},
    {"row":2,"recipient_name":"พิมพ์นิภา","tracking_number":"JD333458809TH","postal_code":"73140","service":"EMS","shipping_fee_satang":2400,"raw_text":"ผู้รับ พิมพ์นิภา JD333458809TH ฿21.00 ฿3.00"}]})
}
async fn hermes(State(mock): State<ShippingMock>, Json(body): Json<Value>) -> Json<Value> {
    mock.reads.fetch_add(1, Ordering::SeqCst);
    assert_eq!(body["tools"], json!([]));
    assert_eq!(body["tool_choice"], "none");
    assert_eq!(body["messages"][1]["content"].as_array().unwrap().len(), 1);
    assert!(
        body["messages"][1]["content"][0]["image_url"]["url"]
            .as_str()
            .unwrap()
            .starts_with("data:image/png;base64,")
    );
    Json(
        json!({"choices":[{"finish_reason":"stop","message":{"content":extraction().to_string()}}]}),
    )
}
async fn push(
    State(mock): State<ShippingMock>,
    headers: axum::http::HeaderMap,
    Json(body): Json<Value>,
) -> axum::response::Response {
    let key = headers["x-line-retry-key"].to_str().unwrap().to_string();
    mock.pushes.lock().await.push((key.clone(), body));
    let first = mock.accepted.lock().await.insert(key);
    // Simulate LINE accepting the message before the connection returns an error.
    if first {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error":"lost response"})),
        )
            .into_response()
    } else {
        (
            StatusCode::CONFLICT,
            Json(json!({"message":"retry key already accepted"})),
        )
            .into_response()
    }
}
async fn seed_order(
    state: &AppState,
    name: &str,
    user: &str,
    number: &str,
) -> (ObjectId, ObjectId) {
    let now = DateTime::now();
    let inquiry = ObjectId::new();
    let conversation = ObjectId::new();
    let submission = ObjectId::new();
    let mut record = state.scope();
    record.extend(doc!{"_id":conversation,"conversation_key":format!("[\"user\",\"{user}\"]"),"created_at":now,"updated_at":now});
    state
        .collection("conversations")
        .insert_one(record)
        .await
        .unwrap();
    let mut record = state.scope();
    record.extend(doc!{"_id":inquiry,"line_user_id":user,"conversation_id":conversation,"status":"OPEN","created_at":now,"updated_at":now});
    state
        .collection("inquiries")
        .insert_one(record)
        .await
        .unwrap();
    let mut record = state.scope();
    record.extend(doc!{"_id":ObjectId::new(),"line_user_id":user,"display_name":format!("LINE {user}"),"picture_url":"https://example.test/customer.png","created_at":now,"updated_at":now});
    state
        .collection("line_users")
        .insert_one(record)
        .await
        .unwrap();
    let mut record = state.scope();
    record.extend(doc!{"_id":submission,"inquiry_id":inquiry,"line_user_id":user,"draft_id":ObjectId::new(),"draft_version":3_i64,"status":"CONFIRMED","alex_order_id":format!("alex-{number}"),"alex_order_number":number,"customer":{"name":name,"phone":"0812345678","address":"กรุงเทพฯ 10150"},"group":{"branch_id":"branch-a","pickup_date":"2026-11-16"},"created_at":now,"updated_at":now});
    state
        .collection("alex_submissions")
        .insert_one(record)
        .await
        .unwrap();
    (submission, inquiry)
}

#[tokio::test]
#[ignore = "requires disposable MongoDB"]
async fn receipt_ocr_matches_only_scoped_orders_and_sends_only_after_explicit_review() {
    let mock = ShippingMock::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = axum::Router::new()
        .route("/v1/toolsets", axum::routing::get(toolsets))
        .route("/v1/chat/completions", axum::routing::post(hermes))
        .route("/v2/bot/message/push", axum::routing::post(push))
        .with_state(mock.clone());
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let mut cfg = config();
    cfg.mongodb_uri = std::env::var("TEST_MONGODB_URI").unwrap_or(cfg.mongodb_uri);
    cfg.hermes_url = Some(format!("http://{address}/v1/chat/completions"));
    cfg.hermes_api_key = Some("test-key".into());
    cfg.line_api_base = format!("http://{address}");
    let state = AppState::connect(cfg).await.unwrap();
    crate::db::indexes::ensure(&state).await.unwrap();
    let (bill, inquiry) = seed_order(&state, "K.กัญญาณัฐ", "user-a", "AR-0001").await;
    let (same_name, _) = seed_order(&state, "K.กัญญาณัฐ", "user-b", "AR-0002").await;
    let upload = json!({"request_id":"12345678-1234-1234-1234-123456789abc","name":"receipt.png","image_base64":"iVBORw0KGgo="});
    let base = "/api/admin/shipping";
    assert_eq!(
        request(
            &state,
            &format!("{base}/receipts"),
            "POST",
            upload.clone(),
            false
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    let uploaded = request(
        &state,
        &format!("{base}/receipts"),
        "POST",
        upload.clone(),
        true,
    )
    .await;
    assert_eq!(uploaded.0, StatusCode::OK);
    let receipt = uploaded.1["receipt"]["_id"]["$oid"]
        .as_str()
        .unwrap()
        .to_string();
    let rid = ObjectId::parse_str(&receipt).unwrap();
    assert_eq!(uploaded.1["receipt"]["status"], "QUEUED");
    assert!(mock.pushes.lock().await.is_empty());
    assert!(shipping::work_one(&state).await.unwrap());
    assert!(!shipping::work_one(&state).await.unwrap());
    let result = shipping::get(&state, rid).await.unwrap();
    assert_eq!(result["receipt"]["status"], "READY");
    assert_eq!(
        result["receipt"]["extraction"]["parcels"][0]["tracking_number"],
        "JD333458790TH"
    );
    assert_eq!(result["receipt"]["total_mismatch"], false);
    assert!(result["trackings"].as_array().unwrap().is_empty());
    assert_eq!(
        request(&state, &format!("{base}/receipts"), "POST", upload, true)
            .await
            .1["receipt"]["_id"],
        result["receipt"]["_id"]
    );
    assert_eq!(mock.reads.load(Ordering::SeqCst), 1);
    assert_eq!(
        request(
            &state,
            &format!("{base}/receipts/{receipt}/image"),
            "GET",
            Value::Null,
            false
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    let candidates = shipping::candidates(&state, "กัญญาณัฐ").await.unwrap();
    assert_eq!(candidates["orders"].as_array().unwrap().len(), 2);
    assert!(
        candidates["orders"]
            .as_array()
            .unwrap()
            .iter()
            .all(|o| o["match_score"] == 100
                && o["chat_customer"]["picture_url"] == "https://example.test/customer.png")
    );
    let other_state = {
        let mut cfg = config();
        cfg.mongodb_db = state.config.mongodb_db.clone();
        cfg.shop_id = "another-shop".into();
        AppState::connect(cfg).await.unwrap()
    };
    assert!(
        shipping::candidates(&other_state, "กัญญาณัฐ").await.unwrap()["orders"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(matches!(
        shipping::image(&other_state, rid).await,
        Err(crate::error::AppError::NotFound)
    ));
    let row = format!("{base}/receipts/{receipt}/rows/1");
    let mut save = json!({"expected_version":0,"submission_id":bill.to_hex(),"recipient_name":"กัญญาณัฐ","tracking_number":"JD 3334 5879 0 TH","postal_code":"10150","service":"EMS","shipping_fee_satang":2400,"message":"เลขพัสดุ JD333458790TH สำหรับ AR-0001 ค่ะ"});
    let saved = request(&state, &row, "POST", save.clone(), true).await;
    assert_eq!(saved.0, StatusCode::OK);
    assert_eq!(saved.1["tracking"]["status"], "READY");
    assert!(mock.pushes.lock().await.is_empty());
    assert_eq!(saved.1["tracking"]["alex_order_id"], "alex-AR-0001");
    assert_eq!(saved.1["tracking"]["alex_order_number"], "AR-0001");
    assert_eq!(saved.1["tracking"]["draft_version"], 3);
    assert_eq!(
        saved.1["tracking"]["inquiry_id"],
        serde_json::to_value(inquiry).unwrap()
    );
    let duplicate = request(
        &state,
        &format!("{base}/receipts/{receipt}/rows/2"),
        "POST",
        save.clone(),
        true,
    )
    .await;
    assert_eq!(duplicate.1["error"]["code"], "VALIDATION_ERROR");
    assert_eq!(
        request(&state, &row, "POST", save.clone(), true).await.1["error"]["code"],
        "RECORD_CONFLICT"
    );
    let tracking_id = saved.1["tracking"]["_id"]["$oid"].as_str().unwrap();
    let reloaded = shipping::get(&state, rid).await.unwrap();
    assert_eq!(
        reloaded["trackings"][0]["chat_customer"]["display_name"],
        "LINE user-a"
    );
    assert_eq!(
        reloaded["trackings"][0]["chat_customer"]["picture_url"],
        "https://example.test/customer.png"
    );
    let send = format!("{base}/trackings/{tracking_id}/send");
    assert_eq!(
        request(&state, &send, "POST", json!({"expected_version":2}), true)
            .await
            .1["error"]["code"],
        "RECORD_CONFLICT"
    );
    let first = request(&state, &send, "POST", json!({"expected_version":1}), true).await;
    assert_eq!(first.1["accepted"], false);
    assert_eq!(first.1["tracking"]["status"], "SEND_FAILED");
    save["expected_version"] = json!(1);
    save["submission_id"] = json!(same_name.to_hex());
    assert_eq!(
        request(&state, &row, "POST", save, true).await.1["error"]["code"],
        "RECORD_CONFLICT"
    );
    let retried = request(&state, &send, "POST", json!({"expected_version":1}), true).await;
    assert_eq!(retried.1["tracking"]["status"], "SENT");
    assert_eq!(
        request(&state, &send, "POST", json!({"expected_version":1}), true)
            .await
            .1["tracking"]["status"],
        "SENT"
    );
    let pushes = mock.pushes.lock().await.clone();
    assert_eq!(pushes.len(), 2);
    assert_eq!(pushes[0], pushes[1]);
    assert_eq!(pushes[0].1["to"], "user-a");
    assert_eq!(mock.accepted.lock().await.len(), 1);
    let tid = ObjectId::parse_str(tracking_id).unwrap();
    state
        .collection("shipping_trackings")
        .update_one(doc! {"_id":tid}, doc! {"$set":{"status":"SENDING"}})
        .await
        .unwrap();
    assert_eq!(
        shipping::get(&state, rid).await.unwrap()["trackings"][0]["status"],
        "SENT"
    );
    assert_eq!(mock.pushes.lock().await.len(), 2);
    let mut restricted = state.clone();
    restricted.access = Some(crate::models::team::Access {
        member_id: ObjectId::new(),
        line_user_id: "staff".into(),
        name: "Staff".into(),
        role: crate::models::team::Role::Staff,
        branch_ids: vec!["branch-b".into()],
        version: 1,
    });
    assert!(
        shipping::candidates(&restricted, "กัญญาณัฐ").await.unwrap()["orders"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        shipping::get(&restricted, rid).await.unwrap()["trackings"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    // An uncertain send must not be repeated after LINE's retry-key safety window.
    let row2 = format!("{base}/receipts/{receipt}/rows/2");
    let saved2=request(&state,&row2,"POST",json!({"expected_version":0,"submission_id":same_name.to_hex(),"recipient_name":"พิมพ์นิภา","tracking_number":"JD333458809TH","postal_code":"73140","service":"EMS","shipping_fee_satang":2400,"message":"เลขพัสดุ JD333458809TH"}),true).await;
    assert_eq!(saved2.0, StatusCode::OK);
    let tracking2 =
        ObjectId::parse_str(saved2.1["tracking"]["_id"]["$oid"].as_str().unwrap()).unwrap();
    state.collection("shipping_trackings").update_one(doc!{"_id":tracking2},doc!{"$set":{"status":"SEND_FAILED","delivery_started_at":DateTime::from_millis(DateTime::now().timestamp_millis()-24*3_600_000)}}).await.unwrap();
    assert_eq!(
        request(
            &state,
            &format!("{base}/trackings/{tracking2}/send"),
            "POST",
            json!({"expected_version":1}),
            true
        )
        .await
        .1["error"]["code"],
        "VALIDATION_ERROR"
    );
    assert_eq!(mock.pushes.lock().await.len(), 2);
    // Refuse to submit receipt data to an extraction profile with executable tools enabled.
    mock.tools_enabled.store(true, Ordering::SeqCst);
    let uploaded2=request(&state,&format!("{base}/receipts"),"POST",json!({"request_id":"22345678-1234-1234-1234-123456789abc","name":"second.png","image_base64":"iVBORw0KGgoB"}),true).await;
    assert_eq!(uploaded2.0, StatusCode::OK);
    let receipt2 =
        ObjectId::parse_str(uploaded2.1["receipt"]["_id"]["$oid"].as_str().unwrap()).unwrap();
    shipping::work_one(&state).await.unwrap();
    let failed = shipping::get(&state, receipt2).await.unwrap();
    assert_eq!(failed["receipt"]["status"], "FAILED");
    assert_eq!(failed["receipt"]["error_code"], "HERMES_TOOLS_ENABLED");
    assert_eq!(mock.reads.load(Ordering::SeqCst), 1);
    mock.tools_enabled.store(false, Ordering::SeqCst);
    assert_eq!(
        request(
            &state,
            &format!("{base}/receipts/{receipt2}/extract"),
            "POST",
            Value::Null,
            true
        )
        .await
        .0,
        StatusCode::OK
    );
    shipping::work_one(&state).await.unwrap();
    assert_eq!(
        shipping::get(&state, receipt2).await.unwrap()["receipt"]["status"],
        "READY"
    );
    assert_eq!(mock.reads.load(Ordering::SeqCst), 2);
    assert_eq!(mock.pushes.lock().await.len(), 2);
    state.db.drop().await.unwrap();
    server.abort();
}
