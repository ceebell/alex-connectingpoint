use super::*;
use axum::{
    Form, Json as AxumJson, Router,
    extract::State as AxumState,
    http::HeaderMap,
    response::{IntoResponse, Response},
    routing::post,
};
use std::collections::{HashMap, HashSet};
use tokio::sync::Mutex;

#[derive(Clone, Default)]
struct MockLine {
    pushes: Arc<Mutex<Vec<(String, Value)>>>,
    accepted: Arc<Mutex<HashSet<String>>>,
    failed_once: Arc<Mutex<HashSet<String>>>,
    profiles: Arc<Mutex<Vec<String>>>,
}

async fn profile(
    AxumState(mock): AxumState<MockLine>,
    axum::extract::Path(user): axum::extract::Path<String>,
    headers: HeaderMap,
) -> Response {
    if headers["authorization"] != "Bearer test-only-token" {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    mock.profiles.lock().await.push(user.clone());
    if user == "Ublocked" {
        return StatusCode::NOT_FOUND.into_response();
    }
    AxumJson(json!({"userId":user,"displayName":"คุณบี ลูกค้า","pictureUrl":"https://profile.line-scdn.net/0h-test","statusMessage":"private"}))
        .into_response()
}

async fn token(Form(form): Form<HashMap<String, String>>) -> Response {
    if form.get("client_secret").map(String::as_str) != Some("login-secret")
        || form.get("redirect_uri").map(String::as_str)
            != Some("http://localhost:3000/api/admin/line/callback")
    {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let code = form.get("code").cloned().unwrap_or_default();
    AxumJson(json!({"id_token":format!("{code}-token"),"access_token":"unused"})).into_response()
}

async fn verify(Form(form): Form<HashMap<String, String>>) -> Response {
    if form.get("nonce").is_none_or(|n| n.len() < 20) {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let sub = match form.get("id_token").map(String::as_str) {
        Some("good-code-token") => "Uadmin1",
        Some("stranger-code-token") => "Ustranger",
        _ => return StatusCode::BAD_REQUEST.into_response(),
    };
    AxumJson(json!({"sub":sub,"aud":"login-channel","name":"แอดมินหนึ่ง"})).into_response()
}

async fn push(
    AxumState(mock): AxumState<MockLine>,
    headers: HeaderMap,
    AxumJson(body): AxumJson<Value>,
) -> Response {
    if headers["authorization"] != "Bearer test-only-token" {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let key = headers["x-line-retry-key"].to_str().unwrap().to_string();
    let text = body["messages"][0]["text"]
        .as_str()
        .unwrap_or("")
        .to_string();
    if text.contains("quota") {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }
    if text.contains("fail-once") && mock.failed_once.lock().await.insert(key.clone()) {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    if !mock.accepted.lock().await.insert(key.clone()) {
        return StatusCode::CONFLICT.into_response();
    }
    let mut pushes = mock.pushes.lock().await;
    pushes.push((key, body));
    AxumJson(json!({"sentMessages":[{"id":format!("sent-{}",pushes.len()),"quoteToken":"q"}]}))
        .into_response()
}

async fn mock_line() -> (String, MockLine) {
    let mock = MockLine::default();
    let app = Router::new()
        .route("/oauth2/v2.1/token", post(token))
        .route("/oauth2/v2.1/verify", post(verify))
        .route("/v2/bot/message/push", post(push))
        .route("/v2/bot/profile/{user}", axum::routing::get(profile))
        .with_state(mock.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (format!("http://{address}"), mock)
}

fn line_config(base: &str) -> Config {
    let mut cfg = config();
    cfg.line_api_base = base.into();
    cfg.line_login_channel_id = Some("login-channel".into());
    cfg.line_login_channel_secret = Some("login-secret".into());
    cfg.line_login_redirect_uri = Some("http://localhost:3000/api/admin/line/callback".into());
    cfg.admin_line_user_ids = vec!["Uadmin1".into()];
    cfg
}

async fn lazy_state(cfg: Config) -> AppState {
    let client = mongodb::Client::with_uri_str(&cfg.mongodb_uri)
        .await
        .unwrap();
    AppState {
        db: client.database(&cfg.mongodb_db),
        config: Arc::new(cfg),
        http: reqwest::Client::new(),
        login_attempts: Arc::new(tokio::sync::Mutex::new((0, std::time::Instant::now()))),
    }
}

async fn get(state: &AppState, uri: &str, cookie: Option<&str>) -> axum::response::Response {
    let mut request = Request::builder().uri(uri);
    if let Some(cookie) = cookie {
        request = request.header("cookie", cookie);
    }
    routes::router(state.clone())
        .unwrap()
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap()
}

fn location(response: &axum::response::Response) -> String {
    response.headers()["location"].to_str().unwrap().to_string()
}

fn set_cookie(response: &axum::response::Response, name: &str) -> String {
    response
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|v| v.to_str().unwrap())
        .find(|v| v.starts_with(&format!("{name}=")))
        .unwrap()
        .to_string()
}

#[tokio::test]
async fn line_login_allows_only_listed_admins_and_revokes_on_removal() {
    let (base, _) = mock_line().await;
    let state = lazy_state(line_config(&base)).await;
    let options = get(&state, "/api/admin/login-options", None).await;
    let body: Value =
        serde_json::from_slice(&to_bytes(options.into_body(), 1000).await.unwrap()).unwrap();
    assert_eq!(body, json!({"line":true,"password":true}));

    async fn start(state: &AppState) -> (String, String) {
        let response = get(state, "/api/admin/line/start", None).await;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let url = reqwest::Url::parse(&location(&response)).unwrap();
        assert_eq!(url.host_str(), Some("access.line.me"));
        let query: HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(query["client_id"], "login-channel");
        assert_eq!(query["scope"], "openid profile");
        assert!(query["nonce"].len() >= 20);
        let cookie = set_cookie(&response, "alex_line_state");
        assert!(cookie.contains("HttpOnly") && cookie.contains("SameSite=Lax"));
        assert!(cookie.contains("Path=/api/admin/line"));
        (
            query["state"].clone(),
            cookie.split(';').next().unwrap().to_string(),
        )
    }

    // Missing or mismatched state cookie never reaches the token endpoint.
    let (login_state, cookie) = start(&state).await;
    let callback = format!("/api/admin/line/callback?code=good-code&state={login_state}");
    let denied = get(&state, &callback, None).await;
    assert_eq!(location(&denied), "/?login_error=LINE_STATE_INVALID");
    let (other_state, _) = start(&state).await;
    let mismatch = format!("/api/admin/line/callback?code=good-code&state={other_state}");
    assert_eq!(
        location(&get(&state, &mismatch, Some(&cookie)).await),
        "/?login_error=LINE_STATE_INVALID"
    );
    assert_eq!(
        location(
            &get(
                &state,
                "/api/admin/line/callback?error=access_denied",
                Some(&cookie)
            )
            .await
        ),
        "/?login_error=LINE_CANCELLED"
    );

    // A LINE account outside ADMIN_LINE_USER_IDS gets its ID to hand to the owner.
    let stranger = format!("/api/admin/line/callback?code=stranger-code&state={login_state}");
    let response = get(&state, &stranger, Some(&cookie)).await;
    assert_eq!(
        location(&response),
        "/?login_error=LINE_NOT_ALLOWED&line_user_id=Ustranger"
    );
    assert!(
        response
            .headers()
            .get_all("set-cookie")
            .iter()
            .all(|v| !v.to_str().unwrap().starts_with("alex_admin="))
    );

    let response = get(&state, &callback, Some(&cookie)).await;
    assert_eq!(location(&response), "/");
    let session = set_cookie(&response, "alex_admin");
    assert!(session.contains("HttpOnly") && session.contains("SameSite=Strict"));
    assert!(!session.contains("login-secret") && !session.contains("test-only-admin"));
    let session = session.split(';').next().unwrap().to_string();
    let info = get(&state, "/api/admin/session", Some(&session)).await;
    assert_eq!(info.status(), StatusCode::OK);
    let info: Value =
        serde_json::from_slice(&to_bytes(info.into_body(), 1000).await.unwrap()).unwrap();
    assert_eq!(info["username"], "แอดมินหนึ่ง");
    assert_eq!(info["login"], "line");

    // Removing the admin from the allowlist revokes the existing session.
    let mut revoked = line_config(&base);
    revoked.admin_line_user_ids = vec!["Uother".into()];
    let revoked = lazy_state(revoked).await;
    assert_eq!(
        get(&revoked, "/api/admin/session", Some(&session))
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );

    // Without ADMIN_PASSWORD only LINE Login is offered.
    let mut line_only = line_config(&base);
    line_only.admin_password = None;
    let line_only = lazy_state(line_only).await;
    let options = get(&line_only, "/api/admin/login-options", None).await;
    let body: Value =
        serde_json::from_slice(&to_bytes(options.into_body(), 1000).await.unwrap()).unwrap();
    assert_eq!(body, json!({"line":true,"password":false}));
    let mut unconfigured = config();
    unconfigured.line_login_channel_id = None;
    let unconfigured = lazy_state(unconfigured).await;
    assert_eq!(
        location(&get(&unconfigured, "/api/admin/line/start", None).await),
        "/?login_error=LINE_NOT_CONFIGURED"
    );
}

#[tokio::test]
async fn reply_route_checks_input_before_database_access() {
    let state = lazy_state(config()).await;
    let path = format!("/api/inquiries/{}/messages", ObjectId::new().to_hex());
    assert_eq!(
        request(
            &state,
            &path,
            "POST",
            json!({"request_id":"x","text":"hi"}),
            false
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        request(&state, &path, "POST", json!({"text":"hi"}), true)
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    let (status, error) = request(
        &state,
        &path,
        "POST",
        json!({"request_id":"not-uuid","text":"hi"}),
        true,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(error["error"]["code"], "VALIDATION_ERROR");
}

#[tokio::test]
#[ignore = "requires disposable MongoDB; run cargo test -- --ignored"]
async fn mongo_replies_are_pushed_once_recorded_and_join_the_context() {
    let (base, mock) = mock_line().await;
    let mut cfg = line_config(&base);
    cfg.mongodb_uri = std::env::var("TEST_MONGODB_URI").unwrap_or(cfg.mongodb_uri);
    cfg.burst_ms = 0;
    let state = AppState::connect(cfg).await.unwrap();
    crate::db::indexes::ensure(&state).await.unwrap();
    events::persist(&state, &event("reply-start", "Ureply", 1))
        .await
        .unwrap();
    drain(&state).await;
    let inquiry = all(&state, "inquiries").await.remove(0);
    let id = inquiry.get_object_id("_id").unwrap().to_hex();
    let path = format!("/api/inquiries/{id}/messages");
    let drafts_path = format!("/api/inquiries/{id}/drafts");
    let before = request(&state, &drafts_path, "GET", json!(null), true)
        .await
        .1["context_hash"]
        .clone();

    let body =
        json!({"request_id":"11111111-1111-4111-8111-111111111111","text":" ได้ค่ะ ชุดว่างวันที่ 15 "});
    let (status, sent) = request(&state, &path, "POST", body.clone(), true).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(sent["message"]["status"], "DONE");
    assert_eq!(sent["message"]["delivery_status"], "SENT");
    assert_eq!(sent["message"]["line_message_id"], "sent-1");
    assert_eq!(sent["message"]["sender_name"], "internal-api");
    assert_eq!(
        sent["message"]["payload"]["message"]["text"],
        "ได้ค่ะ ชุดว่างวันที่ 15"
    );
    {
        let pushes = mock.pushes.lock().await;
        assert_eq!(pushes.len(), 1);
        assert_eq!(pushes[0].0, "11111111-1111-4111-8111-111111111111");
        assert_eq!(pushes[0].1["to"], "Ureply");
    }
    // Same request is idempotent; the same ID with other text is refused.
    let (status, again) = request(&state, &path, "POST", body, true).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(again["message"]["_id"], sent["message"]["_id"]);
    let changed = json!({"request_id":"11111111-1111-4111-8111-111111111111","text":"อื่น"});
    let (status, error) = request(&state, &path, "POST", changed, true).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error["error"]["code"], "RECORD_CONFLICT");
    assert_eq!(mock.pushes.lock().await.len(), 1);

    let (_, context) = request(
        &state,
        &format!("/api/inquiries/{id}/context"),
        "GET",
        json!(null),
        true,
    )
    .await;
    let flat: Vec<&Value> = context["batches"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|b| b.as_array().unwrap())
        .collect();
    assert_eq!(flat.len(), 2);
    assert_eq!(flat[1]["source"], "alex_push");
    assert_eq!(flat[1]["sender_role"], "admin");
    let after = request(&state, &drafts_path, "GET", json!(null), true)
        .await
        .1["context_hash"]
        .clone();
    assert_ne!(before, after);

    // A failed push is recorded, hidden from the chat and retried with the same key.
    let flaky = json!({"request_id":"22222222-2222-4222-8222-222222222222","text":"fail-once"});
    let (status, error) = request(&state, &path, "POST", flaky.clone(), true).await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
    assert_eq!(error["error"]["code"], "LINE_UNAVAILABLE");
    let (_, context) = request(
        &state,
        &format!("/api/inquiries/{id}/context"),
        "GET",
        json!(null),
        true,
    )
    .await;
    assert_eq!(
        context["batches"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|b| b.as_array().unwrap())
            .count(),
        2
    );
    let (status, retried) = request(&state, &path, "POST", flaky, true).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(retried["message"]["status"], "DONE");
    let (status, error) = request(
        &state,
        &path,
        "POST",
        json!({"request_id":"33333333-3333-4333-8333-333333333333","text":"quota"}),
        true,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
    assert_eq!(error["error"]["code"], "LINE_RATE_LIMITED");
    assert_eq!(mock.pushes.lock().await.len(), 2);

    // Group chats are never pushed to the member's user ID.
    let group: LineEvent = serde_json::from_value(
        json!({"type":"message","webhookEventId":"group-1","timestamp":5,
        "source":{"type":"group","groupId":"Cgroup","userId":"Ureply"},
        "message":{"id":"group-msg","type":"text","text":"hi group"}}),
    )
    .unwrap();
    events::persist(&state, &group).await.unwrap();
    drain(&state).await;
    let group_inquiry = all(&state, "inquiries")
        .await
        .into_iter()
        .find(|i| i.get_object_id("_id").unwrap().to_hex() != id)
        .unwrap();
    let group_path = format!(
        "/api/inquiries/{}/messages",
        group_inquiry.get_object_id("_id").unwrap().to_hex()
    );
    let (status, error) = request(
        &state,
        &group_path,
        "POST",
        json!({"request_id":"44444444-4444-4444-8444-444444444444","text":"hi"}),
        true,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error["error"]["code"], "REPLY_UNAVAILABLE");
    assert_eq!(mock.pushes.lock().await.len(), 2);
    state.db.drop().await.unwrap();
}

#[tokio::test]
#[ignore = "requires disposable MongoDB; run cargo test -- --ignored"]
async fn mongo_line_profiles_are_cached_daily_and_shown_on_inquiries() {
    let (base, mock) = mock_line().await;
    let mut cfg = line_config(&base);
    cfg.mongodb_uri = std::env::var("TEST_MONGODB_URI").unwrap_or(cfg.mongodb_uri);
    cfg.burst_ms = 0;
    let state = AppState::connect(cfg).await.unwrap();
    crate::db::indexes::ensure(&state).await.unwrap();
    for (id, user, at) in [
        ("p-1", "Ufriend", 1),
        ("p-2", "Ufriend", 2),
        ("p-3", "Ublocked", 3),
    ] {
        events::persist(&state, &event(id, user, at)).await.unwrap();
    }
    drain(&state).await;
    // Two messages from the same user within a day fetch the profile once.
    assert_eq!(*mock.profiles.lock().await, vec!["Ufriend", "Ublocked"]);
    let users = all(&state, "line_users").await;
    let friend = users
        .iter()
        .find(|u| u.get_str("line_user_id").unwrap() == "Ufriend")
        .unwrap();
    assert_eq!(friend.get_str("display_name").unwrap(), "คุณบี ลูกค้า");
    assert_eq!(
        friend.get_str("picture_url").unwrap(),
        "https://profile.line-scdn.net/0h-test"
    );
    assert!(friend.get("statusMessage").is_none() && friend.get("status_message").is_none());
    let blocked = users
        .iter()
        .find(|u| u.get_str("line_user_id").unwrap() == "Ublocked")
        .unwrap();
    assert_eq!(blocked.get_str("profile_status").unwrap(), "UNAVAILABLE");
    let (status, list) = request(&state, "/api/inquiries", "GET", json!(null), true).await;
    assert_eq!(status, StatusCode::OK);
    let friend_inquiry = list
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["line_user_id"] == "Ufriend")
        .unwrap();
    assert_eq!(friend_inquiry["customer"]["display_name"], "คุณบี ลูกค้า");
    let blocked_inquiry = list
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["line_user_id"] == "Ublocked")
        .unwrap();
    assert!(blocked_inquiry["customer"]["display_name"].is_null());
    state.db.drop().await.unwrap();
}

#[tokio::test]
async fn image_upload_requires_a_public_https_base_before_database_access() {
    let mut cfg = config();
    cfg.public_base_url = None;
    let state = lazy_state(cfg).await;
    let path = format!("/api/inquiries/{}/media", ObjectId::new().to_hex());
    let (status, error) = request(
        &state,
        &path,
        "POST",
        json!({"original":"","preview":""}),
        true,
    )
    .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(error["error"]["code"], "IMAGE_UNAVAILABLE");
    let missing = get(
        &state,
        &format!("/api/line/outbound-media/{}/original", "a".repeat(64)),
        None,
    )
    .await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "requires disposable MongoDB; run cargo test -- --ignored"]
async fn mongo_stickers_and_images_are_sent_with_line_limits() {
    use base64::{Engine, engine::general_purpose::STANDARD};
    let (base, mock) = mock_line().await;
    let mut cfg = line_config(&base);
    cfg.mongodb_uri = std::env::var("TEST_MONGODB_URI").unwrap_or(cfg.mongodb_uri);
    cfg.burst_ms = 0;
    let state = AppState::connect(cfg).await.unwrap();
    crate::db::indexes::ensure(&state).await.unwrap();
    events::persist(&state, &event("media-start", "Umedia", 1))
        .await
        .unwrap();
    drain(&state).await;
    let id = all(&state, "inquiries").await[0]
        .get_object_id("_id")
        .unwrap()
        .to_hex();
    let send = format!("/api/inquiries/{id}/messages");
    let (status, sent) = request(&state, &send, "POST", json!({"request_id":"55555555-5555-4555-8555-555555555555","sticker":{"package_id":"11537","sticker_id":"52002734"}}), true).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(sent["message"]["payload"]["message"]["type"], "sticker");
    assert_eq!(
        mock.pushes.lock().await.last().unwrap().1["messages"][0],
        json!({"type":"sticker","packageId":"11537","stickerId":"52002734"})
    );
    let (status, error) = request(&state, &send, "POST", json!({"request_id":"66666666-6666-4666-8666-666666666666","sticker":{"package_id":"11537","sticker_id":"1"}}), true).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(error["error"]["code"], "VALIDATION_ERROR");

    // Upload: only JPEG/PNG; the stored copy is served publicly by token.
    let original = [&[0xFF, 0xD8, 0xFF, 0xE0][..], b"original-bytes"].concat();
    let preview = [&[0xFF, 0xD8, 0xFF, 0xE0][..], b"preview"].concat();
    let upload = format!("/api/inquiries/{id}/media");
    let (status, error) = request(
        &state,
        &upload,
        "POST",
        json!({"original":STANDARD.encode(b"GIF89a"),"preview":STANDARD.encode(&preview)}),
        true,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(error["error"]["code"], "VALIDATION_ERROR");
    let (status, uploaded) = request(
        &state,
        &upload,
        "POST",
        json!({"original":STANDARD.encode(&original),"preview":STANDARD.encode(&preview)}),
        true,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let original_url = uploaded["media"]["original_url"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(
        original_url.starts_with("/api/line/outbound-media/")
            && original_url.ends_with("/original")
    );
    assert!(!original_url.contains(&id));
    let served = get(&state, &original_url, None).await;
    assert_eq!(served.status(), StatusCode::OK);
    assert_eq!(served.headers()["content-type"], "image/jpeg");
    assert_eq!(served.headers()["x-content-type-options"], "nosniff");
    assert_eq!(
        to_bytes(served.into_body(), 1000).await.unwrap().as_ref(),
        &original[..]
    );
    let tampered = original_url.replace("/original", "/thumbnail");
    assert_eq!(
        get(&state, &tampered, None).await.status(),
        StatusCode::NOT_FOUND
    );

    let image_id = uploaded["media"]["_id"]["$oid"].as_str().unwrap();
    let (status, sent) = request(
        &state,
        &send,
        "POST",
        json!({"request_id":"77777777-7777-4777-8777-777777777777","image_id":image_id}),
        true,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let pushed = mock.pushes.lock().await.last().unwrap().1["messages"][0].clone();
    assert_eq!(pushed["type"], "image");
    assert_eq!(
        pushed["originalContentUrl"],
        format!("https://alex.test{original_url}")
    );
    assert_eq!(
        pushed["previewImageUrl"],
        format!(
            "https://alex.test{}",
            uploaded["media"]["preview_url"].as_str().unwrap()
        )
    );
    assert_eq!(
        sent["message"]["payload"]["message"]["preview_url"],
        uploaded["media"]["preview_url"]
    );
    // An image uploaded for another inquiry cannot be sent here.
    let (status, _) = request(
        &state,
        &format!("/api/inquiries/{}/messages", ObjectId::new().to_hex()),
        "POST",
        json!({"request_id":"88888888-8888-4888-8888-888888888888","image_id":image_id}),
        true,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (_, context) = request(
        &state,
        &format!("/api/inquiries/{id}/context"),
        "GET",
        json!(null),
        true,
    )
    .await;
    let kinds: Vec<String> = context["batches"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|b| b.as_array().unwrap())
        .map(|m| {
            m["payload"]["message"]["type"]
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect();
    assert_eq!(kinds, vec!["text", "sticker", "image"]);
    state.db.drop().await.unwrap();
}
