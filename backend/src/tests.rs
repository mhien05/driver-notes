use crate::{api::router as app, db::AppState, time::now_ms};
use axum::{
    Router,
    http::{Method, StatusCode, header},
};
use axum::{
    body::{Body, to_bytes},
    http::Request,
};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

#[tokio::test]
async fn swagger_documents_live_routes_and_authentication() {
    let db = Connection::open_in_memory().unwrap();
    db.execute_batch(include_str!("../migrations/001_init.sql"))
        .unwrap();
    let app = app(
        AppState {
            db: Arc::new(Mutex::new(db)),
        },
        None,
    );
    let (status, spec) = call(
        &app,
        Method::GET,
        "/api-docs/openapi.json",
        None,
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(spec["openapi"].as_str().unwrap().starts_with('3'));
    assert_eq!(spec["paths"].as_object().unwrap().len(), 11);
    assert_eq!(
        spec["components"]["securitySchemes"]["bearerAuth"]["scheme"],
        "bearer"
    );
    assert!(
        spec["paths"]["/api/v1/auth/register"]["post"]["security"][0]
            .as_object()
            .is_some_and(|requirement| requirement.is_empty())
    );
    assert!(
        spec["paths"]["/api/v1/auth/login"]["post"]["security"][0]
            .as_object()
            .is_some_and(|requirement| requirement.is_empty())
    );
    assert_eq!(
        spec["paths"]["/api/v1/entries"]["post"]["requestBody"]["content"]["application/json"]["schema"]
            ["$ref"],
        "#/components/schemas/EntryInput"
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri("/swagger-ui/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

async fn call(
    app: &Router,
    method: Method,
    path: &str,
    token: Option<&str>,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(token) = token {
        request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, value)
}

#[tokio::test]
async fn deleting_own_shift_keeps_entries_and_rejects_other_users() {
    let db = Connection::open_in_memory().unwrap();
    db.execute_batch(include_str!("../migrations/001_init.sql"))
        .unwrap();
    let app = app(
        AppState {
            db: Arc::new(Mutex::new(db)),
        },
        None,
    );
    let (_, owner) = call(
        &app,
        Method::POST,
        "/api/v1/auth/register",
        None,
        serde_json::json!({"email":"shift-owner@example.com","password":"long-secret-one"}),
    )
    .await;
    let (_, other) = call(
        &app,
        Method::POST,
        "/api/v1/auth/register",
        None,
        serde_json::json!({"email":"other-driver@example.com","password":"long-secret-two"}),
    )
    .await;
    let owner_token = owner["token"].as_str().unwrap();
    let other_token = other["token"].as_str().unwrap();
    let (status, shift) = call(
        &app,
        Method::POST,
        "/api/v1/shifts",
        Some(owner_token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let path = format!("/api/v1/shifts/{}", shift["id"].as_str().unwrap());
    let (status, _) = call(
        &app,
        Method::POST,
        "/api/v1/entries",
        Some(owner_token),
        serde_json::json!({"kind":"bonus","amountVnd":20000,"tipVnd":0,
            "occurredAt":now_ms(),"shiftId":shift["id"]}),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, _) = call(
        &app,
        Method::DELETE,
        &path,
        Some(other_token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = call(
        &app,
        Method::DELETE,
        &path,
        Some(owner_token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, shifts) = call(
        &app,
        Method::GET,
        "/api/v1/shifts",
        Some(owner_token),
        serde_json::json!({}),
    )
    .await;
    let (_, entries) = call(
        &app,
        Method::GET,
        "/api/v1/entries",
        Some(owner_token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(shifts.as_array().unwrap().len(), 0);
    assert_eq!(entries.as_array().unwrap().len(), 1);
    assert!(entries[0]["shiftId"].is_null());
}

#[tokio::test]
async fn users_are_isolated_and_shifts_survive_reloads() {
    let db = Connection::open_in_memory().unwrap();
    db.execute_batch(include_str!("../migrations/001_init.sql"))
        .unwrap();
    let app = app(
        AppState {
            db: Arc::new(Mutex::new(db)),
        },
        None,
    );
    let (status, first) = call(
        &app,
        Method::POST,
        "/api/v1/auth/register",
        None,
        serde_json::json!({"email":"first@example.com","password":"long-secret-one"}),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let first_token = first["token"].as_str().unwrap();
    let (status, second) = call(
        &app,
        Method::POST,
        "/api/v1/auth/register",
        None,
        serde_json::json!({"email":"second@example.com","password":"long-secret-two"}),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let second_token = second["token"].as_str().unwrap();
    let (status, wrong_password) = call(
        &app,
        Method::POST,
        "/api/v1/auth/login",
        None,
        serde_json::json!({"email":"first@example.com","password":"wrong-password"}),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(wrong_password["error"], "Authentication required");
    let (status, logged_in) = call(
        &app,
        Method::POST,
        "/api/v1/auth/login",
        None,
        serde_json::json!({"email":" FIRST@example.com ","password":"long-secret-one"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let login_token = logged_in["token"].as_str().unwrap();
    let (status, _) = call(
        &app,
        Method::POST,
        "/api/v1/auth/logout",
        Some(login_token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = call(
        &app,
        Method::GET,
        "/api/v1/auth/me",
        Some(login_token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, shift) = call(
        &app,
        Method::POST,
        "/api/v1/shifts",
        Some(first_token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let shift_id = shift["id"].as_str().unwrap();
    let (status, _) = call(
        &app,
        Method::POST,
        "/api/v1/shifts",
        Some(first_token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, _) = call(
        &app,
        Method::POST,
        &format!("/api/v1/shifts/{shift_id}/end"),
        Some(second_token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = call(
        &app,
        Method::POST,
        &format!("/api/v1/shifts/{shift_id}/end"),
        Some(first_token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let entry = serde_json::json!({"kind":"delivery","deliveryCount":3,"amountVnd":30000,
            "tipVnd":5000,"occurredAt":now_ms(),"shiftId":shift_id});
    let (status, created) = call(
        &app,
        Method::POST,
        "/api/v1/entries",
        Some(first_token),
        entry,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["deliveryCount"], 3);
    let (status, snapshot) = call(
        &app,
        Method::GET,
        "/api/v1/snapshot",
        Some(second_token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(snapshot["shifts"].as_array().unwrap().len(), 0);
    assert_eq!(snapshot["entries"].as_array().unwrap().len(), 0);
    let (status, snapshot) = call(
        &app,
        Method::GET,
        "/api/v1/snapshot",
        Some(first_token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(snapshot["shifts"].as_array().unwrap().len(), 1);
    assert_eq!(snapshot["entries"].as_array().unwrap().len(), 1);

    let mut invalid_backup = snapshot.clone();
    invalid_backup["entries"][0]["amountVnd"] = serde_json::json!(-1);
    let (status, _) = call(
        &app,
        Method::POST,
        "/api/v1/restore",
        Some(first_token),
        invalid_backup,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, after_invalid) = call(
        &app,
        Method::GET,
        "/api/v1/snapshot",
        Some(first_token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(after_invalid["entries"].as_array().unwrap().len(), 1);

    let (status, restored) = call(
        &app,
        Method::POST,
        "/api/v1/restore",
        Some(first_token),
        snapshot,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(restored["shiftsRestored"], 1);
    assert_eq!(restored["entriesRestored"], 1);
    let (_, after_restore) = call(
        &app,
        Method::GET,
        "/api/v1/snapshot",
        Some(first_token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(
        after_restore["entries"][0]["shiftId"],
        after_restore["shifts"][0]["id"]
    );
}
