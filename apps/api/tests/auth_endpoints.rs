use std::sync::Arc;
use std::time::Duration;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use minirust_api::{router, AppState};
use minirust_database::Database;
use testcontainers::{core::{IntoContainerPort, WaitFor}, runners::AsyncRunner, GenericImage};
use tokio::time::sleep;
use tower::ServiceExt;

struct TestApp {
    router: Router,
    _database: Arc<TestDatabase>,
}

impl TestApp {
    fn router(&self) -> Router {
        self.router.clone()
    }
}

struct TestDatabase {
    _database: Database,
    _container: testcontainers::ContainerAsync<GenericImage>,
}

async fn test_app() -> TestApp {
    let container = GenericImage::new("mariadb", "11")
        .with_exposed_port(3306.tcp())
        .with_env_var("MARIADB_DATABASE", "minirust_test")
        .with_env_var("MARIADB_USER", "minirust_test")
        .with_env_var("MARIADB_PASSWORD", "minirust_test")
        .with_env_var("MARIADB_ROOT_PASSWORD", "minirust_test_root")
        .with_wait_for(WaitFor::message_on_stdout("ready for connections"))
        .start()
        .await
        .expect("test MariaDB container must start");

    let host = container.get_host().await.expect("test container host must be available");
    let port = container.get_host_port_ipv4(3306).await.expect("test MariaDB port must be available");
    let url = format!("mysql://minirust_test:minirust_test@{host}:{port}/minirust_test");

    let mut database = None;
    for _ in 0..30 {
        if let Ok(candidate) = Database::connect(&url).await {
            database = Some(candidate);
            break;
        }
        sleep(Duration::from_millis(200)).await;
    }
    let database = database.expect("test MariaDB must accept connections");

    database.migrate().await.expect("test database migrations must succeed");
    database.seed_admin("admin@minirust.local", "123456", &[b'a'; 32]).await
        .expect("bootstrap admin seed must succeed");

    let state = AppState::new(database.clone(), vec![b'a'; 32], false)
        .expect("test authentication secret must be valid");

    TestApp {
        router: router(state),
        _database: Arc::new(TestDatabase {
            _database: database,
            _container: container,
        }),
    }
}

#[tokio::test]
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


async fn admin_cookie(app: &Router) -> String {
    let request_response = app
        .clone()
        .oneshot(
            Request::post("/api/v1/auth/login/request-code")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"email":"admin@minirust.local"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(request_response.status(), StatusCode::OK);

    let response = app
        .clone()
        .oneshot(
            Request::post("/api/v1/auth/login/verify-code")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"email":"admin@minirust.local","code":"123456"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    response
        .headers()
        .get("set-cookie")
        .expect("admin login must set a session cookie")
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned()
}

#[tokio::test]
async fn admin_user_crud_and_role_assignment() {
    let app = test_app().await;
    let cookie = admin_cookie(&app).await;

    let list = app
        .clone()
        .oneshot(
            Request::get("/api/v1/admin/users")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list.status(), StatusCode::OK);

    let create = app
        .clone()
        .oneshot(
            Request::post("/api/v1/admin/users")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(r#"{"email":"crud@example.com"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::CREATED);

    let get = app
        .clone()
        .oneshot(
            Request::get("/api/v1/admin/users/crud@example.com")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get.status(), StatusCode::OK);

    let update = app
        .clone()
        .oneshot(
            Request::patch("/api/v1/admin/users/crud@example.com")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(r#"{"email":"updated@example.com"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update.status(), StatusCode::OK);

    let assign = app
        .clone()
        .oneshot(
            Request::put("/api/v1/admin/users/updated@example.com/role")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(r#"{"role":"admin"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(assign.status(), StatusCode::OK);

    let premium_get = app
        .clone()
        .oneshot(
            Request::get("/api/v1/admin/users/updated@example.com/entitlements/premium")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(premium_get.status(), StatusCode::OK);

    let premium = app
        .clone()
        .oneshot(
            Request::put("/api/v1/admin/users/updated@example.com/entitlements/premium")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(r#"{"active":true,"expires_at":null}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(premium.status(), StatusCode::OK);

    let remove_premium = app
        .clone()
        .oneshot(
            Request::put("/api/v1/admin/users/updated@example.com/entitlements/premium")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(r#"{"active":false,"expires_at":null}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(remove_premium.status(), StatusCode::OK);

    let invalid_expiry = app
        .clone()
        .oneshot(
            Request::put("/api/v1/admin/users/updated@example.com/entitlements/premium")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(r#"{"active":true,"expires_at":1}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(invalid_expiry.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let premium_delete = app
        .clone()
        .oneshot(
            Request::delete("/api/v1/admin/users/updated@example.com/entitlements/premium")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(premium_delete.status(), StatusCode::OK);

    let remove_role = app
        .clone()
        .oneshot(
            Request::put("/api/v1/admin/users/updated@example.com/role")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(r#"{"role":"none"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(remove_role.status(), StatusCode::OK);

    let protected_delete = app
        .clone()
        .oneshot(
            Request::delete("/api/v1/admin/users/admin@minirust.local")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(protected_delete.status(), StatusCode::CONFLICT);

    let delete = app
        .clone()
        .oneshot(
            Request::delete("/api/v1/admin/users/updated@example.com")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete.status(), StatusCode::OK);

    let missing = app
        .oneshot(
            Request::get("/api/v1/admin/users/updated@example.com")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn admin_user_endpoints_require_authentication() {
    let app = test_app().await;

    let requests = [
        Request::get("/api/v1/admin/users").body(Body::empty()).unwrap(),
        Request::post("/api/v1/admin/users")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"email":"unauthorized@example.com"}"#))
            .unwrap(),
        Request::get("/api/v1/admin/users/user@example.com")
            .body(Body::empty())
            .unwrap(),
        Request::patch("/api/v1/admin/users/user@example.com")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"email":"updated@example.com"}"#))
            .unwrap(),
        Request::delete("/api/v1/admin/users/user@example.com")
            .body(Body::empty())
            .unwrap(),
        Request::put("/api/v1/admin/users/user@example.com/role")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"role":"admin"}"#))
            .unwrap(),
        Request::put("/api/v1/admin/users/user@example.com/entitlements/premium")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"active":true,"expires_at":null}"#))
            .unwrap(),
    ];

    for request in requests {
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
