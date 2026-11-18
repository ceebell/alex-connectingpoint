use super::product_tests::{CatalogMock, catalog_config, catalog_mock, login_mock};
use super::*;
use crate::{
    db::product_favorites,
    error::AppError,
    models::{
        customer::Actor,
        product_favorites::SaveInput,
        team::{Access, Role},
    },
};
use axum::{
    Json,
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use std::sync::atomic::Ordering;

async fn catalog(
    State(mock): State<CatalogMock>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    if mock.mode.load(Ordering::SeqCst) == 9 {
        return Json(json!({"itemNumber":0,"itemList":[]})).into_response();
    }
    catalog_mock(State(mock), headers, Json(body)).await
}

#[tokio::test]
#[ignore = "requires disposable MongoDB; run cargo test -- --include-ignored"]
async fn favorites_are_shared_per_shop_branch_and_refresh_exact_products_before_adding() {
    let mock = CatalogMock::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server_mock = mock.clone();
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            axum::Router::new()
                .route("/api/user/login", axum::routing::post(login_mock))
                .route("/api/v2/GetProductItems", axum::routing::post(catalog))
                .with_state(server_mock),
        )
        .await
        .unwrap();
    });
    let mut cfg = config();
    cfg.mongodb_uri = std::env::var("TEST_MONGODB_URI").unwrap_or(cfg.mongodb_uri);
    cfg.alex = Some(catalog_config(&format!("http://{address}")));
    let state = AppState::connect(cfg).await.unwrap();
    crate::db::indexes::ensure(&state).await.unwrap();
    let first = ObjectId::new();
    let second = ObjectId::new();
    for id in [first, second] {
        let mut inquiry = state.scope();
        inquiry.extend(doc! {"_id":id,"status":"OPEN","created_at":DateTime::now(),"updated_at":DateTime::now()});
        state
            .collection("inquiries")
            .insert_one(inquiry)
            .await
            .unwrap();
    }
    let path = format!("/api/inquiries/{first}/favorites");
    let shared_path = format!("/api/inquiries/{second}/favorites");
    assert_eq!(
        request(
            &state,
            &format!("{path}?branch_id=branch-a"),
            "GET",
            Value::Null,
            false
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        request(
            &state,
            &path,
            "POST",
            json!({"search_id":ObjectId::new().to_hex(),"product_item_id":"item-1"}),
            false
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    for invalid in [
        path.clone(),
        format!("{path}?branch_id=branch-a&unexpected=x"),
    ] {
        let response = request(&state, &invalid, "GET", Value::Null, true).await;
        assert_eq!(response.0, StatusCode::BAD_REQUEST);
        assert_eq!(response.1["error"]["code"], "BAD_REQUEST");
    }
    assert_eq!(
        request(
            &state,
            &format!("{path}?branch_id=branch-b"),
            "GET",
            Value::Null,
            true
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let (status, search) = request(
        &state,
        &format!("/api/inquiries/{first}/products/search"),
        "POST",
        json!({"branch_id":"branch-a","mode":"code","query":"DR-012"}),
        true,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let input = json!({"search_id":search["search_id"],"product_item_id":"item-1"});
    let (status, favorite) = request(&state, &path, "POST", input.clone(), true).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(favorite["candidate"]["product_item_id"], "item-1");
    let id = favorite["id"].as_str().unwrap();
    let (_, same) = request(&state, &path, "POST", input.clone(), true).await;
    assert_eq!(same["id"], id);
    let concurrent = tokio::join!(
        request(&state, &path, "POST", input.clone(), true),
        request(&state, &path, "POST", input.clone(), true)
    );
    assert_eq!(concurrent.0.1["id"], id);
    assert_eq!(concurrent.1.1["id"], id);
    assert_eq!(all(&state, "product_favorites").await.len(), 1);
    let stored = &all(&state, "product_favorites").await[0];
    assert!(
        stored.contains_key("created_at")
            && stored.contains_key("updated_at")
            && stored.contains_key("created_by_id")
    );
    assert_eq!(
        request(&state, &shared_path, "POST", input.clone(), true)
            .await
            .0,
        StatusCode::CONFLICT
    );
    let mut forged = input.clone();
    forged["product_item_id"] = json!("forged");
    assert_eq!(
        request(&state, &path, "POST", forged, true).await.0,
        StatusCode::CONFLICT
    );
    let mut forged = input.clone();
    forged["candidate"] = json!({"rental_fee_satang":1});
    assert_eq!(
        request(&state, &path, "POST", forged, true).await.0,
        StatusCode::BAD_REQUEST
    );
    let (_, shared) = request(
        &state,
        &format!("{shared_path}?branch_id=branch-a"),
        "GET",
        Value::Null,
        true,
    )
    .await;
    assert_eq!(shared["items"][0]["id"], id);
    let mut unrestricted = state.clone();
    let mut unrestricted_config = (*state.config).clone();
    unrestricted_config
        .alex
        .as_mut()
        .unwrap()
        .branch_ids
        .clear();
    unrestricted.config = Arc::new(unrestricted_config);
    assert_eq!(
        product_favorites::list(&unrestricted, second, "branch-a")
            .await
            .unwrap()["items"][0]["id"],
        id
    );
    let mut restricted = state.clone();
    restricted.access = Some(Access {
        member_id: ObjectId::new(),
        line_user_id: "staff".into(),
        name: "Staff".into(),
        role: Role::Staff,
        branch_ids: vec!["branch-b".into()],
        version: 1,
    });
    assert!(matches!(
        product_favorites::list(&restricted, second, "branch-a").await,
        Err(AppError::AlexScope)
    ));
    assert!(matches!(
        product_favorites::remove(&restricted, second, ObjectId::parse_str(id).unwrap()).await,
        Err(AppError::AlexScope)
    ));
    let mut other_shop = state.clone();
    let mut other_shop_config = (*state.config).clone();
    other_shop_config.shop_id = "other-shop".into();
    other_shop.config = Arc::new(other_shop_config);
    let other_shop_id = ObjectId::new();
    let mut other_inquiry = other_shop.scope();
    other_inquiry.extend(doc! {"_id":other_shop_id,"status":"OPEN"});
    other_shop
        .collection("inquiries")
        .insert_one(other_inquiry)
        .await
        .unwrap();
    assert!(
        product_favorites::list(&other_shop, other_shop_id, "branch-a")
            .await
            .unwrap()["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(matches!(
        product_favorites::remove(&other_shop, other_shop_id, ObjectId::parse_str(id).unwrap())
            .await,
        Err(AppError::NotFound)
    ));
    let mut other = state.clone();
    let mut other_config = (*state.config).clone();
    other_config.channel_id = "other-channel".into();
    other.config = Arc::new(other_config);
    let mut inquiry = other.scope();
    let other_id = ObjectId::new();
    inquiry.extend(doc! {"_id":other_id,"status":"OPEN"});
    other
        .collection("inquiries")
        .insert_one(inquiry)
        .await
        .unwrap();
    assert!(
        product_favorites::list(&other, other_id, "branch-a")
            .await
            .unwrap()["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(matches!(
        product_favorites::remove(&other, other_id, ObjectId::parse_str(id).unwrap()).await,
        Err(AppError::NotFound)
    ));
    assert!(
        product_favorites::save(
            &other,
            other_id,
            SaveInput {
                search_id: search["search_id"].as_str().unwrap().into(),
                product_item_id: "item-1".into()
            },
            &Actor {
                id: "other".into(),
                name: "Other".into()
            }
        )
        .await
        .is_err()
    );
    // Favorites outlive searches and customer/draft changes; only catalog identity is reusable.
    state
        .collection("product_searches")
        .delete_many(state.scope())
        .await
        .unwrap();
    mock.mode.store(4, Ordering::SeqCst);
    let add_path = format!("{shared_path}/{id}/cart");
    let (status, cart) = request(&state, &add_path, "POST", json!({}), true).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cart["items"][0]["candidate"]["rental_fee_satang"], 160000);
    assert_eq!(cart["items"][0]["candidate"]["color"], "R");
    assert_eq!(cart["items"][0]["quantity"], 1);
    let (_, cart) = request(&state, &add_path, "POST", json!({}), true).await;
    assert_eq!(cart["items"][0]["quantity"], 2);
    let (_, refreshed) = request(
        &state,
        &format!("{path}?branch_id=branch-a"),
        "GET",
        Value::Null,
        true,
    )
    .await;
    assert_eq!(
        refreshed["items"][0]["candidate"]["rental_fee_satang"],
        160000
    );
    mock.mode.store(9, Ordering::SeqCst);
    assert_eq!(
        request(&state, &add_path, "POST", json!({}), true).await.0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        all(&state, "cart_items").await[0]
            .get_i64("quantity")
            .unwrap(),
        2
    );
    mock.mode.store(5, Ordering::SeqCst);
    assert_eq!(
        request(&state, &add_path, "POST", json!({}), true).await.0,
        StatusCode::BAD_GATEWAY
    );
    assert_eq!(
        all(&state, "cart_items").await[0]
            .get_i64("quantity")
            .unwrap(),
        2
    );
    assert_eq!(all(&state, "product_favorites").await.len(), 1);
    mock.mode.store(0, Ordering::SeqCst);
    state
        .collection("inquiries")
        .update_one(doc! {"_id":second}, doc! {"$set":{"status":"CLOSED"}})
        .await
        .unwrap();
    assert_eq!(
        request(&state, &add_path, "POST", json!({}), true).await.0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        request(&state, &format!("{path}/{id}"), "DELETE", Value::Null, true)
            .await
            .0,
        StatusCode::OK
    );
    assert!(all(&state, "product_favorites").await.is_empty());
    assert_eq!(all(&state, "cart_items").await.len(), 1);
    state.db.drop().await.unwrap();
    server.abort();
}
