use super::*;

fn row(n: usize, inquiry: ObjectId, scope: Document) -> Document {
    let mut record = scope;
    record.extend(doc! {"_id":ObjectId::parse_str(format!("{n:024x}")).unwrap(),"inquiry_id":inquiry,
        "event_at":DateTime::from_millis(1000 + (n/3) as i64 * 1000),"event_id":format!("group-{:04}",n/3),
        "status":"DONE","created_at":DateTime::now(),"updated_at":DateTime::now(),
        "request_id":format!("request-{n}"),"bot_id":"test-destination","external_message_id":format!("oa-{n}"),"oa_chat_id":"fixture","oa_message_id":format!("oa-{n}"),
        "payload":{"type":"message","message":{"type":"text","text":format!("row-{n}")}}});
    record
}

fn rows(page: &Value) -> Vec<&Value> {
    page["batches"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|batch| batch.as_array().unwrap())
        .collect()
}

#[tokio::test]
#[ignore = "requires disposable MongoDB; run cargo test -- --include-ignored"]
async fn latest_chat_pages_cover_all_sources_timestamp_ties_late_messages_and_withdrawn_cursors() {
    let mut cfg = config();
    cfg.mongodb_uri = std::env::var("TEST_MONGODB_URI").unwrap_or(cfg.mongodb_uri);
    let state = AppState::connect(cfg).await.unwrap();
    crate::db::indexes::ensure(&state).await.unwrap();
    let inquiry = ObjectId::new();
    let other = ObjectId::new();
    for id in [inquiry, other] {
        let mut doc = state.scope();
        doc.extend(doc! {"_id":id,"status":"OPEN"});
        state.collection("inquiries").insert_one(doc).await.unwrap();
    }
    for (index, collection) in crate::db::oa_sync::MESSAGE_COLLECTIONS
        .into_iter()
        .enumerate()
    {
        state
            .collection(collection)
            .insert_many(
                (index..650)
                    .step_by(3)
                    .map(|n| row(n, inquiry, state.scope())),
            )
            .await
            .unwrap();
    }
    let path = format!("/api/inquiries/{inquiry}/context");
    assert_eq!(
        request(
            &state,
            &format!("{path}?latest=true"),
            "GET",
            Value::Null,
            false
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    let (_, legacy) = request(&state, &path, "GET", Value::Null, true).await;
    assert_eq!(rows(&legacy).len(), 200);
    assert_eq!(rows(&legacy)[0]["payload"]["message"]["text"], "row-0");
    assert!(legacy["next"].is_string());
    assert!(legacy["previous"].is_null());
    let (status, latest) = request(
        &state,
        &format!("{path}?latest=true"),
        "GET",
        Value::Null,
        true,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(rows(&latest).len(), 200);
    assert_eq!(rows(&latest)[0]["payload"]["message"]["text"], "row-450");
    assert_eq!(rows(&latest)[199]["payload"]["message"]["text"], "row-649");
    assert!(latest["next"].is_null());
    let boundary = latest["previous"].as_str().unwrap().to_string();
    let mut page = latest;
    let mut seen = std::collections::BTreeSet::new();
    let mut lengths = vec![];
    loop {
        let page_rows = rows(&page);
        lengths.push(page_rows.len());
        for item in page_rows {
            assert!(
                seen.insert(item["_id"].to_string()),
                "duplicate row at a page boundary"
            );
        }
        let Some(before) = page["previous"].as_str() else {
            break;
        };
        let response = request(
            &state,
            &format!("{path}?latest=true&before={before}"),
            "GET",
            Value::Null,
            true,
        )
        .await;
        assert_eq!(response.0, StatusCode::OK);
        page = response.1;
    }
    assert_eq!(lengths, vec![200, 200, 200, 50]);
    assert_eq!(seen.len(), 650);
    // A withdrawn boundary remains a valid scoped cursor, without rendering the withdrawn row.
    state
        .collection("line_events")
        .update_one(
            doc! {"_id":ObjectId::parse_str(&boundary).unwrap()},
            doc! {"$set":{"withdrawn":true}},
        )
        .await
        .unwrap();
    let (status, page) = request(
        &state,
        &format!("{path}?latest=true&before={boundary}"),
        "GET",
        Value::Null,
        true,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(rows(&page).len(), 200);
    let mut newest = row(651, inquiry, state.scope());
    newest.insert("event_at", DateTime::from_millis(2_000_000));
    let mut late = row(652, inquiry, state.scope());
    late.insert("event_id", "late-row-652");
    late.insert("event_at", DateTime::from_millis(180_500));
    state
        .collection("line_events")
        .insert_many([newest, late])
        .await
        .unwrap();
    let (_, page) = request(
        &state,
        &format!("{path}?latest=true"),
        "GET",
        Value::Null,
        true,
    )
    .await;
    assert_eq!(
        rows(&page).last().unwrap()["payload"]["message"]["text"],
        "row-651"
    );
    assert!(
        rows(&page)
            .iter()
            .any(|item| item["payload"]["message"]["text"] == "row-652")
    );
    for query in [
        format!("latest=true&after={boundary}"),
        format!("before={boundary}"),
        "latest=invalid".into(),
    ] {
        let result = request(&state, &format!("{path}?{query}"), "GET", Value::Null, true).await;
        assert_eq!(result.0, StatusCode::BAD_REQUEST);
        assert_eq!(result.1["error"]["code"], "BAD_REQUEST");
    }
    assert_eq!(
        request(
            &state,
            &format!("/api/inquiries/{other}/context?latest=true&before={boundary}"),
            "GET",
            Value::Null,
            true
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    let mut outside = state.scope();
    outside.insert("shop_id", "another-shop");
    let outside_id = ObjectId::new();
    let mut record = row(700, inquiry, outside);
    record.insert("_id", outside_id);
    state
        .collection("line_events")
        .insert_one(record)
        .await
        .unwrap();
    assert_eq!(
        request(
            &state,
            &format!("{path}?latest=true&before={outside_id}"),
            "GET",
            Value::Null,
            true
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    state.db.drop().await.unwrap();
}
