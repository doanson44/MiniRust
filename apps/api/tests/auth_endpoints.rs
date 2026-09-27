use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use minirust_api::{router, AppState};
use minirust_database::Database;
use tower::ServiceExt;

async fn test_app() -> axum::Router {
    let url = std::env::var("MINIRUST_TEST_DATABASE_URL")
        .expect("MINIRUST_TEST_DATABASE_URL must be set for auth integration tests");
    let database = Database::connect(&url)
        .await
        .expect("test MariaDB must be reachable");
    database
        .migrate()
        .await
        .expect("test database migrations must succeed");

    let state = AppState::new(database, vec![b'a'; 32], false)
        .expect("test authentication secret must be valid");
    router(state)
}

#[tokio::test]
#[ignore = "requires an isolated MariaDB test database"]
async fn register_request_rejects_invalid_email() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::post("/api/v1/auth/register/request-code")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"email":"not-an-email"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
#[ignore = "requires an isolated MariaDB test database"]
async fn login_request_rejects_invalid_email() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::post("/api/v1/auth/login/request-code")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"email":"not-an-email"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
#[ignore = "requires an isolated MariaDB test database"]
async fn register_verify_rejects_unknown_code() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::post("/api/v1/auth/register/verify-code")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"email":"user@example.com","code":"123456"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
#[ignore = "requires an isolated MariaDB test database"]
async fn login_verify_rejects_unknown_code() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::post("/api/v1/auth/login/verify-code")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"email":"user@example.com","code":"123456"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
#[ignore = "requires an isolated MariaDB test database"]
async fn logout_without_session_is_successful_and_clears_cookie() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::post("/api/v1/auth/logout")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().contains_key("set-cookie"));
}

#[tokio::test]
#[ignore = "requires an isolated MariaDB test database"]
async fn me_without_session_is_unauthorized() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::get("/api/v1/auth/me")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
