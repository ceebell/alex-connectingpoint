use super::*;
use crate::{
    customer_contacts, customer_extract,
    db::{customer_notes, drafts},
    draft_worker,
    error::AppError,
    models::{customer::Actor, customer_notes::CreateInput, draft::PrepareInput},
};
use axum::{Json, extract::State};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Default)]
struct Mock {
    mode: Arc<AtomicUsize>,
    started: Arc<tokio::sync::Notify>,
    resume: Arc<tokio::sync::Notify>,
    calls: Arc<tokio::sync::Mutex<Vec<Value>>>,
}

fn empty() -> Value {
    json!({"fields":{"display_name":null,"phone":null,"address":null},"evidence":[],"conflicts":[],"questions":[]})
}
fn tier(source: Option<&Value>, kind: &str, name: &str, phone: &str) -> Value {
    let Some(source) = source else { return empty() };
    let text = source["text"].as_str().unwrap();
    let reference = source.get("source_ref").unwrap_or(&source["event_ref"]);
    json!({"fields":{"display_name":name,"phone":phone,"address":null},"evidence":[{"field":"display_name","source_type":kind,"source_ref":reference,"quote":text},{"field":"phone","source_type":kind,"source_ref":reference,"quote":text}],"conflicts":[],"questions":[]})
}

fn omit(result: &mut Value, field: &str) {
    result["fields"][field] = Value::Null;
    result["evidence"]
        .as_array_mut()
        .unwrap()
        .retain(|e| e["field"] != field);
}

fn chat_tier(messages: &Value, mode: usize) -> Value {
    let message = messages
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["text"].as_str().unwrap().contains("K.Chat"));
    let mut chat = tier(message, "line_event", "K.Chat", "0833333333");
    if let Some(message) = message {
        chat["fields"]["address"] = json!("99 Road 10110");
        chat["evidence"].as_array_mut().unwrap().push(json!({"field":"address","source_type":"line_event","source_ref":message["event_ref"],"quote":message["text"]}));
    }
    if [1, 3, 8].contains(&mode) {
        omit(&mut chat, "phone");
    }
    if [2, 3, 4, 8].contains(&mode) {
        omit(&mut chat, "display_name");
    }
    if mode == 4 {
        chat["conflicts"] = json!([{"field":"display_name","message":"Conflicting names in chat"}]);
    }
    chat
}

async fn hermes(State(mock): State<Mock>, Json(body): Json<Value>) -> Json<Value> {
    assert_eq!(body["tool_choice"], "none");
    mock.calls.lock().await.push(body.clone());
    let data: Value =
        serde_json::from_str(body["messages"][1]["content"][0]["text"].as_str().unwrap()).unwrap();
    let mode = mock.mode.load(Ordering::SeqCst);
    for forbidden in ["K.Display", "K.OA", "K.Alias", "SECRET_OTHER_CONTACT"] {
        assert!(!data.to_string().contains(forbidden));
    }
    assert!(data.get("display").is_none());
    let result = if body["messages"][0]["content"]
        .as_str()
        .unwrap()
        .starts_with("Resolve customer contact details")
    {
        assert!(!data.to_string().contains("SECRET_OTHER_CONTACT"));
        if mode == 7 {
            mock.started.notify_one();
            mock.resume.notified().await;
        }
        let note = data["notes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["text"].as_str().unwrap().contains("NOTE_CONTACT"));
        let mut notes = tier(note, "customer_note", "K.Note", "081-111-1111");
        let chat = chat_tier(&data["messages"], mode);
        if mode == 8 {
            notes = empty();
        }
        if mode == 5 {
            notes["fields"]["phone"] = json!("0999999999");
        }
        if mode == 6 {
            notes["evidence"][0]["source_ref"] = json!("other-customer-note");
        }
        if mode == 10 {
            notes["evidence"][0]["source_type"] = json!("customer_display");
            notes["evidence"][0]["source_ref"] = json!("current-display");
        }
        let mut result = json!({"notes":notes,"chat":chat});
        if mode == 9 {
            result["display"] = tier(
                Some(&json!({"source_ref":"current-display","text":"K.Display 0822222222"})),
                "customer_display",
                "K.Display",
                "0822222222",
            );
        }
        result
    } else if body["messages"][0]["content"]
        .as_str()
        .unwrap()
        .starts_with("Extract the customer's")
    {
        chat_tier(&data["messages"], mode)
    } else {
        assert_eq!(
            data["customer_fields_resolved_by_server"],
            json!(["customer_name", "customer_phone"])
        );
        json!({"fields":data["fields_schema_example"],"evidence":[],"conflicts":[],"questions":[],"date_options":[],"bank_transfer_slips":[]})
    };
    Json(json!({"choices":[{"finish_reason":"stop","message":{"content":result.to_string()}}]}))
}

async fn setup(mock: &Mock) -> (AppState, tokio::task::JoinHandle<()>, ObjectId) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server_mock = mock.clone();
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            axum::Router::new()
                .route(
                    "/v1/toolsets",
                    axum::routing::get(|| async { Json(json!([])) }),
                )
                .route("/v1/chat/completions", axum::routing::post(hermes))
                .with_state(server_mock),
        )
        .await
        .unwrap();
    });
    let mut cfg = config();
    cfg.mongodb_uri = std::env::var("TEST_MONGODB_URI").unwrap_or(cfg.mongodb_uri);
    cfg.hermes_url = Some(format!("http://{address}/v1/chat/completions"));
    cfg.hermes_api_key = Some("fixture-hermes-key".into());
    cfg.burst_ms = 0;
    let state = AppState::connect(cfg).await.unwrap();
    crate::db::indexes::ensure(&state).await.unwrap();
    for (id, user, text) in [
        (
            "contact-event",
            "Ucurrent",
            "K.Chat 0833333333 99 Road 10110",
        ),
        ("other-event", "Uother", "SECRET_OTHER_CONTACT"),
    ] {
        let mut event = event(id, user, 1);
        event.message = Some(json!({"id":id,"type":"text","text":text}));
        events::persist(&state, &event).await.unwrap();
        let job = events::claim(&state).await.unwrap().unwrap();
        worker::process(&state, &job).await.unwrap();
    }
    state
        .collection("line_users")
        .update_one(
            doc! {"line_user_id":"Ucurrent"},
            doc! {"$set":{"display_name":"K.Display 0822222222"}},
        )
        .await
        .unwrap();
    let inquiry = state
        .collection("inquiries")
        .find_one(doc! {"line_user_id":"Ucurrent"})
        .await
        .unwrap()
        .unwrap()
        .get_object_id("_id")
        .unwrap();
    (state, server, inquiry)
}

async fn note(state: &AppState, user: &str, text: &str, photo: bool) -> (Document, CreateInput) {
    let nonce = ObjectId::new().to_hex();
    let input:CreateInput=serde_json::from_value(json!({"request_id":format!("{}-0000-0000-0000-{}",&nonce[..8],&nonce[12..]),"text":text,"images":if photo{json!([{"name":"fixture.png","image_base64":"iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+yZf8AAAAASUVORK5CYII="}])}else{json!([])}})).unwrap();
    let record = customer_notes::create(
        state,
        user,
        &input,
        input.validate().unwrap(),
        &Actor {
            id: "admin-a".into(),
            name: "Admin A".into(),
        },
    )
    .await
    .unwrap();
    (record, input)
}

#[tokio::test]
#[ignore = "requires disposable MongoDB; run cargo test -- --include-ignored"]
async fn customer_notes_delete_is_scoped_versioned_idempotent_and_hides_private_images() {
    let mock = Mock::default();
    let (state, server, _) = setup(&mock).await;
    let (record, input) = note(&state, "Ucurrent", "NOTE_CONTACT K.Note 081-111-1111", true).await;
    let id = record.get_object_id("_id").unwrap();
    let image = ObjectId::parse_str(
        record.get_array("images").unwrap()[0]
            .as_document()
            .unwrap()
            .get_str("id")
            .unwrap(),
    )
    .unwrap();
    assert!(
        customer_notes::read_image(&state, "Ucurrent", image)
            .await
            .is_ok()
    );
    let path = format!("/api/line/users/Ucurrent/notes/{id}");
    assert_eq!(
        request(
            &state,
            &path,
            "DELETE",
            json!({"expected_version":1}),
            false
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        request(&state, &path, "DELETE", json!({"expected_version":0}), true)
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        request(
            &state,
            &path,
            "DELETE",
            json!({"expected_version":1,"deleted_by":"spoof"}),
            true
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        request(
            &state,
            &format!("/api/line/users/Uother/notes/{id}"),
            "DELETE",
            json!({"expected_version":1}),
            true
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    let mut other = state.clone();
    let mut cfg = (*state.config).clone();
    cfg.channel_id = "other-channel".into();
    other.config = Arc::new(cfg);
    assert!(matches!(
        customer_notes::delete(
            &other,
            "Ucurrent",
            id,
            1,
            &Actor {
                id: "other".into(),
                name: "Other".into()
            }
        )
        .await,
        Err(AppError::NotFound)
    ));
    let content = crate::models::customer_notes::NoteContent {
        text: "Edited contact K.Note 081-111-1111".into(),
        images: vec![crate::models::customer_notes::NoteImage::Existing(image)],
    };
    customer_notes::edit(
        &state,
        "Ucurrent",
        id,
        1,
        content,
        &Actor {
            id: "admin-b".into(),
            name: "Admin B".into(),
        },
    )
    .await
    .unwrap();
    assert_eq!(
        request(&state, &path, "DELETE", json!({"expected_version":1}), true)
            .await
            .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        request(&state, &path, "DELETE", json!({"expected_version":2}), true)
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        request(&state, &path, "DELETE", json!({"expected_version":2}), true)
            .await
            .0,
        StatusCode::OK
    );
    assert!(
        customer_notes::list(&state, "Ucurrent", None)
            .await
            .unwrap()
            .0
            .is_empty()
    );
    assert!(
        customer_notes::list(&state, "Ucurrent", Some(id))
            .await
            .is_ok()
    );
    assert!(matches!(
        customer_notes::get(&state, "Ucurrent", id).await,
        Err(AppError::NotFound)
    ));
    assert!(matches!(
        customer_notes::read_image(&state, "Ucurrent", image).await,
        Err(AppError::NotFound)
    ));
    assert!(matches!(
        customer_notes::create(
            &state,
            "Ucurrent",
            &input,
            input.validate().unwrap(),
            &Actor {
                id: "a".into(),
                name: "A".into()
            }
        )
        .await,
        Err(AppError::RecordConflict)
    ));
    let raw = state
        .collection("customer_notes")
        .find_one(doc! {"_id":id})
        .await
        .unwrap()
        .unwrap();
    assert_eq!(raw.get_i64("version").unwrap(), 3);
    assert!(raw.contains_key("deleted_at"));
    assert_eq!(raw.get_array("history").unwrap().len(), 3);
    assert_eq!(
        raw.get_array("history").unwrap()[2]
            .as_document()
            .unwrap()
            .get_str("operation")
            .unwrap(),
        "DELETE"
    );
    assert!(!raw.get_str("deleted_by_id").unwrap().is_empty());
    state.db.drop().await.unwrap();
    server.abort();
}

#[tokio::test]
#[ignore = "requires disposable MongoDB; run cargo test -- --include-ignored"]
async fn customer_contacts_prioritize_chat_then_notes_and_exclude_all_display_names() {
    let mock = Mock::default();
    let (state, server, id) = setup(&mock).await;
    let (record, _) = note(
        &state,
        "Ucurrent",
        "NOTE_CONTACT K.Note 081-111-1111",
        false,
    )
    .await;
    note(&state, "Uother", "SECRET_OTHER_CONTACT", false).await;
    let path = format!("/api/inquiries/{id}/customer-extract");
    for (mode, name, phone) in [
        (0, "K.Chat", "0833333333"),
        (1, "K.Chat", "0811111111"),
        (2, "K.Note", "0833333333"),
        (3, "K.Note", "0811111111"),
    ] {
        mock.mode.store(mode, Ordering::SeqCst);
        let (status, result) = request(&state, &path, "POST", json!({}), true).await;
        assert_eq!(status, StatusCode::OK, "{result}");
        assert_eq!(result["fields"]["display_name"], name);
        assert_eq!(result["fields"]["phone"], phone);
        assert_eq!(result["fields"]["address"], "99 Road 10110");
        for (field, from_note) in [
            ("display_name", mode == 2 || mode == 3),
            ("phone", mode == 1 || mode == 3),
        ] {
            let proof = result["evidence"]
                .as_array()
                .unwrap()
                .iter()
                .find(|e| e["field"] == field)
                .unwrap();
            assert_eq!(
                proof["source_type"],
                if from_note {
                    "customer_note"
                } else {
                    "line_event"
                }
            );
        }
    }
    mock.mode.store(4, Ordering::SeqCst);
    let (_, result) = request(&state, &path, "POST", json!({}), true).await;
    assert!(result["fields"]["display_name"].is_null());
    assert_eq!(result["conflicts"][0]["field"], "display_name");
    for mode in [5, 6, 10] {
        mock.mode.store(mode, Ordering::SeqCst);
        assert_eq!(
            request(&state, &path, "POST", json!({}), true).await.0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    mock.mode.store(9, Ordering::SeqCst);
    assert_ne!(
        request(&state, &path, "POST", json!({}), true).await.0,
        StatusCode::OK
    );
    mock.mode.store(8, Ordering::SeqCst);
    let (status, missing) = request(&state, &path, "POST", json!({}), true).await;
    assert_eq!(status, StatusCode::OK);
    assert!(missing["fields"]["display_name"].is_null());
    assert!(missing["fields"]["phone"].is_null());
    mock.mode.store(0, Ordering::SeqCst);
    let note_id = record.get_object_id("_id").unwrap();
    customer_notes::delete(
        &state,
        "Ucurrent",
        note_id,
        1,
        &Actor {
            id: "a".into(),
            name: "A".into(),
        },
    )
    .await
    .unwrap();
    let (_, result) = request(&state, &path, "POST", json!({}), true).await;
    assert_eq!(result["fields"]["display_name"], "K.Chat");
    assert!(
        result["evidence"]
            .as_array()
            .unwrap()
            .iter()
            .all(|e| e["source_type"] != "customer_note")
    );
    let contacts_before_profile_changes =
        customer_contacts::load(&state, "Ucurrent").await.unwrap();
    state
        .collection("line_users")
        .update_one(
            doc! {"line_user_id":"Ucurrent"},
            doc! {"$set":{"oa_display_name":"K.OA 0855555555"}},
        )
        .await
        .unwrap();
    assert_eq!(
        request(&state, &path, "POST", json!({}), true).await.1["fields"]["display_name"],
        "K.Chat"
    );
    crate::db::customers::save_alias(
        &state,
        "Ucurrent",
        crate::models::customer_notes::AliasInput {
            expected_version: 0,
            custom_name: Some("K.Alias 0844444444".into()),
        },
        &Actor {
            id: "a".into(),
            name: "A".into(),
        },
    )
    .await
    .unwrap();
    assert_eq!(
        request(&state, &path, "POST", json!({}), true).await.1["fields"]["display_name"],
        "K.Chat"
    );
    assert_eq!(
        customer_contacts::load(&state, "Ucurrent")
            .await
            .unwrap()
            .hash,
        contacts_before_profile_changes.hash
    );
    mock.mode.store(8, Ordering::SeqCst);
    let (_, missing) = request(&state, &path, "POST", json!({}), true).await;
    assert!(missing["fields"]["display_name"].is_null());
    assert!(missing["fields"]["phone"].is_null());
    state.db.drop().await.unwrap();
    server.abort();
}

#[tokio::test]
#[ignore = "requires disposable MongoDB; run cargo test -- --include-ignored"]
async fn customer_contact_changes_during_extraction_fail_and_ai_drafts_use_chat_then_notes() {
    let mock = Mock::default();
    let (state, server, id) = setup(&mock).await;
    let (record, _) = note(
        &state,
        "Ucurrent",
        "NOTE_CONTACT K.Note 081-111-1111",
        false,
    )
    .await;
    let note_id = record.get_object_id("_id").unwrap();
    mock.mode.store(7, Ordering::SeqCst);
    let current = state.clone();
    let pending = tokio::spawn(async move { customer_extract::extract(&current, id).await });
    mock.started.notified().await;
    customer_notes::delete(
        &state,
        "Ucurrent",
        note_id,
        1,
        &Actor {
            id: "a".into(),
            name: "A".into(),
        },
    )
    .await
    .unwrap();
    mock.resume.notify_one();
    assert!(matches!(
        pending.await.unwrap(),
        Err(AppError::ContextChanged)
    ));
    mock.mode.store(1, Ordering::SeqCst);
    note(
        &state,
        "Ucurrent",
        "NOTE_CONTACT K.Note 081-111-1111",
        false,
    )
    .await;
    let input = PrepareInput {
        request_id: "contact-priority-draft".into(),
        expected_version: 0,
        summary: "Prepare order from these customer details".into(),
        image_event_ids: vec![],
        fields: None,
        confirmed_fields: vec![],
        context_hash: None,
    };
    drafts::enqueue(&state, id, input, false).await.unwrap();
    let job = drafts::claim(&state).await.unwrap().unwrap();
    draft_worker::process(&state, &job).await.unwrap();
    let inquiry = drafts::inquiry(&state, id).await.unwrap();
    let draft = drafts::active(&state, &inquiry).await.unwrap().unwrap();
    let result = draft.get_document("result").unwrap();
    let fields = result.get_document("fields").unwrap();
    assert_eq!(fields.get_str("customer_name").unwrap(), "K.Chat");
    assert_eq!(fields.get_str("customer_phone").unwrap(), "0811111111");
    assert!(
        result.get_array("evidence").unwrap().iter().any(|e| e
            .as_document()
            .unwrap()
            .get_str("source_type")
            .unwrap()
            == "customer_note")
    );
    let context = customer_contacts::load(&state, "Ucurrent").await.unwrap();
    assert!(!context.note_sources.is_empty());
    state.db.drop().await.unwrap();
    server.abort();
}
