use super::*;
use axum::{Form, Json, Router, routing::post};
use std::collections::HashMap;

#[test]
fn team_inputs_and_tokens_are_scoped_and_do_not_allow_owner_invitations() {
    use crate::models::team::{self, InviteInput, Role};
    let input = InviteInput {
        request_id: "11111111-1111-4111-8111-111111111111".into(),
        name: "พิม".into(),
        role: Role::Staff,
        branch_ids: vec!["branch-a".into()],
        note: String::new(),
    };
    assert!(input.validate().is_ok());
    let mut owner = input;
    owner.role = Role::Owner;
    assert!(owner.validate().is_err());
    owner.role = Role::Staff;
    owner.branch_ids.push("branch-a".into());
    assert!(owner.validate().is_err());
    let id = ObjectId::new();
    let token = team::token("fixture-secret", "shop-a", "channel-a", id).unwrap();
    assert_eq!(token.len(), 43);
    assert_ne!(
        token,
        team::token("fixture-secret", "shop-b", "channel-a", id).unwrap()
    );
    assert_ne!(
        token,
        team::token("other-secret", "shop-a", "channel-a", id).unwrap()
    );
    assert!(team::token_hash("bad").is_err());
    assert!(!team::token_hash(&token).unwrap().contains(&token));
}

async fn oauth_token(Form(form): Form<HashMap<String, String>>) -> Json<Value> {
    assert_eq!(
        form.get("client_secret").map(String::as_str),
        Some("fixture-login-secret")
    );
    Json(json!({"id_token":form["code"]}))
}
async fn oauth_verify(Form(form): Form<HashMap<String, String>>) -> Json<Value> {
    assert!(form["nonce"].len() > 20);
    Json(
        json!({"aud":"fixture-login-channel","sub":form["id_token"],"name":format!("LINE {}",form["id_token"]),"picture":"https://profile.line-scdn.net/test-avatar"}),
    )
}
async fn catalog_login(Json(body): Json<Value>) -> Json<Value> {
    assert_eq!(body["username"], "fixture-user");
    Json(
        json!({"token":"fixture-catalog-token","shopInfoId":"alex-shop","userBranch":[{"branchId":"branch-a","branchName":"สาขาเอ"},{"branchId":"branch-b","branchName":"สาขาบี"}]}),
    )
}
async fn mock() -> String {
    let router = Router::new()
        .route("/oauth2/v2.1/token", post(oauth_token))
        .route("/oauth2/v2.1/verify", post(oauth_verify))
        .route("/api/user/login", post(catalog_login));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{address}")
}
async fn raw(
    state: &AppState,
    path: &str,
    method: &str,
    body: Value,
    cookie: &str,
    csrf: bool,
) -> axum::response::Response {
    let mut request = Request::builder()
        .uri(path)
        .method(method)
        .header("content-type", "application/json");
    if !cookie.is_empty() {
        request = request.header("cookie", cookie);
    }
    if csrf {
        request = request.header("x-admin-request", "1");
    }
    routes::router(state.clone())
        .unwrap()
        .oneshot(request.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap()
}
fn response_cookie(response: &axum::response::Response, name: &str) -> Option<String> {
    response
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|header| header.to_str().ok())
        .find(|cookie| cookie.starts_with(&format!("{name}=")))
        .map(|cookie| cookie.split(';').next().unwrap().to_string())
}
async fn parsed(response: axum::response::Response) -> (StatusCode, Value) {
    let status = response.status();
    let body = to_bytes(response.into_body(), 8 * 1024 * 1024)
        .await
        .unwrap();
    (status, serde_json::from_slice(&body).unwrap())
}
async fn call(
    state: &AppState,
    path: &str,
    method: &str,
    body: Value,
    cookie: &str,
) -> (StatusCode, Value) {
    parsed(raw(state, path, method, body, cookie, true).await).await
}
async fn login(state: &AppState, user: &str, invite_cookie: &str) -> Option<String> {
    let start = raw(
        state,
        "/api/admin/line/start",
        "GET",
        Value::Null,
        "",
        false,
    )
    .await;
    let oauth_cookie = response_cookie(&start, "alex_line_state").unwrap();
    let url = reqwest::Url::parse(start.headers()["location"].to_str().unwrap()).unwrap();
    let state_param = url
        .query_pairs()
        .find(|(key, _)| key == "state")
        .unwrap()
        .1
        .into_owned();
    let mut callback = reqwest::Url::parse("http://localhost/api/admin/line/callback").unwrap();
    callback
        .query_pairs_mut()
        .append_pair("code", user)
        .append_pair("state", &state_param);
    let response = raw(
        state,
        &format!("{}?{}", callback.path(), callback.query().unwrap()),
        "GET",
        Value::Null,
        &format!("{oauth_cookie}; {invite_cookie}"),
        false,
    )
    .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    response_cookie(&response, "alex_admin")
}
async fn open(state: &AppState, token: &str) -> String {
    let response = raw(
        state,
        "/api/admin/invite/open",
        "POST",
        json!({"token":token}),
        "",
        true,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let cookie = response_cookie(&response, "alex_team_invite").unwrap();
    let text = to_bytes(response.into_body(), 10000).await.unwrap();
    assert!(
        !String::from_utf8(text.to_vec())
            .unwrap()
            .contains("token_hash")
    );
    cookie
}
async fn invite(state: &AppState, owner: &str, name: &str, role: &str) -> Value {
    let body = json!({"request_id":format!("00000000-0000-4000-8000-{}",&ObjectId::new().to_hex()[12..]),"name":name,"role":role,"branch_ids":["branch-a"],"note":"กะบ่าย"});
    let (status, created) = call(
        state,
        "/api/admin/team/invitations",
        "POST",
        body.clone(),
        owner,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{created}");
    let (_, retry) = call(state, "/api/admin/team/invitations", "POST", body, owner).await;
    assert_eq!(retry, created);
    created
}

#[tokio::test]
#[ignore = "requires disposable MongoDB; run cargo test mongo_team_onboarding -- --ignored"]
async fn mongo_team_onboarding_roles_invites_and_revocation() {
    let base = mock().await;
    let mut cfg = config();
    cfg.mongodb_uri = std::env::var("TEST_MONGODB_URI").unwrap_or(cfg.mongodb_uri);
    cfg.line_api_base = base.clone();
    cfg.line_login_channel_id = Some("fixture-login-channel".into());
    cfg.line_login_channel_secret = Some("fixture-login-secret".into());
    cfg.line_login_redirect_uri = Some("http://localhost/api/admin/line/callback".into());
    cfg.admin_line_user_ids = vec!["Uowner".into()];
    let mut alex = super::product_tests::catalog_config(&base);
    alex.branch_ids.clear();
    cfg.alex = Some(alex);
    let state = AppState::connect(cfg).await.unwrap();
    crate::db::indexes::ensure(&state).await.unwrap();
    crate::db::team::bootstrap(&state).await.unwrap();
    assert!(crate::db::team::enabled(&state));
    let owner = login(&state, "Uowner", "").await.unwrap();
    let (_, session) = call(&state, "/api/admin/session", "GET", Value::Null, &owner).await;
    assert_eq!(session["membership"]["role"], "OWNER");
    assert_eq!(session["capabilities"]["manage_team"], true);
    let owner_id = session["membership"]["id"].as_str().unwrap().to_string();
    assert_eq!(
        call(
            &state,
            "/api/admin/login",
            "POST",
            json!({"username":"admin","password":"test-only-password"}),
            ""
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert!(login(&state, "Uunknown", "").await.is_none());
    assert_eq!(call(&state,"/api/admin/team/invitations","POST",json!({"request_id":"11111111-1111-4111-8111-111111111111","name":"bad","role":"OWNER","branch_ids":["branch-a"]}),&owner).await.0,StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(call(&state,"/api/admin/team/invitations","POST",json!({"request_id":"22222222-2222-4222-8222-222222222222","name":"bad branch","role":"STAFF","branch_ids":["outside"]}),&owner).await.0,StatusCode::FORBIDDEN);
    let created = invite(&state, &owner, "พิม", "STAFF").await;
    let token = created["token"].as_str().unwrap();
    let id = created["member"]["id"].as_str().unwrap();
    let invite_cookie = open(&state, token).await;
    let staff = login(&state, "Ustaff", &invite_cookie).await.unwrap();
    assert_eq!(
        call(
            &state,
            "/api/admin/invite/accept",
            "POST",
            json!({"invitation_id":ObjectId::new().to_hex(),"expected_version":1}),
            &format!("{staff}; {invite_cookie}")
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        call(&state, "/api/inquiries", "GET", Value::Null, &staff)
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        raw(
            &state,
            "/api/admin/invite/accept",
            "POST",
            json!({"invitation_id":id,"expected_version":1}),
            &format!("{staff}; {invite_cookie}"),
            false
        )
        .await
        .status(),
        StatusCode::UNAUTHORIZED
    );
    let (status, accepted) = call(
        &state,
        "/api/admin/invite/accept",
        "POST",
        json!({"invitation_id":id,"expected_version":1}),
        &format!("{staff}; {invite_cookie}"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{accepted}");
    assert_eq!(accepted["membership"]["status"], "PENDING");
    let (_, repeat) = call(
        &state,
        "/api/admin/invite/accept",
        "POST",
        json!({"invitation_id":id,"expected_version":1}),
        &format!("{staff}; {invite_cookie}"),
    )
    .await;
    assert_eq!(repeat, accepted);
    for path in [
        "/api/inquiries",
        "/api/line/users/Ucustomer/notes",
        "/api/media/111111111111111111111111",
        "/api/admin/team",
        "/api/inquiries/111111111111111111111111/drafts",
    ] {
        assert_eq!(
            call(&state, path, "GET", Value::Null, &staff).await.0,
            StatusCode::FORBIDDEN,
            "{path}"
        );
    }
    let (_, approved) = call(
        &state,
        &format!("/api/admin/team/{id}/approve"),
        "POST",
        json!({"expected_version":2}),
        &owner,
    )
    .await;
    assert_eq!(approved["member"]["status"], "ACTIVE");
    assert_eq!(
        call(
            &state,
            &format!("/api/admin/team/{id}/approve"),
            "POST",
            json!({"expected_version":2}),
            &owner
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let (status, staff_session) =
        call(&state, "/api/admin/session", "GET", Value::Null, &staff).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(staff_session["username"], "พิม");
    assert_eq!(staff_session["capabilities"]["submit_order"], false);
    let (_, settings) = call(&state, "/api/admin/alex", "GET", Value::Null, &staff).await;
    assert_eq!(settings["branches"].as_array().unwrap().len(), 1);
    assert_eq!(settings["branches"][0]["id"], "branch-a");
    for path in ["/api/admin/team", &format!("/api/admin/team/{id}/history")] {
        assert_eq!(
            call(&state, path, "GET", Value::Null, &staff).await.0,
            StatusCode::FORBIDDEN
        );
    }
    for suffix in ["checkout/submit", "checkout/resolve", "order"] {
        assert_eq!(
            call(
                &state,
                &format!("/api/inquiries/111111111111111111111111/{suffix}"),
                "POST",
                json!({}),
                &staff
            )
            .await
            .0,
            StatusCode::FORBIDDEN
        );
    }
    let mut customer = state.scope();
    customer.extend(doc! {"_id":ObjectId::new(),"line_user_id":"Ucustomer","created_at":DateTime::now(),"updated_at":DateTime::now()});
    state
        .collection("line_users")
        .insert_one(customer)
        .await
        .unwrap();
    let note =
        json!({"request_id":"33333333-3333-4333-8333-333333333333","text":"โน้ตของพิม","images":[]});
    let (status, note) = call(
        &state,
        "/api/line/users/Ucustomer/notes",
        "POST",
        note,
        &staff,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{note}");
    assert_eq!(note["note"]["created_by"], "พิม");
    assert_eq!(note["note"]["created_by_id"], "line:Ustaff");
    let mut update = json!({"expected_version":3,"name":"พิมใหม่","role":"MANAGER","branch_ids":["branch-a"],"status":"ACTIVE","note":"กะเช้า"});
    let (status, changed) = call(
        &state,
        &format!("/api/admin/team/{id}/update"),
        "POST",
        update.clone(),
        &owner,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{changed}");
    assert_eq!(
        call(
            &state,
            "/api/inquiries/111111111111111111111111/checkout/submit",
            "POST",
            json!({}),
            &staff
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        call(&state, "/api/admin/team", "GET", Value::Null, &staff)
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    let (_, changed_session) = call(&state, "/api/admin/session", "GET", Value::Null, &staff).await;
    assert_eq!(changed_session["username"], "พิมใหม่");
    assert_eq!(changed_session["capabilities"]["submit_order"], true);
    let bill_inquiry = ObjectId::new();
    let bill_id = ObjectId::new();
    let mut bill = state.scope();
    bill.extend(doc! {"_id":bill_id,"inquiry_id":bill_inquiry,"group":{"branch_id":"branch-b"},"status":"SUBMIT_UNKNOWN","active":false,"created_at":DateTime::now(),"updated_at":DateTime::now()});
    state
        .collection("alex_submissions")
        .insert_one(bill)
        .await
        .unwrap();
    let resolve_url = format!("/api/inquiries/{bill_inquiry}/checkout/resolve");
    let resolution = json!({"submission_id":bill_id.to_hex(),"outcome":"created","order_id":"fixture-order","verified_in_alex":true});
    assert_eq!(
        call(&state, &resolve_url, "POST", resolution.clone(), &staff)
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &state,
            &format!("/api/inquiries/{bill_inquiry}/checkout/submit"),
            "POST",
            json!({"review_id":bill_id.to_hex(),"confirmed":true}),
            &staff
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        state
            .collection("alex_submissions")
            .find_one(doc! {"_id":bill_id})
            .await
            .unwrap()
            .unwrap()
            .get_str("status")
            .unwrap(),
        "SUBMIT_UNKNOWN"
    );
    let (status, resolved) = call(&state, &resolve_url, "POST", resolution, &owner).await;
    assert_eq!(status, StatusCode::OK, "{resolved}");
    assert_eq!(resolved["status"], "CONFIRMED");
    let other_inquiry = ObjectId::new();
    let mut cart = state.scope();
    cart.extend(doc! {"_id":ObjectId::new(),"inquiry_id":other_inquiry,"branch_id":"branch-b","quantity":1_i64,"revision":1_i64,"created_at":DateTime::now()});
    state
        .collection("cart_items")
        .insert_one(cart)
        .await
        .unwrap();
    assert_eq!(
        call(
            &state,
            &format!("/api/inquiries/{other_inquiry}/cart"),
            "GET",
            Value::Null,
            &staff
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let access = crate::db::team::access(&state, "Ustaff", "pim")
        .await
        .unwrap();
    let job =
        doc! {"requested_by_member":access.member_id,"requested_by_member_version":access.version};
    update["expected_version"] = json!(4);
    update["status"] = json!("SUSPENDED");
    let (status, suspended) = call(
        &state,
        &format!("/api/admin/team/{id}/update"),
        "POST",
        update.clone(),
        &owner,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{suspended}");
    assert_eq!(
        call(&state, "/api/inquiries", "GET", Value::Null, &staff)
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert!(crate::db::team::job_state(&state, &job).await.is_err());
    crate::db::team::bootstrap(&state).await.unwrap();
    assert_eq!(
        call(&state, "/api/inquiries", "GET", Value::Null, &staff)
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    update["expected_version"] = json!(5);
    update["status"] = json!("ACTIVE");
    assert_eq!(
        call(
            &state,
            &format!("/api/admin/team/{id}/update"),
            "POST",
            update.clone(),
            &owner
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(&state, "/api/inquiries", "GET", Value::Null, &staff)
            .await
            .0,
        StatusCode::OK
    );
    update["expected_version"] = json!(1);
    assert_eq!(
        call(
            &state,
            &format!("/api/admin/team/{owner_id}/update"),
            "POST",
            update,
            &owner
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let (_, history) = call(
        &state,
        &format!("/api/admin/team/{id}/history"),
        "GET",
        Value::Null,
        &owner,
    )
    .await;
    assert_eq!(history["events"].as_array().unwrap().len(), 6);
    assert_eq!(history["events"][0]["actor_id"], "line:Uowner");
    assert_eq!(history["events"][4]["action"], "ACCEPTED");
    let foreign = ObjectId::new();
    state.collection("team_memberships").insert_one(doc! {"_id":foreign,"shop_id":"foreign-shop","channel_id":"foreign-channel","status":"ACTIVE"}).await.unwrap();
    assert_eq!(
        call(
            &state,
            &format!("/api/admin/team/{foreign}"),
            "GET",
            Value::Null,
            &owner
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );

    // Two identities can authenticate with a forwarded link, but only one can consume it.
    let race = invite(&state, &owner, "รับพร้อมกัน", "MANAGER").await;
    let race_cookie = open(&state, race["token"].as_str().unwrap()).await;
    let user_a = login(&state, "UraceA", &race_cookie).await.unwrap();
    let user_b = login(&state, "UraceB", &race_cookie).await.unwrap();
    let cookie_a = format!("{user_a}; {race_cookie}");
    let cookie_b = format!("{user_b}; {race_cookie}");
    let (a, b) = tokio::join!(
        call(
            &state,
            "/api/admin/invite/accept",
            "POST",
            json!({"invitation_id":race["member"]["id"],"expected_version":1}),
            &cookie_a
        ),
        call(
            &state,
            "/api/admin/invite/accept",
            "POST",
            json!({"invitation_id":race["member"]["id"],"expected_version":1}),
            &cookie_b
        )
    );
    assert!(
        (a.0 == StatusCode::OK && b.0 == StatusCode::CONFLICT)
            || (b.0 == StatusCode::OK && a.0 == StatusCode::CONFLICT)
    );
    let (_, race_member) = call(
        &state,
        &format!("/api/admin/team/{}", race["member"]["id"].as_str().unwrap()),
        "GET",
        Value::Null,
        &owner,
    )
    .await;
    assert_eq!(race_member["member"]["status"], "PENDING");
    assert_eq!(
        call(&state, "/api/inquiries", "GET", Value::Null, &user_a)
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(&state, "/api/inquiries", "GET", Value::Null, &user_b)
            .await
            .0,
        StatusCode::FORBIDDEN
    );

    // A single verified identity cannot claim two invitations concurrently.
    let first = invite(&state, &owner, "duplicate-one", "STAFF").await;
    let second = invite(&state, &owner, "duplicate-two", "STAFF").await;
    let first_context = open(&state, first["token"].as_str().unwrap()).await;
    let second_context = open(&state, second["token"].as_str().unwrap()).await;
    let identity = login(&state, "Uduplicate", &first_context).await.unwrap();
    let first_cookie = format!("{identity}; {first_context}");
    let second_cookie = format!("{identity}; {second_context}");
    let (first_result, second_result) = tokio::join!(
        call(
            &state,
            "/api/admin/invite/accept",
            "POST",
            json!({"invitation_id":first["member"]["id"],"expected_version":1}),
            &first_cookie
        ),
        call(
            &state,
            "/api/admin/invite/accept",
            "POST",
            json!({"invitation_id":second["member"]["id"],"expected_version":1}),
            &second_cookie
        )
    );
    assert!(
        (first_result.0 == StatusCode::OK && second_result.0 == StatusCode::CONFLICT)
            || (second_result.0 == StatusCode::OK && first_result.0 == StatusCode::CONFLICT),
        "first={first_result:?}, second={second_result:?}"
    );
    let mut duplicate_filter = state.scope();
    duplicate_filter.extend(doc! {"line_user_id":"Uduplicate","live_member":true});
    assert_eq!(
        state
            .collection("team_memberships")
            .count_documents(duplicate_filter)
            .await
            .unwrap(),
        1
    );
    let expired = invite(&state, &owner, "หมดอายุ", "STAFF").await;
    let expired_id = ObjectId::parse_str(expired["member"]["id"].as_str().unwrap()).unwrap();
    state
        .collection("team_memberships")
        .update_one(
            doc! {"_id":expired_id},
            doc! {"$set":{"expires_at":DateTime::from_millis(0)}},
        )
        .await
        .unwrap();
    assert_eq!(
        call(
            &state,
            &format!("/api/admin/team/{expired_id}/link"),
            "POST",
            json!({}),
            &owner
        )
        .await
        .1["error"]["code"],
        "INVITE_EXPIRED"
    );
    let expired_cookie = open(&state, expired["token"].as_str().unwrap()).await;
    assert!(login(&state, "Uexpired", &expired_cookie).await.is_none());
    let cancelled = invite(&state, &owner, "ยกเลิก", "STAFF").await;
    let cancelled_id = cancelled["member"]["id"].as_str().unwrap();
    assert_eq!(
        call(
            &state,
            &format!("/api/admin/team/{cancelled_id}/cancel"),
            "POST",
            json!({"expected_version":1}),
            &owner
        )
        .await
        .0,
        StatusCode::OK
    );
    let cancelled_cookie = open(&state, cancelled["token"].as_str().unwrap()).await;
    assert!(
        login(&state, "Ucancelled", &cancelled_cookie)
            .await
            .is_none()
    );
    let mut settings = (*state.config).clone();
    settings.admin_line_user_ids.push("UlateWhitelist".into());
    let changed_state = AppState {
        config: Arc::new(settings),
        ..state.clone()
    };
    crate::db::team::bootstrap(&changed_state).await.unwrap();
    assert!(
        crate::db::team::membership(&state, "UlateWhitelist")
            .await
            .unwrap()
            .is_none()
    );
    let (_, options) = call(&state, "/api/admin/login-options", "GET", Value::Null, "").await;
    assert_eq!(options["password"], false);
    state.db.drop().await.unwrap();
}
