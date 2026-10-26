use super::*;
use crate::{
    db::{drafts, products},
    draft_worker,
    models::draft::{DraftFields, PrepareInput, SummaryInput},
};
use axum::{Json, extract::State, response::IntoResponse};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Default)]
struct Flow {
    mode: Arc<AtomicUsize>,
    calls: Arc<tokio::sync::Mutex<Vec<Value>>>,
    started: Arc<tokio::sync::Notify>,
    resume: Arc<tokio::sync::Notify>,
}

async fn login(State(flow): State<Flow>, Json(body): Json<Value>) -> axum::response::Response {
    assert_eq!(body["username"], "fixture-user");
    assert_eq!(body["password"], "fixture-password");
    flow.calls.lock().await.push(json!({"operation":"login"}));
    if flow.mode.load(Ordering::SeqCst) == 3 {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    Json(json!({"shopInfoId":"alex-shop","token":"fixture-alex-token","userBranch":[{"branchId":"branch-a","branchName":"Branch A"}]})).into_response()
}

async fn catalog(
    State(flow): State<Flow>,
    headers: axum::http::HeaderMap,
    Json(body): Json<Value>,
) -> Json<Value> {
    assert_eq!(headers["authorization"], "Bearer fixture-alex-token");
    let mut calls = flow.calls.lock().await;
    assert_eq!(calls[0]["operation"], "login");
    calls.push(json!({"operation":"catalog","body":body}));
    drop(calls);
    let mode = flow.mode.load(Ordering::SeqCst);
    if mode == 7 {
        flow.started.notify_one();
        flow.resume.notified().await;
    }
    let candidate = |id: &str| json!({"productItemId":id,"code":"DR-NINA","name":"Nina dress","color":if mode==5{"blue"}else{"dark brown"},"size":if mode==5{"L"}else{"M"},"rentalPrice":1000,"bail":2000,"mainImage":"/photo.png"});
    let items = match mode {
        // Searching inside the resolved category: no keyword, only the category code and options.
        8 => {
            assert_eq!(body["category"], "HT");
            assert_eq!(body["keyword"], "");
            let mut item = candidate("haatiya-1");
            item["code"] = json!("0288HTBP");
            item["name"] = json!("Haatiya - Blush Pink");
            item["color"] = json!("Blush Pink");
            vec![item]
        }
        9 => {
            assert_eq!(body["category"], "");
            assert_eq!(body["keyword"], "Unknown line");
            vec![]
        }
        1 => vec![candidate("nina-1"), candidate("nina-2")],
        2 => vec![],
        6 if body["pageNumber"] == 1 => (0..15)
            .map(|index| {
                let mut item = candidate(&format!("other-{index}"));
                item["name"] = json!("Other dress");
                item
            })
            .collect(),
        _ => vec![candidate("nina-1")],
    };
    Json(json!({"itemNumber":if mode==6{16}else{items.len()},"itemList":items}))
}

async fn categories(headers: axum::http::HeaderMap) -> Json<Value> {
    assert_eq!(headers["authorization"], "Bearer fixture-alex-token");
    Json(
        json!([{"code":"HT","description":"Haatiya","state":true},{"code":"OLD","description":"Unknown line","state":false}]),
    )
}

async fn extract(State(flow): State<Flow>, Json(body): Json<Value>) -> Json<Value> {
    let data: Value =
        serde_json::from_str(body["messages"][1]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(data["input_scope"], "admin_inputs_only");
    assert!(data.get("events").is_none());
    assert!(data.get("previous_draft").is_none());
    assert!(!data.to_string().contains("CHAT-ONLY"));
    assert!(!data.to_string().contains("OLD-DRAFT"));
    let mut fields: DraftFields = serde_json::from_value(json!({"items":[{"product_name":"nina","color":"dark brown","size":"M","quantity":1}],"amounts":{"rental_fee_satang":55000,"shipping_fee_satang":4000}})).unwrap();
    if flow.mode.load(Ordering::SeqCst) == 4 {
        fields.amounts.rental_fee_satang = None;
    }
    if [8, 9].contains(&flow.mode.load(Ordering::SeqCst)) {
        fields.items[0].product_name = None;
        fields.items[0].color = Some("Blush Pink".into());
        fields.items[0].category = Some(if flow.mode.load(Ordering::SeqCst) == 8 {
            "haatiya".into()
        } else {
            "Unknown line".into()
        });
    }
    let image = data["selected_image_event_refs"]
        .as_array()
        .unwrap()
        .first();
    let evidence: Vec<Value> = fields.values().keys().map(|field| json!({
        "field":field,"source_type":if image.is_some(){"admin_attachment"}else{"admin_summary"},
        "source_ref":image.unwrap_or(&data["summary_ref"]),
        "quote":if image.is_some(){json!("visible booking summary")}else{data["admin_summary"].clone()}
    })).collect();
    let content =
        json!({"fields":fields,"evidence":evidence,"conflicts":[],"questions":[]}).to_string();
    Json(json!({"choices":[{"finish_reason":"stop","message":{"content":content}}]}))
}

async fn fixture(mode: usize) -> (AppState, Flow, tokio::task::JoinHandle<()>, ObjectId) {
    let flow = Flow::default();
    flow.mode.store(mode, Ordering::SeqCst);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = axum::Router::new()
        .route("/api/user/login", axum::routing::post(login))
        .route("/api/v2/GetProductItems", axum::routing::post(catalog))
        .route("/api/category", axum::routing::get(categories))
        .route(
            "/v1/toolsets",
            axum::routing::get(|| async {
                Json(json!({"object":"list","platform":"api_server","data":[]}))
            }),
        )
        .route("/v1/chat/completions", axum::routing::post(extract))
        .with_state(flow.clone());
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let mut cfg = config();
    cfg.mongodb_uri = std::env::var("TEST_MONGODB_URI").unwrap_or(cfg.mongodb_uri);
    cfg.burst_ms = 0;
    cfg.alex = Some(super::product_tests::catalog_config(&format!(
        "http://{address}"
    )));
    cfg.hermes_url = Some(format!("http://{address}/v1/chat/completions"));
    cfg.hermes_api_key = Some("fixture-hermes-key".into());
    let state = AppState::connect(cfg).await.unwrap();
    crate::db::indexes::ensure(&state).await.unwrap();
    let mut event = event("catalog-flow", "catalog-customer", 1);
    event.message.as_mut().unwrap()["text"] =
        json!("CHAT-ONLY Wrong product BLUE XL previous order 9999 baht");
    events::persist(&state, &event).await.unwrap();
    let incoming = events::claim(&state).await.unwrap().unwrap();
    worker::process(&state, &incoming).await.unwrap();
    let id = all(&state, "inquiries").await[0]
        .get_object_id("_id")
        .unwrap();
    let old: DraftFields = serde_json::from_value(
        json!({"customer_name":"OLD-DRAFT","items":[{"product_name":"OLD-DRAFT"}]}),
    )
    .unwrap();
    super::product_tests::manual(&state, id, 0, old).await;
    assert!(flow.calls.lock().await.is_empty());
    (state, flow, server, id)
}

async fn enqueue(state: &AppState, id: ObjectId, image: bool) -> Document {
    let mut images = vec![];
    if image {
        let png = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+yZf8AAAAASUVORK5CYII=";
        let (_, uploaded) = request(
            state,
            &format!("/api/inquiries/{id}/draft-images"),
            "POST",
            json!({"name":"summary.png","image_base64":png}),
            true,
        )
        .await;
        images.push(uploaded["attachment"]["id"].as_str().unwrap().to_string());
    }
    let snapshot = drafts::snapshot(state, &drafts::inquiry(state, id).await.unwrap())
        .await
        .unwrap();
    drafts::enqueue(
        state,
        id,
        PrepareInput {
            request_id: "auto-catalog".into(),
            expected_version: 1,
            summary: if image {
                String::new()
            } else {
                "nina dark brown M, quantity 1, rental 550 + Flash 40".into()
            },
            image_event_ids: images,
            fields: None,
            confirmed_fields: vec![],
            context_hash: Some(snapshot.hash),
        },
        false,
    )
    .await
    .unwrap();
    drafts::claim(state).await.unwrap().unwrap()
}

#[tokio::test]
#[ignore = "requires disposable MongoDB; run cargo test -- --ignored"]
async fn admin_inputs_search_logged_in_catalog_fill_real_fields_and_keep_ambiguous_choices() {
    for (mode, image) in [
        (0, false),
        (0, true),
        (1, false),
        (2, false),
        (3, false),
        (4, false),
        (5, false),
        (6, false),
    ] {
        let (state, flow, server, id) = fixture(mode).await;
        let job = enqueue(&state, id, image).await;
        draft_worker::process(&state, &job).await.unwrap();
        let inquiry = drafts::inquiry(&state, id).await.unwrap();
        let saved = drafts::safe_drafts(&state, &inquiry).await.unwrap();
        let latest = &saved[0];
        let item = &latest["result"]["fields"]["items"][0];
        assert!(latest["result"]["fields"]["customer_name"].is_null());
        // Quoted fees stay order-level; catalog prices live in the cart, not the draft.
        let amounts = &latest["result"]["fields"]["amounts"];
        if mode == 4 {
            assert!(amounts["rental_fee_satang"].is_null());
        } else {
            assert_eq!(amounts["rental_fee_satang"], 55000);
        }
        assert_eq!(amounts["shipping_fee_satang"], 4000);
        assert!(item.get("amounts").is_none());
        let lookup = &latest["catalog_lookup"][0];
        let matched = mode == 0 || mode == 4 || mode == 6;
        if matched {
            assert_eq!(lookup["status"], "MATCHED");
            assert_eq!(item["product_code"], "DR-NINA");
            assert_eq!(item["product_name"], "Nina dress");
            assert_eq!(lookup["selected_product_item_id"], "nina-1");
            assert!(
                latest["result"]["evidence"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|e| e["field"] == "items.0.product_code"
                        && e["source_type"] == "alex_catalog")
            );
        } else {
            assert!(item["product_code"].is_null());
            assert_eq!(
                lookup["status"],
                match mode {
                    1 | 5 => "NEEDS_SELECTION",
                    2 => "NOT_FOUND",
                    3 => "FAILED",
                    _ => unreachable!(),
                }
            );
            assert!(!latest["result"]["questions"].as_array().unwrap().is_empty());
        }
        let result = products::list(&state, id).await.unwrap();
        assert!(all(&state, "orders").await.is_empty());
        let calls = flow.calls.lock().await;
        assert_eq!(calls[0]["operation"], "login");
        if mode != 3 {
            assert_eq!(calls[1]["body"]["keyword"], "nina");
            assert_eq!(calls[1]["body"]["branchId"], "branch-a");
            assert_eq!(calls[1]["body"]["color"], "dark brown|");
            assert_eq!(calls[1]["body"]["size"], "M|");
            let suggestions = &result["suggestions"][0]["search"];
            if matched {
                assert_eq!(suggestions["candidates"][0]["product_item_id"], "nina-1");
                assert!(
                    suggestions["candidates"][0]["image_url"]
                        .as_str()
                        .unwrap()
                        .ends_with("/photo.png")
                );
            }
        }
        drop(calls);
        if matched {
            // The AI suggestion goes into the cart only when the admin adds it.
            let (status, cart) = request(
                &state,
                &format!("/api/inquiries/{id}/cart"),
                "POST",
                json!({"search_id":lookup["search_id"],"product_item_id":"nina-1"}),
                true,
            )
            .await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(cart["items"][0]["candidate"]["product_item_id"], "nina-1");
            assert_eq!(cart["items"][0]["quantity"], 1);
        }
        state.db.drop().await.unwrap();
        server.abort();
    }
}

#[tokio::test]
#[ignore = "requires disposable MongoDB; run cargo test -- --ignored"]
async fn poster_titles_are_searched_as_alex_categories_by_code() {
    for mode in [8, 9] {
        let (state, _flow, server, id) = fixture(mode).await;
        let job = enqueue(&state, id, true).await;
        draft_worker::process(&state, &job).await.unwrap();
        let inquiry = drafts::inquiry(&state, id).await.unwrap();
        let latest = &drafts::safe_drafts(&state, &inquiry).await.unwrap()[0];
        let item = &latest["result"]["fields"]["items"][0];
        let lookup = &latest["catalog_lookup"][0];
        if mode == 8 {
            assert_eq!(lookup["status"], "MATCHED");
            assert_eq!(item["category"], "haatiya");
            assert_eq!(item["product_code"], "0288HTBP");
            assert_eq!(item["product_name"], "Haatiya - Blush Pink");
            let suggestions = products::list(&state, id).await.unwrap();
            assert_eq!(suggestions["suggestions"][0]["search"]["category"], "HT");
        } else {
            // An inactive or unknown category is never used; the title is searched as a name.
            assert_eq!(lookup["status"], "NOT_FOUND");
            assert!(
                latest["result"]["questions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|question| question
                        .as_str()
                        .unwrap()
                        .contains("ไม่พบประเภท Unknown line"))
            );
        }
        state.db.drop().await.unwrap();
        server.abort();
    }
}

#[tokio::test]
#[ignore = "requires disposable MongoDB; run cargo test -- --ignored"]
async fn summary_changed_during_catalog_lookup_never_commits_old_enrichment() {
    let (state, flow, server, id) = fixture(7).await;
    let job = enqueue(&state, id, false).await;
    let cloned = state.clone();
    let processing = tokio::spawn(async move { draft_worker::process(&cloned, &job).await });
    flow.started.notified().await;
    let snapshot = drafts::snapshot(&state, &drafts::inquiry(&state, id).await.unwrap())
        .await
        .unwrap();
    drafts::save_summary(
        &state,
        id,
        SummaryInput {
            expected_revision: 0,
            text: "Changed booking".into(),
            image_event_ids: vec![],
            context_hash: snapshot.hash,
        },
        &crate::models::customer::Actor {
            id: "admin".into(),
            name: "Admin".into(),
        },
    )
    .await
    .unwrap();
    flow.resume.notify_one();
    assert!(matches!(
        processing.await.unwrap(),
        Err(crate::error::AppError::ContextChanged)
    ));
    assert_eq!(
        drafts::version(&drafts::inquiry(&state, id).await.unwrap()),
        1
    );
    state.db.drop().await.unwrap();
    server.abort();
}
