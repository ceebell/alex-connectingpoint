use super::*;
use crate::{
    db::{customer_notes, customers},
    models::{
        customer::Actor,
        customer_notes::{AliasInput, CreateInput, EditInput},
    },
};

fn create_input(request: &str, text: &str, image: bool) -> CreateInput {
    serde_json::from_value(json!({"request_id":request,"text":text,"images":if image {json!([{"name":"photo.png","image_base64":STANDARD.encode(b"\x89PNG\r\n\x1a\nnote-test")}])} else {json!([])}})).unwrap()
}

#[test]
fn customer_notes_validate_calendar_independent_text_and_private_images() {
    let input = create_input("00000000-0000-0000-0000-000000000001", " note ", true);
    let content = input.validate().unwrap();
    assert_eq!(content.text, "note");
    assert_eq!(content.images.len(), 1);
    for value in [
        json!({"request_id":"bad","text":"note"}),
        json!({"request_id":input.request_id,"text":""}),
        json!({"request_id":input.request_id,"text":"x\u{0000}"}),
        json!({"request_id":input.request_id,"text":"x".repeat(5001)}),
        json!({"request_id":input.request_id,"text":"note","images":[{"id":ObjectId::new().to_hex()}]}),
        json!({"request_id":input.request_id,"text":"note","images":[{"name":"file.svg","image_base64":STANDARD.encode(b"<svg></svg>")}]}),
        json!({"request_id":input.request_id,"text":"note","images":[{"name":"file.png","image_base64":STANDARD.encode(vec![0_u8;2*1024*1024+1])}]}),
    ] {
        assert!(
            serde_json::from_value::<CreateInput>(value)
                .unwrap()
                .validate()
                .is_err()
        );
    }
    assert!(
        serde_json::from_value::<CreateInput>(
            json!({"request_id":input.request_id,"text":"note","updated_by":"spoof"})
        )
        .is_err()
    );
    let image = ObjectId::new().to_hex();
    let duplicate: EditInput = serde_json::from_value(
        json!({"expected_version":1,"text":"note","images":[{"id":image},{"id":image}]}),
    )
    .unwrap();
    assert!(duplicate.validate().is_err());
    let alias = AliasInput {
        expected_version: 0,
        custom_name: Some("  ชื่อตั้งเอง  ".into()),
    }
    .validate()
    .unwrap();
    assert_eq!(alias.custom_name.as_deref(), Some("ชื่อตั้งเอง"));
    assert!(
        AliasInput {
            expected_version: -1,
            custom_name: None
        }
        .validate()
        .is_err()
    );
    assert!(
        AliasInput {
            expected_version: 0,
            custom_name: Some("x\nname".into())
        }
        .validate()
        .is_err()
    );
}

#[tokio::test]
async fn customer_notes_routes_authorize_and_validate_before_database_access() {
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
    for (path, method, body) in [
        (
            "/api/line/users/Uvalid/alias",
            "POST",
            json!({"expected_version":0,"custom_name":"name"}),
        ),
        ("/api/line/users/Uvalid/notes", "GET", json!(null)),
        (
            "/api/line/users/Uvalid/notes",
            "POST",
            json!({"request_id":"bad","text":"note"}),
        ),
        (
            "/api/line/users/Uvalid/note-images/000000000000000000000001",
            "GET",
            json!(null),
        ),
    ] {
        assert_eq!(
            request(&state, path, method, body, false).await.0,
            StatusCode::UNAUTHORIZED
        );
    }
    for (path, body, status) in [
        (
            "/api/line/users/bad.id/alias",
            json!({"expected_version":0,"custom_name":"name"}),
            StatusCode::BAD_REQUEST,
        ),
        (
            "/api/line/users/Uvalid/alias",
            json!({"expected_version":-1,"custom_name":"name"}),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            "/api/line/users/Uvalid/notes",
            json!({"request_id":"bad","text":"note"}),
            StatusCode::BAD_REQUEST,
        ),
        (
            "/api/line/users/Uvalid/notes",
            json!({"request_id":"00000000-0000-0000-0000-000000000001","text":"note","created_by":"spoof"}),
            StatusCode::BAD_REQUEST,
        ),
        (
            "/api/line/users/Uvalid/notes/not-an-id",
            json!({"expected_version":1,"text":"note"}),
            StatusCode::BAD_REQUEST,
        ),
    ] {
        assert_eq!(request(&state, path, "POST", body, true).await.0, status);
    }
}

#[tokio::test]
#[ignore = "requires disposable MongoDB; run cargo test mongo_customer_aliases_and_notes -- --ignored"]
async fn mongo_customer_aliases_and_notes() {
    let mut cfg = config();
    cfg.mongodb_uri = std::env::var("TEST_MONGODB_URI").unwrap_or(cfg.mongodb_uri);
    let state = AppState::connect(cfg).await.unwrap();
    crate::db::indexes::ensure(&state).await.unwrap();
    let now = DateTime::now();
    let user = "Unotes";
    let other = "Uother";
    for user in [user, other] {
        let mut row = state.scope();
        row.extend(doc! {"_id":ObjectId::new(),"line_user_id":user,"display_name":"LINE name","oa_display_name":"OA name","profile_checked_at":now,"created_at":now,"updated_at":now});
        state
            .collection("line_users")
            .insert_one(row)
            .await
            .unwrap();
    }
    let a = Actor {
        id: "line:admin-a".into(),
        name: "Admin A".into(),
    };
    let b = Actor {
        id: "line:admin-b".into(),
        name: "Admin B".into(),
    };
    let alias = customers::save_alias(
        &state,
        user,
        AliasInput {
            expected_version: 0,
            custom_name: Some("Local name".into()),
        },
        &a,
    )
    .await
    .unwrap();
    assert_eq!(alias.get_i64("version").unwrap(), 1);
    assert!(matches!(
        customers::save_alias(
            &state,
            user,
            AliasInput {
                expected_version: 0,
                custom_name: Some("stale".into())
            },
            &b
        )
        .await,
        Err(crate::error::AppError::RecordConflict)
    ));
    let inquiry = ObjectId::new();
    let mut row = state.scope();
    row.extend(doc! {"_id":inquiry,"line_user_id":user,"conversation_id":ObjectId::new(),"last_activity_at":now,"created_at":now,"updated_at":now,"status":"RECEIVED"});
    state.collection("inquiries").insert_one(row).await.unwrap();
    let inbox = request(&state, "/api/inquiries", "GET", json!(null), true)
        .await
        .1;
    assert_eq!(inbox[0]["customer"]["display_name"], "Local name");
    assert_eq!(inbox[0]["customer"]["line_display_name"], "OA name");
    let mut filter = state.scope();
    filter.insert("line_user_id", user);
    state
        .collection("line_users")
        .update_one(filter, doc! {"$set":{"oa_display_name":"OA changed"}})
        .await
        .unwrap();
    assert_eq!(
        request(&state, "/api/inquiries", "GET", json!(null), true)
            .await
            .1[0]["customer"]["display_name"],
        "Local name"
    );
    customers::save_alias(
        &state,
        user,
        AliasInput {
            expected_version: 1,
            custom_name: None,
        },
        &b,
    )
    .await
    .unwrap();
    assert_eq!(
        request(&state, "/api/inquiries", "GET", json!(null), true)
            .await
            .1[0]["customer"]["display_name"],
        "OA changed"
    );
    let input = create_input("00000000-0000-0000-0000-000000000001", "first note", true);
    let note = customer_notes::create(&state, user, &input, input.validate().unwrap(), &a)
        .await
        .unwrap();
    let id = note.get_object_id("_id").unwrap();
    let image = note.get_array("images").unwrap()[0].as_document().unwrap();
    let image_id = ObjectId::parse_str(image.get_str("id").unwrap()).unwrap();
    let retry = customer_notes::create(&state, user, &input, input.validate().unwrap(), &a)
        .await
        .unwrap();
    assert_eq!(retry.get_object_id("_id").unwrap(), id);
    let changed = create_input(&input.request_id, "different content", false);
    assert!(matches!(
        customer_notes::create(&state, user, &changed, changed.validate().unwrap(), &a).await,
        Err(crate::error::AppError::RecordConflict)
    ));
    let file = state
        .db
        .gridfs_bucket(None)
        .find_one(doc! {"_id":image_id})
        .await
        .unwrap()
        .unwrap();
    assert!(!file.metadata.unwrap().contains_key("expires_at"));
    assert_eq!(
        customer_notes::read_image(&state, user, image_id)
            .await
            .unwrap()
            .1,
        b"\x89PNG\r\n\x1a\nnote-test"
    );
    assert!(matches!(
        customer_notes::read_image(&state, other, image_id).await,
        Err(crate::error::AppError::NotFound)
    ));
    let edit: EditInput =
        serde_json::from_value(json!({"expected_version":1,"text":"edited note","images":[]}))
            .unwrap();
    let edited = customer_notes::edit(&state, user, id, 1, edit.validate().unwrap(), &b)
        .await
        .unwrap();
    assert_eq!(edited.get_i64("version").unwrap(), 2);
    assert_eq!(edited.get_str("created_by").unwrap(), "Admin A");
    assert_eq!(edited.get_str("updated_by").unwrap(), "Admin B");
    assert_eq!(edited.get_str("updated_by_id").unwrap(), "line:admin-b");
    let history = edited.get_array("history").unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(
        history[0].as_document().unwrap().get_str("text").unwrap(),
        "first note"
    );
    assert_eq!(
        history[0]
            .as_document()
            .unwrap()
            .get_array("images")
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        history[1]
            .as_document()
            .unwrap()
            .get_str("updated_by")
            .unwrap(),
        "Admin B"
    );
    assert!(matches!(
        customer_notes::edit(&state, user, id, 1, edit.validate().unwrap(), &a).await,
        Err(crate::error::AppError::RecordConflict)
    ));
    let retained: EditInput = serde_json::from_value(
        json!({"expected_version":2,"text":"restored image","images":[{"id":image_id.to_hex()}]}),
    )
    .unwrap();
    customer_notes::edit(&state, user, id, 2, retained.validate().unwrap(), &a)
        .await
        .unwrap();
    let wrong_note_input =
        create_input("00000000-0000-0000-0000-000000000002", "second note", false);
    let wrong_note = customer_notes::create(
        &state,
        user,
        &wrong_note_input,
        wrong_note_input.validate().unwrap(),
        &a,
    )
    .await
    .unwrap();
    assert!(matches!(
        customer_notes::edit(
            &state,
            user,
            wrong_note.get_object_id("_id").unwrap(),
            1,
            retained.validate().unwrap(),
            &b
        )
        .await,
        Err(crate::error::AppError::NotFound)
    ));
    assert!(matches!(
        customer_notes::get(&state, other, id).await,
        Err(crate::error::AppError::NotFound)
    ));
    let mut other_scope = state.clone();
    let mut other_config = (*state.config).clone();
    other_config.channel_id = "other-channel".into();
    other_scope.config = Arc::new(other_config);
    assert!(matches!(
        customer_notes::get(&other_scope, user, id).await,
        Err(crate::error::AppError::NotFound)
    ));
    assert!(matches!(
        customer_notes::read_image(&other_scope, user, image_id).await,
        Err(crate::error::AppError::NotFound)
    ));
    for index in 0..53 {
        let mut row = state.scope();
        row.extend(doc! {"_id":ObjectId::new(),"line_user_id":other,"request_id":format!("page-{index}"),"text":format!("note {index}"),"images":[],"version":1_i64,"created_at":DateTime::from_millis(now.timestamp_millis()+index),"updated_at":now});
        state
            .collection("customer_notes")
            .insert_one(row)
            .await
            .unwrap();
    }
    let (page, next) = customer_notes::list(&state, other, None).await.unwrap();
    assert_eq!(page.len(), 50);
    assert_eq!(page[0].get_str("text").unwrap(), "note 52");
    let (remaining, last) = customer_notes::list(
        &state,
        other,
        Some(ObjectId::parse_str(next.unwrap()).unwrap()),
    )
    .await
    .unwrap();
    assert_eq!(remaining.len(), 3);
    assert!(last.is_none());
    assert_eq!(remaining[2].get_str("text").unwrap(), "note 0");
    assert!(customer_notes::list(&state, other, Some(id)).await.is_err());
    state.db.drop().await.unwrap();
}
