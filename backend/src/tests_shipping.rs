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

#[tokio::test]
#[ignore = "requires disposable MongoDB"]
async fn chat_names_find_old_orders_and_keep_duplicate_customers_separate() {
    let state = AppState::connect(config()).await.unwrap();
    crate::db::indexes::ensure(&state).await.unwrap();
    let (bill, _) = seed_order(&state, "Different billing name", "old-user", "OLD-001").await;
    state
        .collection("alex_submissions")
        .update_one(
            doc! {"_id":bill},
            doc! {"$set":{"created_at":DateTime::from_millis(0)}},
        )
        .await
        .unwrap();
    let mut user = state.scope();
    user.insert("line_user_id", "old-user");
    state
        .collection("line_users")
        .update_one(user.clone(), doc! {"$set":{"display_name":"น้องพิม"}})
        .await
        .unwrap();
    user.extend(doc! {"_id":ObjectId::new(),"custom_name":"คุณพิมพ์นิภา","created_at":DateTime::now(),"updated_at":DateTime::now()});
    state
        .collection("customer_aliases")
        .insert_one(user)
        .await
        .unwrap();
    let mut profile = state.scope();
    profile.extend(doc! {"_id":ObjectId::new(),"line_user_id":"old-user","display_name":"Admin Edited Profile","created_at":DateTime::now(),"updated_at":DateTime::now()});
    state
        .collection("customer_profiles")
        .insert_one(profile)
        .await
        .unwrap();
    state
        .collection("line_users")
        .update_one(
            doc! {"line_user_id":"old-user"},
            doc! {"$set":{"oa_display_name":"OA Current Name"}},
        )
        .await
        .unwrap();
    let fillers: Vec<_> = (0..1001).map(|n| {
        let mut row=state.scope();
        row.extend(doc! {"_id":ObjectId::new(),"status":"CONFIRMED","alex_order_id":format!("filler-{n}"),"line_user_id":"unused","customer":{"name":"Unrelated"},"group":{"branch_id":"branch-a"},"created_at":DateTime::now(),"updated_at":DateTime::now()});
        row
    }).collect();
    state
        .collection("alex_submissions")
        .insert_many(fillers)
        .await
        .unwrap();
    for query in [
        "พิมพ์นิภา",
        "น้องพิม",
        "Admin Edited Profile",
        "OA Current Name",
        "Different billing name",
        "OLD-001",
    ] {
        let result = shipping::candidates(&state, query).await.unwrap();
        assert_eq!(result["orders"].as_array().unwrap().len(), 1, "{query}");
        assert_eq!(result["orders"][0]["submission_id"], bill.to_hex());
        assert_eq!(
            result["orders"][0]["chat_customer"]["display_name"],
            "คุณพิมพ์นิภา"
        );
        assert_eq!(
            result["orders"][0]["match_score"],
            if query == "OLD-001" { 110 } else { 100 }
        );
    }
    let (other, _) = seed_order(&state, "Another billing name", "other-user", "OTHER-001").await;
    let mut user = state.scope();
    user.insert("line_user_id", "other-user");
    state
        .collection("line_users")
        .update_one(user, doc! {"$set":{"display_name":"พิมพ์นิภา"}})
        .await
        .unwrap();
    let result = shipping::candidates(&state, "พิมพ์นิภา").await.unwrap();
    assert_eq!(result["orders"].as_array().unwrap().len(), 2);
    assert_ne!(
        result["orders"][0]["line_user_id"],
        result["orders"][1]["line_user_id"]
    );
    state
        .collection("alex_submissions")
        .update_one(
            doc! {"_id":other},
            doc! {"$set":{"group.branch_id":"branch-b"}},
        )
        .await
        .unwrap();
    let mut restricted = state.clone();
    restricted.access = Some(crate::models::team::Access {
        member_id: ObjectId::new(),
        line_user_id: "staff".into(),
        name: "Staff".into(),
        role: crate::models::team::Role::Staff,
        branch_ids: vec!["branch-a".into()],
        version: 1,
    });
    let result = shipping::candidates(&restricted, "พิมพ์นิภา").await.unwrap();
    assert_eq!(result["orders"].as_array().unwrap().len(), 1);
    assert_eq!(result["orders"][0]["submission_id"], bill.to_hex());
    let mut other_shop = config();
    other_shop.mongodb_db = state.config.mongodb_db.clone();
    other_shop.shop_id = "another-shop".into();
    let other_shop = AppState::connect(other_shop).await.unwrap();
    assert!(
        shipping::candidates(&other_shop, "พิมพ์นิภา").await.unwrap()["orders"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    state.db.drop().await.unwrap();
}

#[tokio::test]
#[ignore = "requires disposable MongoDB"]
async fn unconfirmed_bill_and_edited_contact_names_find_chat_without_authorizing_tracking() {
    let state = AppState::connect(config()).await.unwrap();
    crate::db::indexes::ensure(&state).await.unwrap();
    let (bill, inquiry) = seed_order(
        &state,
        "K. Example Recipient",
        "user-pending",
        "PENDING-001",
    )
    .await;
    state.collection("alex_submissions").update_one(doc! {"_id":bill},doc! {"$set":{"status":"SUBMIT_UNKNOWN"},"$unset":{"alex_order_id":"","alex_order_number":""}}).await.unwrap();
    state
        .collection("line_users")
        .update_one(
            doc! {"line_user_id":"user-pending"},
            doc! {"$set":{"display_name":"Chat A"}},
        )
        .await
        .unwrap();
    let result = shipping::candidates(&state, "Example Recipient")
        .await
        .unwrap();
    assert!(result["orders"].as_array().unwrap().is_empty());
    let matched = &result["customer_matches"][0];
    assert_eq!(matched["chat_customer"]["display_name"], "Chat A");
    assert_eq!(
        matched["chat_customer"]["picture_url"],
        "https://example.test/customer.png"
    );
    assert_eq!(matched["matched_name"], "K. Example Recipient");
    assert_eq!(matched["match_source"], "alex_submission");
    assert_eq!(matched["submission_status"], "SUBMIT_UNKNOWN");
    assert_eq!(matched["inquiry_id"], inquiry.to_hex());
    assert!(matched.get("customer").is_none());
    // Older submissions may have saved their customer only in the reviewed form or payload.
    for fields in [
        doc! {"review_input":{"customer_name":"Example Recipient"}},
        doc! {"input":{"customer_name":"Example Recipient"}},
        doc! {"payload":{"orderDetail":{"customerName":"Example Recipient"}}},
    ] {
        state
            .collection("alex_submissions")
            .update_one(
                doc! {"_id":bill},
                doc! {"$unset":{"customer":"","review_input":"","input":"","payload":""}},
            )
            .await
            .unwrap();
        state
            .collection("alex_submissions")
            .update_one(doc! {"_id":bill}, doc! {"$set":fields})
            .await
            .unwrap();
        let result = shipping::candidates(&state, "Example Recipient")
            .await
            .unwrap();
        assert_eq!(
            result["customer_matches"][0]["line_user_id"],
            "user-pending"
        );
        assert!(result["orders"].as_array().unwrap().is_empty());
    }
    let (profile_bill, profile_inquiry) = seed_order(
        &state,
        "Unrelated Billing Name",
        "user-profile",
        "PROFILE-001",
    )
    .await;
    state
        .collection("alex_submissions")
        .delete_one(doc! {"_id":profile_bill})
        .await
        .unwrap();
    let now = DateTime::now();
    let mut profile = state.scope();
    profile.extend(doc! {"_id":ObjectId::new(),"line_user_id":"user-profile","display_name":"Edited Contact Recipient","created_at":now,"updated_at":now});
    state
        .collection("customer_profiles")
        .insert_one(profile)
        .await
        .unwrap();
    let mut alias = state.scope();
    alias.extend(doc! {"_id":ObjectId::new(),"line_user_id":"user-profile","custom_name":"Chat B Custom","created_at":now,"updated_at":now});
    state
        .collection("customer_aliases")
        .insert_one(alias)
        .await
        .unwrap();
    let result = shipping::candidates(&state, "Edited Contact Recipient")
        .await
        .unwrap();
    assert!(result["orders"].as_array().unwrap().is_empty());
    assert_eq!(
        result["customer_matches"][0]["match_source"],
        "customer_profile"
    );
    assert_eq!(
        result["customer_matches"][0]["chat_customer"]["display_name"],
        "Chat B Custom"
    );
    assert_eq!(
        result["customer_matches"][0]["inquiry_id"],
        profile_inquiry.to_hex()
    );
    assert_eq!(
        result["customer_matches"][0]["chat_customer"]["picture_url"],
        "https://example.test/customer.png"
    );
    // A mismatched submission must never associate another customer's chat with this name.
    state
        .collection("alex_submissions")
        .update_one(
            doc! {"_id":bill},
            doc! {"$set":{"inquiry_id":profile_inquiry}},
        )
        .await
        .unwrap();
    assert!(
        shipping::candidates(&state, "Example Recipient")
            .await
            .unwrap()["customer_matches"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    state
        .collection("alex_submissions")
        .update_one(doc! {"_id":bill}, doc! {"$set":{"inquiry_id":inquiry}})
        .await
        .unwrap();
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
        shipping::candidates(&restricted, "Example Recipient")
            .await
            .unwrap()["customer_matches"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let mut other = config();
    other.mongodb_db = state.config.mongodb_db.clone();
    other.channel_id = "other-channel".into();
    let other = AppState::connect(other).await.unwrap();
    assert!(
        shipping::candidates(&other, "Edited Contact Recipient")
            .await
            .unwrap()["customer_matches"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let receipt_id = ObjectId::new();
    let mut receipt = state.scope();
    receipt.extend(doc! {"_id":receipt_id,"request_id":"fixture-request","content_hash":"fixture-image","status":"READY","extraction":mongodb::bson::to_document(&extraction()).unwrap(),"created_at":now,"updated_at":now});
    state
        .collection("shipping_receipts")
        .insert_one(receipt)
        .await
        .unwrap();
    let actor = crate::models::customer::Actor {
        id: "fixture-admin".into(),
        name: "Fixture Admin".into(),
    };
    let input = json!({"expected_version":0,"submission_id":bill.to_hex(),"recipient_name":"Example Recipient","tracking_number":"JD333458790TH","postal_code":"10150","service":"EMS","shipping_fee_satang":2400,"message":"Tracking JD333458790TH"});
    assert!(matches!(
        shipping::save(
            &state,
            receipt_id,
            1,
            serde_json::from_value(input).unwrap(),
            &actor
        )
        .await,
        Err(crate::error::AppError::NotFound)
    ));
    assert_eq!(
        state
            .collection("shipping_trackings")
            .count_documents(doc! {})
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        state
            .collection("outbound_messages")
            .count_documents(doc! {})
            .await
            .unwrap(),
        0
    );
    state.db.drop().await.unwrap();
}

#[tokio::test]
#[ignore = "requires disposable MongoDB"]
async fn name_fallback_prefers_bills_then_profiles_then_current_scoped_notes() {
    let state = AppState::connect(config()).await.unwrap();
    crate::db::indexes::ensure(&state).await.unwrap();
    let (bill, inquiry) = seed_order(
        &state,
        "Example Recipient Fullname",
        "note-user",
        "NOTE-001",
    )
    .await;
    let now = DateTime::now();
    let mut profile = state.scope();
    profile.extend(doc! {"_id":ObjectId::new(),"line_user_id":"note-user","display_name":"Example Recipient","notes":"","created_at":now,"updated_at":now});
    state
        .collection("customer_profiles")
        .insert_one(profile)
        .await
        .unwrap();
    let note_id = ObjectId::new();
    let mut note = state.scope();
    note.extend(doc! {"_id":note_id,"line_user_id":"note-user","text":"ชื่อผู้รับ Example Recipient นัดส่งวันจันทร์","version":1_i64,"images":[],"history":[],"created_by":"Fixture Admin","updated_by":"Fixture Admin","created_at":now,"updated_at":now});
    state
        .collection("customer_notes")
        .insert_one(note.clone())
        .await
        .unwrap();
    let result = shipping::candidates(&state, "Example Recipient")
        .await
        .unwrap();
    assert_eq!(
        result["customer_matches"][0]["match_source"],
        "alex_submission"
    );
    assert_eq!(
        result["customer_matches"][0]["matched_name"],
        "Example Recipient Fullname"
    );
    assert_eq!(result["orders"][0]["match_score"], 85);
    state
        .collection("alex_submissions")
        .update_one(
            doc! {"_id":bill},
            doc! {"$set":{"customer.name":"Completely Other"}},
        )
        .await
        .unwrap();
    let result = shipping::candidates(&state, "Example Recipient")
        .await
        .unwrap();
    assert_eq!(
        result["customer_matches"][0]["match_source"],
        "customer_profile"
    );
    assert_eq!(result["orders"][0]["match_score"], 100);
    state
        .collection("customer_profiles")
        .update_one(
            doc! {"line_user_id":"note-user"},
            doc! {"$set":{"display_name":"Unrelated Contact"}},
        )
        .await
        .unwrap();
    let result = shipping::candidates(&state, "Example Recipient")
        .await
        .unwrap();
    assert_eq!(result["customer_matches"].as_array().unwrap().len(), 1);
    assert_eq!(
        result["customer_matches"][0]["match_source"],
        "customer_note"
    );
    assert_eq!(result["customer_matches"][0]["match_score"], 80);
    assert_eq!(
        result["customer_matches"][0]["chat_customer"]["display_name"],
        "LINE note-user"
    );
    assert_eq!(
        result["customer_matches"][0]["chat_customer"]["picture_url"],
        "https://example.test/customer.png"
    );
    assert_eq!(
        result["customer_matches"][0]["inquiry_id"],
        inquiry.to_hex()
    );
    assert_eq!(result["orders"][0]["match_score"], 80);
    assert!(
        result["customer_matches"][0]["matched_name"]
            .as_str()
            .unwrap()
            .contains("Example Recipient")
    );
    // A spelling resemblance in free text is not name evidence.
    assert!(
        shipping::candidates(&state, "Exampel Recipient")
            .await
            .unwrap()["customer_matches"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    // Search only current note text, never an older revision or attached images.
    state.collection("customer_notes").update_one(doc! {"_id":note_id},doc! {"$set":{"text":"ติดต่อเรียบร้อยแล้ว","images":[{"name":"Example Recipient.png"}],"history":[{"text":"Example Recipient"}]}}).await.unwrap();
    assert!(
        shipping::candidates(&state, "Example Recipient")
            .await
            .unwrap()["customer_matches"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    state
        .collection("customer_profiles")
        .update_one(
            doc! {"line_user_id":"note-user"},
            doc! {"$set":{"notes":"ผู้รับ Example Recipient แจ้งเปลี่ยนวันรับ"}},
        )
        .await
        .unwrap();
    let result = shipping::candidates(&state, "Example Recipient")
        .await
        .unwrap();
    assert_eq!(
        result["customer_matches"][0]["match_source"],
        "profile_note"
    );
    assert_eq!(result["customer_matches"][0]["match_score"], 80);
    let mut restricted = state.clone();
    restricted.access = Some(crate::models::team::Access {
        member_id: ObjectId::new(),
        line_user_id: "staff".into(),
        name: "Staff".into(),
        role: crate::models::team::Role::Staff,
        branch_ids: vec!["branch-b".into()],
        version: 1,
    });
    let result = shipping::candidates(&restricted, "Example Recipient")
        .await
        .unwrap();
    assert!(result["orders"].as_array().unwrap().is_empty());
    assert_eq!(
        result["customer_matches"][0]["match_source"],
        "profile_note"
    );
    // Notes share the existing shop-wide chat scope; another shop/channel remains isolated.
    note.insert("_id", ObjectId::new());
    note.insert("channel_id", "other-channel");
    note.insert("text", "Other Channel Recipient");
    state
        .collection("customer_notes")
        .insert_one(note.clone())
        .await
        .unwrap();
    note.insert("_id", ObjectId::new());
    note.insert("channel_id", state.config.channel_id.clone());
    note.insert("shop_id", "other-shop");
    note.insert("text", "Other Shop Recipient");
    state
        .collection("customer_notes")
        .insert_one(note)
        .await
        .unwrap();
    for query in ["Other Channel Recipient", "Other Shop Recipient"] {
        let result = shipping::candidates(&state, query).await.unwrap();
        assert!(result["customer_matches"].as_array().unwrap().is_empty());
    }
    let mut other = config();
    other.mongodb_db = state.config.mongodb_db.clone();
    other.channel_id = "unrelated-channel".into();
    let other = AppState::connect(other).await.unwrap();
    assert!(
        shipping::candidates(&other, "Example Recipient")
            .await
            .unwrap()["customer_matches"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    // Looking up a note is read-only and cannot create a tracking or message.
    assert_eq!(
        state
            .collection("shipping_trackings")
            .count_documents(doc! {})
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        state
            .collection("outbound_messages")
            .count_documents(doc! {})
            .await
            .unwrap(),
        0
    );
    state.db.drop().await.unwrap();
}

#[tokio::test]
#[ignore = "requires disposable MongoDB"]
async fn deleted_batches_hide_every_row_block_old_actions_and_preserve_delivery_history() {
    let state = AppState::connect(config()).await.unwrap();
    crate::db::indexes::ensure(&state).await.unwrap();
    let (bill, _) = seed_order(&state, "Customer", "user-a", "AR-001").await;
    let id = ObjectId::new();
    let now = DateTime::now();
    let mut receipt = state.scope();
    receipt.extend(doc! {"_id":id,"request_id":"receipt-request","content_hash":"receipt-image","status":"READY","name":"batch.png","extraction":mongodb::bson::to_document(&extraction()).unwrap(),"created_at":now,"updated_at":now});
    state
        .collection("shipping_receipts")
        .insert_one(receipt)
        .await
        .unwrap();
    let actor = crate::models::customer::Actor {
        id: "admin".into(),
        name: "Admin".into(),
    };
    let input = json!({"expected_version":0,"submission_id":bill.to_hex(),"recipient_name":"Customer","tracking_number":"JD333458790TH","postal_code":"10150","service":"EMS","shipping_fee_satang":2400,"message":"Tracking JD333458790TH"});
    let saved = shipping::save(
        &state,
        id,
        1,
        serde_json::from_value(input.clone()).unwrap(),
        &actor,
    )
    .await
    .unwrap();
    let tracking = ObjectId::parse_str(saved["tracking"]["_id"]["$oid"].as_str().unwrap()).unwrap();
    let mut second = input.clone();
    second["tracking_number"] = json!("JD333458809TH");
    second["message"] = json!("Tracking JD333458809TH");
    shipping::save(
        &state,
        id,
        2,
        serde_json::from_value(second).unwrap(),
        &actor,
    )
    .await
    .unwrap();
    // An active save/send lease prevents deletion by another backend instance.
    state.collection("shipping_receipts").update_one(doc! {"_id":id},doc! {"$set":{"operation_id":ObjectId::new(),"operation_until":DateTime::from_millis(now.timestamp_millis()+60_000)}}).await.unwrap();
    assert!(matches!(
        shipping::delete(&state, id, &actor).await,
        Err(crate::error::AppError::RecordConflict)
    ));
    state
        .collection("shipping_receipts")
        .update_one(
            doc! {"_id":id},
            doc! {"$unset":{"operation_id":"","operation_until":""}},
        )
        .await
        .unwrap();
    let mut restricted = state.clone();
    restricted.access = Some(crate::models::team::Access {
        member_id: ObjectId::new(),
        line_user_id: "staff".into(),
        name: "Staff".into(),
        role: crate::models::team::Role::Staff,
        branch_ids: vec!["branch-b".into()],
        version: 1,
    });
    let mut member = state.scope();
    member.extend(doc! {"_id":restricted.access.as_ref().unwrap().member_id,"status":"ACTIVE","version":1_i64,"created_at":now,"updated_at":now});
    state
        .collection("team_memberships")
        .insert_one(member)
        .await
        .unwrap();
    assert!(matches!(
        shipping::delete(&restricted, id, &actor).await,
        Err(crate::error::AppError::AlexScope)
    ));
    let mut other = config();
    other.mongodb_db = state.config.mongodb_db.clone();
    other.channel_id = "another-channel".into();
    let other = AppState::connect(other).await.unwrap();
    assert!(matches!(
        shipping::delete(&other, id, &actor).await,
        Err(crate::error::AppError::NotFound)
    ));
    let path = format!("/api/admin/shipping/receipts/{id}/delete");
    assert_eq!(
        request(&state, &path, "POST", Value::Null, false).await.0,
        StatusCode::UNAUTHORIZED
    );
    let mut outbound = state.scope();
    outbound.extend(doc! {"_id":ObjectId::new(),"status":"DONE","text":"Tracking JD333458790TH","created_at":now,"updated_at":now});
    state
        .collection("outbound_messages")
        .insert_one(outbound)
        .await
        .unwrap();
    state
        .collection("shipping_trackings")
        .update_one(
            doc! {"_id":tracking},
            doc! {"$set":{"status":"SENT","delivery_started_at":now}},
        )
        .await
        .unwrap();
    assert_eq!(
        request(&state, &path, "POST", Value::Null, true).await.1["deleted"],
        true
    );
    assert_eq!(
        request(&state, &path, "POST", Value::Null, true).await.1["deleted"],
        true
    );
    assert!(matches!(
        shipping::delete(&restricted, id, &actor).await,
        Err(crate::error::AppError::AlexScope)
    ));
    assert!(
        shipping::list(&state, None).await.unwrap()["receipts"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(matches!(
        shipping::get(&state, id).await,
        Err(crate::error::AppError::NotFound)
    ));
    assert!(matches!(
        shipping::image(&state, id).await,
        Err(crate::error::AppError::NotFound)
    ));
    assert!(matches!(
        shipping::save(
            &state,
            id,
            1,
            serde_json::from_value(input).unwrap(),
            &actor
        )
        .await,
        Err(crate::error::AppError::NotFound)
    ));
    assert!(matches!(
        shipping::send(&state, tracking, 1, &actor).await,
        Err(crate::error::AppError::NotFound)
    ));
    assert!(!shipping::work_one(&state).await.unwrap());
    assert!(shipping::extract_again(&state, id, &actor).await.is_err());
    assert_eq!(
        state
            .collection("shipping_trackings")
            .count_documents(doc! {"receipt_id":id})
            .await
            .unwrap(),
        2
    );
    assert_eq!(
        state
            .collection("outbound_messages")
            .count_documents(doc! {})
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        state
            .collection("alex_submissions")
            .count_documents(doc! {"_id":bill})
            .await
            .unwrap(),
        1
    );
    let deleted = state
        .collection("shipping_receipts")
        .find_one(doc! {"_id":id})
        .await
        .unwrap()
        .unwrap();
    assert_eq!(deleted.get_str("deleted_by").unwrap(), "internal-api");
    assert!(!deleted.contains_key("operation_id"));
    // Deletion works before OCR finishes, and the worker cannot resurrect the batch.
    for status in ["QUEUED", "READING", "FAILED"] {
        let id = ObjectId::new();
        let mut row = state.scope();
        row.extend(doc! {"_id":id,"status":status,"request_id":id.to_hex(),"content_hash":id.to_hex(),"created_at":now,"updated_at":now});
        state
            .collection("shipping_receipts")
            .insert_one(row)
            .await
            .unwrap();
        shipping::delete(&state, id, &actor).await.unwrap();
    }
    shipping::recover(&state).await.unwrap();
    assert!(!shipping::work_one(&state).await.unwrap());
    assert!(
        shipping::list(&state, None).await.unwrap()["receipts"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    state.db.drop().await.unwrap();
}

#[tokio::test]
#[ignore = "requires disposable MongoDB"]
async fn tracking_can_send_to_an_explicit_chat_without_confirming_an_alex_bill() {
    let mock = ShippingMock::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = axum::Router::new()
        .route("/v2/bot/message/push", axum::routing::post(push))
        .with_state(mock.clone());
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let mut cfg = config();
    cfg.mongodb_uri = std::env::var("TEST_MONGODB_URI").unwrap_or(cfg.mongodb_uri);
    cfg.line_api_base = format!("http://{address}");
    let state = AppState::connect(cfg).await.unwrap();
    crate::db::indexes::ensure(&state).await.unwrap();
    let (bill, inquiry) = seed_order(&state, "Recipient", "direct-chat-user", "AR-UNKNOWN").await;
    state.collection("alex_submissions").update_one(doc! {"_id":bill}, doc! {"$set":{"status":"SUBMIT_UNKNOWN"},"$unset":{"alex_order_id":"","alex_order_number":""}}).await.unwrap();
    let rid = ObjectId::new();
    let now = DateTime::now();
    let mut receipt = state.scope();
    receipt.extend(doc! {"_id":rid,"status":"READY","name":"test.png","extraction":mongodb::bson::to_document(&extraction()).unwrap(),"created_at":now,"updated_at":now});
    state
        .collection("shipping_receipts")
        .insert_one(receipt)
        .await
        .unwrap();
    let base = "/api/admin/shipping";
    let row = format!("{base}/receipts/{rid}/rows/1");
    let mut input = json!({"expected_version":0,"inquiry_id":inquiry.to_hex(),"recipient_name":"Recipient","tracking_number":"JD333458790TH","postal_code":"10150","service":"EMS","shipping_fee_satang":2400,"message":"Tracking JD333458790TH"});
    let mut other_cfg = config();
    other_cfg.mongodb_uri = state.config.mongodb_uri.clone();
    other_cfg.mongodb_db = state.config.mongodb_db.clone();
    other_cfg.shop_id = "another-shop".into();
    let other = AppState::connect(other_cfg).await.unwrap();
    assert_eq!(
        request(&other, &row, "POST", input.clone(), true).await.0,
        StatusCode::NOT_FOUND
    );
    input["inquiry_id"] = ObjectId::new().to_hex().into();
    assert_eq!(
        request(&state, &row, "POST", input.clone(), true).await.0,
        StatusCode::NOT_FOUND
    );
    input["inquiry_id"] = inquiry.to_hex().into();
    let saved = request(&state, &row, "POST", input.clone(), true).await;
    assert_eq!(saved.0, StatusCode::OK, "{:?}", saved.1);
    assert!(saved.1["tracking"]["submission_id"].is_null());
    assert!(saved.1["tracking"]["alex_order_id"].is_null());
    assert_eq!(saved.1["tracking"]["line_user_id"], "direct-chat-user");
    assert!(mock.pushes.lock().await.is_empty());
    let tracking =
        ObjectId::parse_str(saved.1["tracking"]["_id"]["$oid"].as_str().unwrap()).unwrap();
    let send = format!("{base}/trackings/{tracking}/send");
    let reloaded = shipping::get(&state, rid).await.unwrap();
    assert_eq!(
        reloaded["trackings"][0]["chat_customer"]["display_name"],
        "LINE direct-chat-user"
    );
    assert_eq!(
        request(&other, &send, "POST", json!({"expected_version":1}), true)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    // Stored recipient identity must still match the selected chat at delivery time.
    state
        .collection("shipping_trackings")
        .update_one(
            doc! {"_id":tracking},
            doc! {"$set":{"line_user_id":"wrong-user"}},
        )
        .await
        .unwrap();
    assert_eq!(
        request(&state, &send, "POST", json!({"expected_version":1}), true)
            .await
            .1["error"]["code"],
        "RECORD_CONFLICT"
    );
    assert!(mock.pushes.lock().await.is_empty());
    state
        .collection("shipping_trackings")
        .update_one(
            doc! {"_id":tracking},
            doc! {"$set":{"line_user_id":"direct-chat-user"}},
        )
        .await
        .unwrap();
    let first = request(&state, &send, "POST", json!({"expected_version":1}), true).await;
    assert_eq!(first.1["tracking"]["status"], "SEND_FAILED");
    input["expected_version"] = 1.into();
    assert_eq!(
        request(&state, &row, "POST", input, true).await.1["error"]["code"],
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
    assert_eq!(pushes[0].0, pushes[1].0);
    assert_eq!(pushes[0].1["to"], "direct-chat-user");
    assert_eq!(pushes[0].1["messages"][0]["text"], "Tracking JD333458790TH");
    assert_eq!(
        state
            .collection("alex_submissions")
            .find_one(doc! {"_id":bill})
            .await
            .unwrap()
            .unwrap()
            .get_str("status")
            .unwrap(),
        "SUBMIT_UNKNOWN"
    );
    let actor = crate::models::customer::Actor {
        id: "test".into(),
        name: "Admin".into(),
    };
    shipping::delete(&state, rid, &actor).await.unwrap();
    assert!(matches!(
        shipping::get(&state, rid).await,
        Err(crate::error::AppError::NotFound)
    ));
    assert_eq!(
        state
            .collection("shipping_trackings")
            .count_documents(doc! {"_id":tracking})
            .await
            .unwrap(),
        1
    );
    state.db.drop().await.unwrap();
    server.abort();
}
