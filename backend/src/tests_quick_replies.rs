use super::*;
use crate::{
    db::{outbound, quick_replies},
    models::{
        customer::Actor,
        outbound::SendInput,
        quick_replies::{CreateInput, EditInput},
    },
};
use axum::{Json, Router, extract::State, routing::post};
use std::sync::atomic::{AtomicUsize, Ordering};

fn text_input(request: &str, text: &str) -> Value {
    json!({"request_id":request,"title":"สวัสดี","kind":"text","text":text})
}
fn card_input(request: &str) -> Value {
    let image = STANDARD.encode(b"\x89PNG\r\n\x1a\nquick-reply-fixture");
    json!({"request_id":request,"title":"ตารางไซซ์","kind":"image","image":{"original":image,"preview":image}})
}
#[test]
fn quick_replies_validate_disjoint_text_and_image_inputs_and_versions() {
    let id = "11111111-1111-4111-8111-111111111111";
    let (_, content) = serde_json::from_value::<CreateInput>(text_input(id, " สวัสดีค่ะ\nยินดีต้อนรับ "))
        .unwrap()
        .validate()
        .unwrap();
    assert_eq!(content.text, "สวัสดีค่ะ\nยินดีต้อนรับ");
    assert!(
        serde_json::from_value::<CreateInput>(card_input(id))
            .unwrap()
            .validate()
            .unwrap()
            .1
            .image
            .is_some()
    );
    for value in [
        text_input("bad", "hi"),
        text_input(id, ""),
        text_input(id, &"ก".repeat(5001)),
        text_input(id, "a\0b"),
        json!({"request_id":id,"title":"/bad","kind":"text","text":"hi"}),
        json!({"request_id":id,"title":"a\nb","kind":"text","text":"hi"}),
        json!({"request_id":id,"title":"x","kind":"image"}),
        json!({"request_id":id,"title":"x","kind":"flex","text":"hi"}),
        json!({"request_id":id,"title":"x","kind":"image","image":{"original":STANDARD.encode(b"<svg onload='alert(1)'>"),"preview":"bad"}}),
    ] {
        assert!(
            serde_json::from_value::<CreateInput>(value)
                .unwrap()
                .validate()
                .is_err()
        );
    }
    let mut mixed = card_input(id);
    mixed["text"] = json!("not allowed");
    assert!(
        serde_json::from_value::<CreateInput>(mixed)
            .unwrap()
            .validate()
            .is_err()
    );
    let mut spoof = text_input(id, "hi");
    spoof["updated_by"] = json!("spoof");
    assert!(serde_json::from_value::<CreateInput>(spoof).is_err());
    assert!(
        serde_json::from_value::<EditInput>(
            json!({"expected_version":0,"kind":"text","title":"title","text":"hi"})
        )
        .unwrap()
        .validate()
        .is_err()
    );
    assert!(
        serde_json::from_value::<EditInput>(
            json!({"expected_version":1,"kind":"image","title":"title"})
        )
        .unwrap()
        .validate()
        .is_ok()
    );
}
#[tokio::test]
async fn quick_reply_routes_reject_unauthenticated_and_invalid_payloads_before_db() {
    let cfg = config();
    let client = mongodb::Client::with_uri_str(&cfg.mongodb_uri)
        .await
        .unwrap();
    let state = AppState {
        db: client.database(&cfg.mongodb_db),
        config: Arc::new(cfg),
        http: reqwest::Client::new(),
        team_enabled: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        access: None,
        login_attempts: Arc::new(tokio::sync::Mutex::new((0, std::time::Instant::now()))),
    };
    let path = "/api/admin/quick-replies";
    let item = format!("{path}/000000000000000000000001");
    for (url, method) in [
        (path.to_string(), "GET"),
        (path.to_string(), "POST"),
        (item.clone(), "POST"),
        (format!("{item}/delete"), "POST"),
        (format!("{item}/image/preview"), "GET"),
    ] {
        assert_eq!(
            request(&state, &url, method, json!({}), false).await.0,
            StatusCode::UNAUTHORIZED
        );
    }
    for (url, body) in [
        (path.to_string(), text_input("bad", "hi")),
        (
            item.clone(),
            json!({"expected_version":-1,"kind":"text","title":"title","text":"hi"}),
        ),
        (format!("{item}/delete"), json!({"expected_version":0})),
        (
            "/api/inquiries/000000000000000000000001/quick-replies/000000000000000000000002/media"
                .to_string(),
            json!({"expected_version":0}),
        ),
    ] {
        assert_eq!(
            request(&state, &url, "POST", body, true).await.0,
            StatusCode::BAD_REQUEST
        );
    }
}
async fn push(State(count): State<Arc<AtomicUsize>>, Json(body): Json<Value>) -> Json<Value> {
    assert_eq!(body["messages"][0]["type"], "image");
    count.fetch_add(1, Ordering::SeqCst);
    Json(json!({"sentMessages":[{"id":"fixture-image"}]}))
}
#[tokio::test]
#[ignore = "requires disposable MongoDB"]
async fn mongo_quick_replies_crud_scope_images_cas_and_outbound_retry() {
    let count = Arc::new(AtomicUsize::new(0));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mut cfg = config();
    cfg.line_api_base = format!("http://{}", listener.local_addr().unwrap());
    let pushes = count.clone();
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            Router::new()
                .route("/v2/bot/message/push", post(push))
                .with_state(pushes),
        )
        .await
        .unwrap();
    });
    let state = AppState::connect(cfg).await.unwrap();
    crate::db::indexes::ensure(&state).await.unwrap();
    let base = "/api/admin/quick-replies";
    let req = "11111111-1111-4111-8111-111111111111";
    let (status, created) = request(
        &state,
        base,
        "POST",
        text_input(req, "สวัสดีค่ะ ยินดีต้อนรับสู่ Arisa Rental ค่ะ"),
        true,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["item"]["created_by"], "internal-api");
    let (_, same) = request(
        &state,
        base,
        "POST",
        text_input(req, "สวัสดีค่ะ ยินดีต้อนรับสู่ Arisa Rental ค่ะ"),
        true,
    )
    .await;
    assert_eq!(same["item"]["_id"], created["item"]["_id"]);
    assert_eq!(
        request(&state, base, "POST", text_input(req, "different"), true)
            .await
            .0,
        StatusCode::CONFLICT
    );
    let reply = ObjectId::parse_str(created["item"]["_id"]["$oid"].as_str().unwrap()).unwrap();
    let actor = Actor {
        id: "line:staff".into(),
        name: "Staff B".into(),
    };
    let edit = || {
        serde_json::from_value::<EditInput>(
            json!({"expected_version":1,"title":"ต้อนรับ","kind":"text","text":"edited"}),
        )
        .unwrap()
        .validate()
        .unwrap()
        .1
    };
    let (a, b) = tokio::join!(
        quick_replies::edit(&state, reply, 1, edit(), &actor),
        quick_replies::edit(&state, reply, 1, edit(), &actor)
    );
    assert_eq!(usize::from(a.is_ok()) + usize::from(b.is_ok()), 1);
    let (_, listed) = request(&state, base, "GET", json!(null), true).await;
    assert_eq!(listed["items"][0]["version"], 2);
    assert_eq!(listed["items"][0]["updated_by"], "Staff B");
    let card_req = "22222222-2222-4222-8222-222222222222";
    let (_, created) = request(&state, base, "POST", card_input(card_req), true).await;
    let card = ObjectId::parse_str(created["item"]["_id"]["$oid"].as_str().unwrap()).unwrap();
    assert!(created["item"].get("original_file_id").is_none());
    assert!(
        created["item"]["preview_url"]
            .as_str()
            .unwrap()
            .starts_with(base)
    );
    let bytes = quick_replies::image(&state, card, "preview").await.unwrap();
    assert_eq!(bytes.mime, "image/png");
    let grid = state
        .db
        .gridfs_bucket(None)
        .find_one(doc! {"metadata.quick_reply_id":card})
        .await
        .unwrap()
        .unwrap();
    assert!(!grid.metadata.unwrap().contains_key("expires_at"));
    let mut other = state.clone();
    let mut config = (*state.config).clone();
    config.shop_id = "other-shop".into();
    other.config = Arc::new(config);
    assert!(quick_replies::image(&other, card, "preview").await.is_err());
    assert!(
        quick_replies::list(&other, None)
            .await
            .unwrap()
            .0
            .is_empty()
    );
    assert!(
        quick_replies::delete(&other, card, 1, &actor)
            .await
            .is_err()
    );
    let mut wrong_channel = state.clone();
    let mut config = (*state.config).clone();
    config.channel_id = "other-channel".into();
    wrong_channel.config = Arc::new(config);
    assert!(
        quick_replies::image(&wrong_channel, card, "preview")
            .await
            .is_err()
    );
    let mut replacement = card_input(card_req);
    replacement.as_object_mut().unwrap().remove("request_id");
    replacement["expected_version"] = json!(1);
    let (version, content) = serde_json::from_value::<EditInput>(replacement)
        .unwrap()
        .validate()
        .unwrap();
    let edited_card = quick_replies::edit(&state, card, version, content, &actor)
        .await
        .unwrap();
    assert_eq!(edited_card.get_i64("version").unwrap(), 2);
    assert_eq!(
        state
            .db
            .collection::<Document>("fs.files")
            .count_documents(doc! {"metadata.quick_reply_id":card})
            .await
            .unwrap(),
        2
    );
    let inquiry = ObjectId::new();
    let conversation = ObjectId::new();
    let now = DateTime::now();
    let mut row = state.scope();
    row.extend(doc! {"_id":conversation,"conversation_key":"[\"user\",\"Uquick\"]","created_at":now,"updated_at":now});
    state
        .collection("conversations")
        .insert_one(row)
        .await
        .unwrap();
    let mut row = state.scope();
    row.extend(doc! {"_id":inquiry,"line_user_id":"Uquick","conversation_id":conversation,"status":"RECEIVED","created_at":now,"updated_at":now});
    state.collection("inquiries").insert_one(row).await.unwrap();
    assert!(
        quick_replies::prepare_image(&state, inquiry, card, 1, &actor)
            .await
            .is_err()
    );
    let prepared = quick_replies::prepare_image(&state, inquiry, card, 2, &actor)
        .await
        .unwrap();
    assert_eq!(count.load(Ordering::SeqCst), 0);
    let media = prepared.get_object_id("_id").unwrap();
    quick_replies::delete(&state, card, 2, &actor)
        .await
        .unwrap();
    assert!(quick_replies::image(&state, card, "preview").await.is_err());
    assert!(
        state
            .db
            .gridfs_bucket(None)
            .find_one(doc! {"metadata.quick_reply_id":card})
            .await
            .unwrap()
            .is_none()
    );
    // A prepared reply is a separate snapshot and survives source-card deletion.
    let send = || SendInput {
        request_id: "33333333-3333-4333-8333-333333333333".into(),
        image_id: Some(media.to_hex()),
        ..Default::default()
    };
    let first = outbound::send(&state, inquiry, send(), &actor)
        .await
        .unwrap();
    let again = outbound::send(&state, inquiry, send(), &actor)
        .await
        .unwrap();
    assert_eq!(
        first.get_object_id("_id").unwrap(),
        again.get_object_id("_id").unwrap()
    );
    let stored = state
        .collection("outbound_messages")
        .count_documents(state.scope())
        .await
        .unwrap();
    assert_eq!(stored, 1);
    assert_eq!(count.load(Ordering::SeqCst), 1);
    assert_eq!(
        request(
            &state,
            &format!("{base}/{reply}/delete"),
            "POST",
            json!({"expected_version":1}),
            true
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let (_, deleted) = request(
        &state,
        &format!("{base}/{reply}/delete"),
        "POST",
        json!({"expected_version":2}),
        true,
    )
    .await;
    assert_eq!(deleted["deleted"], true);
    let (_, listed) = request(&state, base, "GET", json!(null), true).await;
    assert_eq!(listed["items"], json!([]));
    let rows: Vec<_> = (0..81).map(|index| {
        let mut row = state.scope(); row.extend(doc! {"_id":ObjectId::new(),"kind":"text","title":format!("template-{index}"),"text":"reply","version":1_i64,"created_by_id":"fixture","request_id":format!("page-{index}"),"created_at":now,"updated_at":now}); row
    }).collect();
    state
        .collection("quick_replies")
        .insert_many(rows)
        .await
        .unwrap();
    let (first_page, next) = quick_replies::list(&state, None).await.unwrap();
    assert_eq!(first_page.len(), 80);
    let (last_page, next) =
        quick_replies::list(&state, Some(ObjectId::parse_str(next.unwrap()).unwrap()))
            .await
            .unwrap();
    assert_eq!(last_page.len(), 1);
    assert!(next.is_none());
    let ids: std::collections::HashSet<_> = first_page
        .iter()
        .chain(&last_page)
        .map(|item| item.get_object_id("_id").unwrap())
        .collect();
    assert_eq!(ids.len(), 81);
    state.db.drop().await.unwrap();
    server.abort();
}
