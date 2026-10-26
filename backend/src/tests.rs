use crate::{
    config::Config,
    db::{events, inquiries, media},
    models::line::LineEvent,
    routes,
    state::AppState,
    worker,
};
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use hmac::{Hmac, Mac};
use mongodb::bson::{DateTime, Document, doc, oid::ObjectId};
use serde_json::{Value, json};
use sha2::Sha256;
use std::sync::Arc;
use tower::ServiceExt;

#[path = "tests_customers.rs"]
mod customer_tests;
#[path = "tests_line.rs"]
mod line_tests;
#[path = "tests_oa_sync.rs"]
mod oa_sync_tests;
#[path = "tests_products.rs"]
mod product_tests;
#[path = "tests_step2.rs"]
mod step2;

fn config() -> Config {
    Config {
        port: 8080,
        mongodb_uri: "mongodb://127.0.0.1:27018".into(),
        mongodb_db: format!("step1_test_{}", ObjectId::new().to_hex()),
        cors_origin: None,
        shop_id: "test-shop".into(),
        channel_id: "test-channel".into(),
        destination: "test-destination".into(),
        channel_secret: "test-only-secret".into(),
        channel_access_token: "test-only-token".into(),
        admin_token: "test-only-admin".into(),
        burst_ms: 3000,
        media_max_bytes: 20971520,
        media_retention_days: 30,
        admin_username: "admin".into(),
        admin_password: Some("test-only-password".into()),
        admin_cookie_secure: false,
        hermes_url: None,
        hermes_api_key: None,
        hermes_model: "hermes-agent".into(),
        hermes_timeout_seconds: 5,
        line_api_base: "http://127.0.0.1:9".into(),
        line_login_channel_id: None,
        line_login_channel_secret: None,
        line_login_redirect_uri: None,
        admin_line_user_ids: vec![],
        oa_connector_enabled: true,
        public_base_url: Some("https://alex.test".into()),
        alex: None,
        alex_mcp_token: None,
    }
}

fn event(id: &str, user: &str, timestamp: i64) -> LineEvent {
    serde_json::from_value(
        json!({"type":"message", "webhookEventId":id, "timestamp":timestamp,
        "source":{"type":"user", "userId":user},
        "message":{"id":format!("100{timestamp}"), "type":"text", "text":format!("text-{id}")}}),
    )
    .unwrap()
}

fn sign(secret: &str, body: &[u8]) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(body);
    STANDARD.encode(mac.finalize().into_bytes())
}

async fn request(
    state: &AppState,
    path: &str,
    method: &str,
    body: Value,
    auth: bool,
) -> (StatusCode, Value) {
    let bytes = serde_json::to_vec(&body).unwrap();
    let mut request = Request::builder()
        .uri(path)
        .method(method)
        .header("content-type", "application/json");
    if path == "/api/line/webhook" {
        request = request.header(
            "x-line-signature",
            sign(&state.config.channel_secret, &bytes),
        );
    }
    if auth {
        request = request.header(
            "authorization",
            format!("Bearer {}", state.config.admin_token),
        );
    }
    let response = routes::router(state.clone())
        .unwrap()
        .oneshot(request.body(Body::from(bytes)).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 5_000_000).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[test]
fn signatures_cover_exact_raw_body() {
    let body = br#"{ "destination":"test-destination", "events":[] }"#;
    let signature = sign("test-only-secret", body);
    assert!(routes::webhook::verify("test-only-secret", &signature, body).is_ok());
    assert!(routes::webhook::verify("test-only-secret", &signature, b"{}").is_err());
    assert!(routes::webhook::verify("wrong", &signature, body).is_err());
    assert!(routes::webhook::verify("test-only-secret", "malformed", body).is_err());
}

#[test]
fn conversation_identity_separates_group_members_and_rooms() {
    let mut first = event("a", "user-a", 1000);
    let mut second = first.clone();
    second.source.as_mut().unwrap().user_id = Some("user-b".into());
    assert_ne!(first.conversation_key(), second.conversation_key());
    first.source.as_mut().unwrap().kind = "group".into();
    assert!(first.conversation_key().is_none());
    first.source.as_mut().unwrap().group_id = Some("group-a".into());
    let group_a = first.conversation_key();
    first.source.as_mut().unwrap().group_id = Some("group-b".into());
    assert_ne!(group_a, first.conversation_key());
    first.source.as_mut().unwrap().user_id = None;
    assert!(first.conversation_key().is_none());
}

#[test]
fn burst_batches_follow_event_time_and_preserve_gaps() {
    let records = [1000, 2000, 4999, 8000]
        .into_iter()
        .map(|t| doc! {"event_at":DateTime::from_millis(t)})
        .collect();
    let result = inquiries::batches(records, 3000);
    assert_eq!(result.iter().map(Vec::len).collect::<Vec<_>>(), vec![3, 1]);
}

#[tokio::test]
async fn public_routes_verify_before_database_access() {
    // Construct a lazy Mongo client; these requests must never touch the database.
    let config = config();
    let client = mongodb::Client::with_uri_str(&config.mongodb_uri)
        .await
        .unwrap();
    let state = AppState {
        db: client.database(&config.mongodb_db),
        config: Arc::new(config),
        http: reqwest::Client::new(),
        login_attempts: Arc::new(tokio::sync::Mutex::new((0, std::time::Instant::now()))),
    };
    let (status, body) = request(&state, "/api/health", "GET", Value::Null, false).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({"status":"ok"}));
    let (status, body) = request(&state, "/api/health", "POST", Value::Null, false).await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(body["error"]["code"], "METHOD_NOT_ALLOWED");
    let (status, body) = request(
        &state,
        "/api/line/webhook",
        "POST",
        json!({"destination":"test-destination", "events":[]}),
        false,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["accepted"], 0);
    let (status, _) = request(
        &state,
        "/api/line/webhook",
        "POST",
        json!({"destination":"other-channel", "events":[]}),
        false,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = request(&state, "/api/inquiries", "GET", Value::Null, false).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    for path in ["/api/line/users", "/api/line/users/customer/messages"] {
        assert_eq!(
            request(&state, path, "GET", Value::Null, false).await.0,
            StatusCode::UNAUTHORIZED
        );
    }
    let response = routes::router(state.clone())
        .unwrap()
        .oneshot(
            Request::builder()
                .uri("/api/line/webhook")
                .method("POST")
                .body(Body::from("not-json"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let (status, _) = request(&state, "/api/line/webhook", "POST", json!({"destination":"test-destination", "events":[{"type":"message", "webhookEventId":"bad", "timestamp":1}]}), false).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn webhook_does_not_acknowledge_database_failure() {
    let config = config();
    let mut options = mongodb::options::ClientOptions::parse("mongodb://127.0.0.1:1")
        .await
        .unwrap();
    options.server_selection_timeout = Some(std::time::Duration::from_millis(100));
    let client = mongodb::Client::with_options(options).unwrap();
    let state = AppState {
        db: client.database(&config.mongodb_db),
        config: Arc::new(config),
        http: reqwest::Client::new(),
        login_attempts: Arc::new(tokio::sync::Mutex::new((0, std::time::Instant::now()))),
    };
    let (status, response) = request(
        &state,
        "/api/line/webhook",
        "POST",
        json!({"destination":"test-destination", "events":[event("durability", "customer", 1)]}),
        false,
    )
    .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(response["error"]["code"], "DATABASE_UNAVAILABLE");
}

async fn drain(state: &AppState) {
    while let Some(record) = events::claim(state).await.unwrap() {
        worker::process(state, &record).await.unwrap();
    }
}

async fn all(state: &AppState, collection: &str) -> Vec<Document> {
    use futures_util::TryStreamExt;
    state
        .collection(collection)
        .find(state.scope())
        .await
        .unwrap()
        .try_collect()
        .await
        .unwrap()
}

#[tokio::test]
#[ignore = "requires MongoDB; set TEST_MONGODB_URI and run cargo test -- --ignored"]
async fn mongo_end_to_end_inquiry_queue() {
    let mut config = config();
    config.mongodb_uri = std::env::var("TEST_MONGODB_URI").unwrap_or(config.mongodb_uri);
    let state = AppState::connect(config).await.unwrap();
    crate::db::indexes::ensure(&state).await.unwrap();
    crate::db::indexes::ensure(&state).await.unwrap();
    let late = event("late", "customer-a", 4000);
    let early = event("early", "customer-a", 1000);
    let (status, response) = request(
        &state,
        "/api/line/webhook",
        "POST",
        json!({"destination":"test-destination", "events":[late, early]}),
        false,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["accepted"], 2);
    assert_eq!(all(&state, "line_events").await.len(), 2);
    assert!(all(&state, "inquiries").await.is_empty()); // ACK precedes all worker processing.
    assert!(!events::persist(&state, &early).await.unwrap());
    // A crash after claim is recovered by an expired lease.
    let claimed = events::claim(&state).await.unwrap().unwrap();
    state
        .collection("line_events")
        .update_one(
            doc! {"_id":claimed.get_object_id("_id").unwrap()},
            doc! {"$set":{"available_at":DateTime::from_millis(0)}},
        )
        .await
        .unwrap();
    drain(&state).await;
    assert_eq!(all(&state, "inquiries").await.len(), 1);
    let inquiries = all(&state, "inquiries").await;
    let inquiry_id = inquiries[0].get_object_id("_id").unwrap();
    let path = format!("/api/inquiries/{inquiry_id}/context");
    let (status, context) = request(&state, &path, "GET", Value::Null, true).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(context["batches"][0][0]["event_id"], "early");
    assert_eq!(context["batches"][0][1]["event_id"], "late");
    let jobs = all(&state, "analysis_jobs").await;
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].get_array("event_ids").unwrap().len(), 2);
    // Concurrent redeliveries and workers share the same unique event/inquiry records.
    let concurrent = event("concurrent", "parallel-customer", 4500);
    let (first, second) = tokio::join!(
        events::persist(&state, &concurrent),
        events::persist(&state, &concurrent)
    );
    assert_eq!(
        usize::from(first.unwrap()) + usize::from(second.unwrap()),
        1
    );
    let parallel = event("parallel", "parallel-customer", 4600);
    events::persist(&state, &parallel).await.unwrap();
    let (first, second) = tokio::join!(events::claim(&state), events::claim(&state));
    let first = first.unwrap().unwrap();
    let second = second.unwrap().unwrap();
    assert_ne!(
        first.get_object_id("_id").unwrap(),
        second.get_object_id("_id").unwrap()
    );
    assert!(events::renew(&state, &first).await.unwrap());
    let (first_result, second_result) = tokio::join!(
        worker::process(&state, &first),
        worker::process(&state, &second)
    );
    first_result.unwrap();
    second_result.unwrap();
    assert_eq!(all(&state, "inquiries").await.len(), 2);
    assert!(!events::renew(&state, &first).await.unwrap());
    // Postback date parameters remain available as original evidence.
    let postback: LineEvent = serde_json::from_value(
        json!({"type":"postback", "webhookEventId":"postback", "timestamp":4700,
        "source":{"type":"user", "userId":"postback-customer"},
        "postback":{"data":"select-date", "params":{"date":"2026-11-15"}}}),
    )
    .unwrap();
    events::persist(&state, &postback).await.unwrap();
    drain(&state).await;
    let saved = all(&state, "line_events")
        .await
        .into_iter()
        .find(|e| e.get_str("event_id").ok() == Some("postback"))
        .unwrap();
    assert_eq!(
        saved
            .get_document("payload")
            .unwrap()
            .get_document("postback")
            .unwrap()
            .get_document("params")
            .unwrap()
            .get_str("date")
            .unwrap(),
        "2026-11-15"
    );
    // Tenant scoping applies to reads and webhook deduplication.
    let mut other_config = (*state.config).clone();
    other_config.shop_id = "other-shop".into();
    let other = AppState {
        config: Arc::new(other_config),
        ..state.clone()
    };
    assert_eq!(
        request(&other, &path, "GET", Value::Null, true).await.0,
        StatusCode::NOT_FOUND
    );
    assert!(events::persist(&other, &early).await.unwrap());
    drain(&other).await;
    assert_eq!(all(&other, "inquiries").await.len(), 1);
    // Multiple open inquiries require an explicit admin selection.
    let conversation = inquiries[0].get_object_id("conversation_id").unwrap();
    let (status, new_inquiry) = request(
        &state,
        &format!("/api/conversations/{conversation}/inquiries"),
        "POST",
        Value::Null,
        true,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let selected = new_inquiry["_id"]["$oid"].as_str().unwrap();
    events::persist(&state, &event("ambiguous", "customer-a", 5000))
        .await
        .unwrap();
    drain(&state).await;
    let ambiguous = all(&state, "line_events")
        .await
        .into_iter()
        .find(|e| e.get_str("event_id").ok() == Some("ambiguous"))
        .unwrap();
    assert_eq!(ambiguous.get_str("status").unwrap(), "NEEDS_ADMIN");
    let assignment_path = format!(
        "/api/line/events/{}/assign",
        ambiguous.get_object_id("_id").unwrap()
    );
    assert_eq!(
        request(
            &state,
            &assignment_path,
            "POST",
            json!({"inquiry_id":selected}),
            true
        )
        .await
        .0,
        StatusCode::OK
    );
    drain(&state).await;
    // Image processing uses an existing private GridFS reference on retry.
    let mut image = event("image", "image-customer", 6000);
    image.message =
        Some(json!({"id":"123456789", "type":"image", "contentProvider":{"type":"line"}}));
    events::persist(&state, &image).await.unwrap();
    let record = events::claim(&state).await.unwrap().unwrap();
    let mut metadata = state.scope();
    metadata.insert("event_id", record.get_object_id("_id").unwrap());
    let media_id = media::store(
        &state,
        metadata,
        "123456789",
        "image/png",
        b"test-image-bytes",
    )
    .await
    .unwrap();
    worker::process(&state, &record).await.unwrap();
    assert_eq!(
        media::read(&state, media_id).await.unwrap().1,
        b"test-image-bytes"
    );
    assert!(media::read(&other, media_id).await.is_err());
    let unsend: LineEvent = serde_json::from_value(
        json!({"type":"unsend", "webhookEventId":"unsend", "timestamp":7000,
        "source":{"type":"user", "userId":"image-customer"}, "unsend":{"messageId":"123456789"}}),
    )
    .unwrap();
    events::persist(&state, &unsend).await.unwrap();
    drain(&state).await;
    assert!(media::read(&state, media_id).await.is_err());
    let image = all(&state, "line_events")
        .await
        .into_iter()
        .find(|e| e.get_str("event_id").ok() == Some("image"))
        .unwrap();
    assert!(image.get_bool("withdrawn").unwrap());
    assert!(
        !image
            .get_document("payload")
            .unwrap()
            .contains_key("message")
    );
    // Out-of-order unsends also suppress a message arriving later.
    media::tombstone(&state, "1008000").await.unwrap();
    events::persist(&state, &event("already-withdrawn", "customer-b", 8000))
        .await
        .unwrap();
    drain(&state).await;
    let withdrawn = all(&state, "line_events")
        .await
        .into_iter()
        .find(|e| e.get_str("event_id").ok() == Some("already-withdrawn"))
        .unwrap();
    assert!(withdrawn.get_bool("withdrawn").unwrap());
    // Failed work is observable, bounded, and can be explicitly retried by an admin.
    events::persist(&state, &event("failure", "failure-customer", 8500))
        .await
        .unwrap();
    let mut failed = events::claim(&state).await.unwrap().unwrap();
    failed.insert("attempts", 8);
    events::retry(&state, &failed, "MEDIA_UNAVAILABLE")
        .await
        .unwrap();
    let retry_path = format!(
        "/api/line/events/{}/retry",
        failed.get_object_id("_id").unwrap()
    );
    let (status, _) = request(&state, &retry_path, "POST", Value::Null, true).await;
    assert_eq!(status, StatusCode::OK);
    drain(&state).await;
    // The queue survives a fresh connection (no in-memory channel dependency).
    events::persist(&state, &event("restart", "customer-c", 9000))
        .await
        .unwrap();
    let reconnected = AppState::connect((*state.config).clone()).await.unwrap();
    drain(&reconnected).await;
    assert_eq!(
        all(&state, "line_events")
            .await
            .into_iter()
            .find(|e| e.get_str("event_id").ok() == Some("restart"))
            .unwrap()
            .get_str("status")
            .unwrap(),
        "DONE"
    );
    assert!(state.config.mongodb_db.starts_with("step1_test_"));
    state.db.drop().await.unwrap();
}

#[tokio::test]
#[ignore = "requires MongoDB; set TEST_MONGODB_URI and run cargo test -- --ignored"]
async fn mongo_user_registry_and_chat_history() {
    let mut config = config();
    config.mongodb_uri = std::env::var("TEST_MONGODB_URI").unwrap_or(config.mongodb_uri);
    let state = AppState::connect(config).await.unwrap();
    crate::db::indexes::ensure(&state).await.unwrap();
    crate::db::indexes::ensure(&state).await.unwrap();

    let follow: LineEvent = serde_json::from_value(json!({
        "type":"follow", "webhookEventId":"follow", "timestamp":100,
        "source":{"type":"user", "userId":"customer-a"}
    }))
    .unwrap();
    let webhook = json!({"destination":"test-destination", "events":[follow]});
    // A partial write must not be ACKed; the same event repairs registration on redelivery.
    state
        .db
        .run_command(doc! {"collMod":"line_users", "validator":{"line_user_id":"reject-test-user"}})
        .await
        .unwrap();
    assert_eq!(
        request(&state, "/api/line/webhook", "POST", webhook.clone(), false)
            .await
            .0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(all(&state, "line_events").await.len(), 1);
    assert!(all(&state, "line_users").await.is_empty());
    state
        .db
        .run_command(doc! {"collMod":"line_users", "validator":{}})
        .await
        .unwrap();
    let (status, response) = request(&state, "/api/line/webhook", "POST", webhook, false).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["accepted"], 0);
    assert_eq!(all(&state, "line_users").await.len(), 1);
    assert!(all(&state, "inquiries").await.is_empty());

    let late = event("later", "customer-a", 500);
    let early = event("earlier", "customer-a", 50);
    events::persist(&state, &late).await.unwrap();
    events::persist(&state, &early).await.unwrap();
    events::persist(&state, &late).await.unwrap();
    let users = all(&state, "line_users").await;
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].get_str("line_user_id").unwrap(), "customer-a");
    assert_eq!(
        users[0]
            .get_datetime("first_seen_at")
            .unwrap()
            .timestamp_millis(),
        50
    );
    assert_eq!(
        users[0]
            .get_datetime("last_seen_at")
            .unwrap()
            .timestamp_millis(),
        500
    );
    assert!(users[0].get_datetime("created_at").is_ok());
    assert!(users[0].get_datetime("updated_at").is_ok());

    let mut group = event("group-a", "customer-a", 500);
    group.source.as_mut().unwrap().kind = "group".into();
    group.source.as_mut().unwrap().group_id = Some("shared-group".into());
    events::persist(&state, &group).await.unwrap();
    group.webhook_event_id = "group-b".into();
    group.source.as_mut().unwrap().user_id = Some("customer-b".into());
    events::persist(&state, &group).await.unwrap();
    group.webhook_event_id = "no-user".into();
    group.source.as_mut().unwrap().user_id = None;
    events::persist(&state, &group).await.unwrap();
    assert_eq!(all(&state, "line_users").await.len(), 2);

    let path = "/api/line/users/customer-a/messages";
    let (status, response) = request(&state, path, "GET", Value::Null, true).await;
    assert_eq!(status, StatusCode::OK);
    let messages = response["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 3); // Follow events are not chat messages.
    assert_eq!(messages[0]["event_id"], "earlier");
    assert_eq!(messages[1]["event_id"], "group-a"); // Timestamp ties use event_id.
    assert_eq!(messages[2]["event_id"], "later");
    assert_eq!(messages[0]["status"], "PENDING"); // History is available before processing.
    let cursor = messages[0]["_id"]["$oid"].as_str().unwrap();
    let (_, next_page) = request(
        &state,
        &format!("{path}?after={cursor}"),
        "GET",
        Value::Null,
        true,
    )
    .await;
    assert_eq!(next_page["messages"].as_array().unwrap().len(), 2);
    assert_eq!(
        request(
            &state,
            &format!("/api/line/users/customer-b/messages?after={cursor}"),
            "GET",
            Value::Null,
            true
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    for bad in [
        "/api/line/users?before=bad",
        "/api/line/users/customer-a/messages?after=bad",
    ] {
        assert_eq!(
            request(&state, bad, "GET", Value::Null, true).await.0,
            StatusCode::BAD_REQUEST
        );
    }

    // Scope both user registration and history by shop and channel, even for the same user ID.
    for (shop, channel) in [
        ("other-shop", "test-channel"),
        ("test-shop", "other-channel"),
    ] {
        let mut config = (*state.config).clone();
        config.shop_id = shop.into();
        config.channel_id = channel.into();
        let other = AppState {
            config: Arc::new(config),
            ..state.clone()
        };
        let (_, list) = request(&other, "/api/line/users", "GET", Value::Null, true).await;
        assert!(list["users"].as_array().unwrap().is_empty());
        let (_, history) = request(&other, path, "GET", Value::Null, true).await;
        assert!(history["messages"].as_array().unwrap().is_empty());
        events::persist(&other, &late).await.unwrap();
        assert_eq!(all(&other, "line_users").await.len(), 1);
        let (_, history) = request(&other, path, "GET", Value::Null, true).await;
        assert_eq!(history["messages"].as_array().unwrap().len(), 1);
    }

    // Redact an unsent message before purge runs, and keep the history cursor usable after purge.
    media::tombstone(&state, "10050").await.unwrap();
    let (_, history) = request(&state, path, "GET", Value::Null, true).await;
    assert_eq!(history["messages"][0]["withdrawn"], true);
    assert!(history["messages"][0].get("payload").is_none());
    media::purge_message(&state, "10050").await.unwrap();
    assert_eq!(
        request(
            &state,
            &format!("{path}?after={cursor}"),
            "GET",
            Value::Null,
            true
        )
        .await
        .0,
        StatusCode::OK
    );

    // A full page never repeats or drops the boundary item, including timestamp ties.
    let mut fixtures = Vec::new();
    for i in 0..201 {
        let event = event(&format!("page-{i:03}"), "paged-customer", 1000);
        let mut record = state.scope();
        let now = DateTime::now();
        record.extend(doc! {"_id":ObjectId::new(), "event_id":&event.webhook_event_id,
            "event_at":DateTime::from_millis(1000), "payload":mongodb::bson::to_document(&event).unwrap(),
            "status":"FAILED", "lease_token":ObjectId::new(), "created_at":now, "updated_at":now});
        fixtures.push(record);
    }
    // Legacy events remain readable even without a line_users record.
    state
        .collection("line_events")
        .insert_many(fixtures)
        .await
        .unwrap();
    let page_path = "/api/line/users/paged-customer/messages";
    let (_, first) = request(&state, page_path, "GET", Value::Null, true).await;
    assert_eq!(first["messages"].as_array().unwrap().len(), 200);
    assert_eq!(first["messages"][199]["event_id"], "page-199");
    assert!(first["messages"][0].get("lease_token").is_none());
    let next = first["next"].as_str().unwrap();
    let (_, second) = request(
        &state,
        &format!("{page_path}?after={next}"),
        "GET",
        Value::Null,
        true,
    )
    .await;
    assert_eq!(second["messages"].as_array().unwrap().len(), 1);
    assert_eq!(second["messages"][0]["event_id"], "page-200");
    assert!(second["next"].is_null());

    let mut fixtures = Vec::new();
    for i in 0..101 {
        let mut user = state.scope();
        let now = DateTime::now();
        user.extend(
            doc! {"_id":ObjectId::new(), "line_user_id":format!("fixture-user-{i}"),
            "first_seen_at":now, "last_seen_at":now, "created_at":now, "updated_at":now},
        );
        fixtures.push(user);
    }
    state
        .collection("line_users")
        .insert_many(fixtures)
        .await
        .unwrap();
    let (_, first) = request(&state, "/api/line/users", "GET", Value::Null, true).await;
    assert_eq!(first["users"].as_array().unwrap().len(), 100);
    let next = first["next"].as_str().unwrap();
    let (_, second) = request(
        &state,
        &format!("/api/line/users?before={next}"),
        "GET",
        Value::Null,
        true,
    )
    .await;
    assert_eq!(second["users"].as_array().unwrap().len(), 3);
    assert!(second["next"].is_null());
    let reconnected = AppState::connect((*state.config).clone()).await.unwrap();
    assert_eq!(all(&reconnected, "line_users").await.len(), 103);
    assert!(state.config.mongodb_db.starts_with("step1_test_"));
    state.db.drop().await.unwrap();
}
