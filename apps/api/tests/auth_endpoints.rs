use std::sync::Once;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use minirust_api::{router, AppState};
use minirust_database::Database;
use sqlx::{Connection, Executor, MySqlConnection};
use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    GenericImage, ImageExt,
};
use tokio::sync::OnceCell;
use tower::ServiceExt;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

struct TestApp {
    router: Router,
}

impl TestApp {
    fn router(&self) -> Router {
        self.router.clone()
    }
}

static INIT_TRACING: Once = Once::new();
static TEST_MARIADB: OnceCell<TestMariaDb> = OnceCell::const_new();

fn init_test_tracing() {
    INIT_TRACING.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::from_default_env().add_directive(tracing::Level::ERROR.into()),
            )
            .with_test_writer()
            .try_init();
    });
}

struct TestMariaDb {
    _container: testcontainers::ContainerAsync<GenericImage>,
    host: String,
    port: u16,
}

async fn test_mariadb() -> &'static TestMariaDb {
    TEST_MARIADB
        .get_or_init(|| async {
            let container = GenericImage::new("mariadb", "11")
                .with_wait_for(WaitFor::message_on_stderr("ready for connections"))
                .with_exposed_port(3306.tcp())
                .with_env_var("MARIADB_DATABASE", "minirust_test")
                .with_env_var("MARIADB_USER", "minirust_test")
                .with_env_var("MARIADB_PASSWORD", "minirust_test")
                .with_env_var("MARIADB_ROOT_PASSWORD", "minirust_test_root")
                .start()
                .await
                .expect("test MariaDB container must start");

            let host = container
                .get_host()
                .await
                .expect("test container host must be available");
            let port = container
                .get_host_port_ipv4(3306)
                .await
                .expect("test MariaDB port must be available");

            TestMariaDb {
                _container: container,
                host: host.to_string(),
                port,
            }
        })
        .await
}

async fn test_app() -> TestApp {
    init_test_tracing();

    let mariadb = test_mariadb().await;
    let database_name = format!("minirust_test_{}", Uuid::now_v7().simple());
    let url = format!(
        "mysql://root:minirust_test_root@{}:{}/{}",
        mariadb.host, mariadb.port, database_name
    );

    let maintenance_url = format!(
        "mysql://root:minirust_test_root@{}:{}/minirust_test",
        mariadb.host, mariadb.port
    );
    let mut retries = 5;
    let mut maintenance = loop {
        match MySqlConnection::connect(&maintenance_url).await {
            Ok(connection) => break connection,
            Err(_error) if retries > 0 => {
                retries -= 1;
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
            Err(error) => panic!("test MariaDB maintenance connection failed: {error}"),
        }
    };

    let create_database = format!("CREATE DATABASE `{database_name}`");
    maintenance
        .execute(create_database.as_str())
        .await
        .expect("test database must be created");

    let database = Database::connect(&url)
        .await
        .expect("test MariaDB must accept connections");
    database
        .migrate()
        .await
        .expect("test database migrations must succeed");
    database
        .seed_admin("admin@minirust.local", "123456", &[b'a'; 32])
        .await
        .expect("bootstrap admin seed must succeed");

    let state = AppState::new(database, vec![b'a'; 32], false)
        .expect("test authentication secret must be valid");

    TestApp {
        router: router(state),
    }
}

#[tokio::test]
async fn register_request_rejects_invalid_email() {
    let app = test_app().await;

    let response = app
        .router()
        .oneshot(
            Request::post("/api/v1/auth/register/request-verification")
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
        .router()
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
        .router()
        .oneshot(
            Request::post("/api/v1/auth/register/verify")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"token":"invalid-token"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn local_registration_returns_verification_link() {
    let app = test_app().await;

    let response = app
        .router()
        .oneshot(
            Request::post("/api/v1/auth/register/request-verification")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"email":"local@example.com"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let verification_url = body["data"]["verification_url"]
        .as_str()
        .expect("local registration must return a verification link");
    assert!(verification_url.starts_with("/register/verify?token="));

    let token = verification_url
        .strip_prefix("/register/verify?token=")
        .unwrap();
    let verify = app
        .router()
        .oneshot(
            Request::post("/api/v1/auth/register/verify")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::json!({"token": token}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(verify.status(), StatusCode::OK);
    assert!(verify.headers().contains_key("set-cookie"));
}


#[tokio::test]
async fn register_existing_email_reports_already_registered() {
    let app = test_app().await;

    let first = app
        .router()
        .oneshot(
            Request::post("/api/v1/auth/register/request-verification")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"email":"existing@example.com"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);

    let first_body = axum::body::to_bytes(first.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let first_body: serde_json::Value = serde_json::from_slice(&first_body).unwrap();
    let token = first_body["data"]["verification_url"]
        .as_str()
        .and_then(|url| url.strip_prefix("/register/verify?token="))
        .expect("registration must return a local verification token");

    let verify = app
        .router()
        .oneshot(
            Request::post("/api/v1/auth/register/verify")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::json!({"token": token}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(verify.status(), StatusCode::OK);

    let response = app
        .router()
        .oneshot(
            Request::post("/api/v1/auth/register/request-verification")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"email":"existing@example.com"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body["data"]["accepted"], true);
    assert_eq!(body["data"]["email_exists"], true);
    assert_eq!(body["data"]["verification_url"], serde_json::Value::Null);
}

#[tokio::test]
async fn login_verify_rejects_unknown_code() {
    let app = test_app().await;

    let response = app
        .router()
        .oneshot(
            Request::post("/api/v1/auth/login/verify-code")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"user@example.com","code":"123456"}"#,
                ))
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
        .router()
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
        .router()
        .oneshot(Request::get("/api/v1/auth/me").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

async fn admin_cookie(app: &TestApp) -> (String, String) {
    let request_response = app
        .router()
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
        .router()
        .oneshot(
            Request::post("/api/v1/auth/login/verify-code")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"admin@minirust.local","code":"123456"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let cookie = response
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.split(';').next().unwrap_or_default().to_owned())
        .expect("admin login must set a session cookie");

    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let user_id = body["data"]["user"]["id"].as_str().unwrap().to_owned();

    (cookie, user_id)
}

#[tokio::test]
async fn authenticated_user_can_update_profile() {
    let app = test_app().await;
    let (cookie, _admin_id) = admin_cookie(&app).await;

    let response = app.router()
        .oneshot(
            Request::patch("/api/v1/users/me")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(r#"{"full_name":"MiniRust Admin","avatar_url":"https://example.com/avatar.png"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let me = app
        .router()
        .oneshot(
            Request::get("/api/v1/auth/me")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(me.status(), StatusCode::OK);
    let body = axum::body::to_bytes(me.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body = String::from_utf8(body.to_vec()).unwrap();
    assert!(body.contains("MiniRust Admin"));
    assert!(body.contains("https://example.com/avatar.png"));
}

#[tokio::test]
async fn self_service_account_actions_require_authentication() {
    let app = test_app().await;

    let lock = app
        .router()
        .oneshot(
            Request::post("/api/v1/users/me/lock")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(lock.status(), StatusCode::UNAUTHORIZED);

    let delete = app
        .router()
        .oneshot(
            Request::delete("/api/v1/users/me")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn protected_bootstrap_admin_cannot_lock_or_delete_self() {
    let app = test_app().await;
    let (cookie, _admin_id) = admin_cookie(&app).await;

    let lock = app
        .router()
        .oneshot(
            Request::post("/api/v1/users/me/lock")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(lock.status(), StatusCode::CONFLICT);

    let delete = app
        .router()
        .oneshot(
            Request::delete("/api/v1/users/me")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn admin_user_crud_and_role_assignment() {
    let app = test_app().await;
    let (cookie, admin_id) = admin_cookie(&app).await;

    let list = app
        .router()
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
        .router()
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

    let body = axum::body::to_bytes(create.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let user_id = body["data"]["id"].as_str().unwrap().to_owned();

    let get = app
        .router()
        .oneshot(
            Request::get(format!("/api/v1/admin/users/{user_id}"))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get.status(), StatusCode::OK);

    let update = app
        .router()
        .oneshot(
            Request::patch(format!("/api/v1/admin/users/{user_id}"))
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(r#"{"email":"updated@example.com"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update.status(), StatusCode::OK);

    let assign = app
        .router()
        .oneshot(
            Request::put(format!("/api/v1/admin/users/{user_id}/role"))
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(r#"{"role":"admin"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(assign.status(), StatusCode::OK);

    let premium_get = app
        .router()
        .oneshot(
            Request::get(format!(
                "/api/v1/admin/users/{user_id}/entitlements/premium"
            ))
            .header("cookie", &cookie)
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(premium_get.status(), StatusCode::OK);

    let premium = app
        .router()
        .oneshot(
            Request::put(format!(
                "/api/v1/admin/users/{user_id}/entitlements/premium"
            ))
            .header("content-type", "application/json")
            .header("cookie", &cookie)
            .body(Body::from(r#"{"active":true,"expires_at":null}"#))
            .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(premium.status(), StatusCode::OK);

    let remove_premium = app
        .router()
        .oneshot(
            Request::put(format!(
                "/api/v1/admin/users/{user_id}/entitlements/premium"
            ))
            .header("content-type", "application/json")
            .header("cookie", &cookie)
            .body(Body::from(r#"{"active":false,"expires_at":null}"#))
            .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(remove_premium.status(), StatusCode::OK);

    let invalid_expiry = app
        .router()
        .oneshot(
            Request::put(format!(
                "/api/v1/admin/users/{user_id}/entitlements/premium"
            ))
            .header("content-type", "application/json")
            .header("cookie", &cookie)
            .body(Body::from(r#"{"active":true,"expires_at":1}"#))
            .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(invalid_expiry.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let premium_delete = app
        .router()
        .oneshot(
            Request::delete(format!(
                "/api/v1/admin/users/{user_id}/entitlements/premium"
            ))
            .header("cookie", &cookie)
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(premium_delete.status(), StatusCode::OK);

    let remove_role = app
        .router()
        .oneshot(
            Request::put(format!("/api/v1/admin/users/{user_id}/role"))
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(r#"{"role":"none"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(remove_role.status(), StatusCode::OK);

    let protected_delete = app
        .router()
        .oneshot(
            Request::delete(format!("/api/v1/admin/users/{admin_id}"))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(protected_delete.status(), StatusCode::CONFLICT);

    // The currently authenticated admin cannot delete their own account through the admin endpoint.
    let self_delete = app
        .router()
        .oneshot(
            Request::delete(format!("/api/v1/admin/users/{admin_id}"))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(self_delete.status(), StatusCode::CONFLICT);

    let delete = app
        .router()
        .oneshot(
            Request::delete(format!("/api/v1/admin/users/{user_id}"))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete.status(), StatusCode::NO_CONTENT);

    let missing = app
        .router()
        .oneshot(
            Request::get(format!("/api/v1/admin/users/{user_id}"))
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
        Request::get("/api/v1/admin/users")
            .body(Body::empty())
            .unwrap(),
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
        Request::put("/api/v1/users/me/language")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"locale":"en"}"#))
            .unwrap(),
        Request::put("/api/v1/admin/users/user@example.com/entitlements/premium")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"active":true,"expires_at":null}"#))
            .unwrap(),
    ];

    for request in requests {
        let response = app.router().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
#[tokio::test]
async fn authenticated_user_can_update_and_read_locale() {
    let app = test_app().await;
    let (cookie, _) = admin_cookie(&app).await;

    let update = app
        .router()
        .oneshot(
            Request::put("/api/v1/users/me/language")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(r#"{"locale":"en"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update.status(), StatusCode::OK);

    let body = axum::body::to_bytes(update.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["data"]["locale"], "en");

    let me = app
        .router()
        .oneshot(
            Request::get("/api/v1/auth/me")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(me.status(), StatusCode::OK);

    let body = axum::body::to_bytes(me.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["data"]["locale"], "en");
}

#[tokio::test]
async fn locale_update_rejects_unsupported_language() {
    let app = test_app().await;
    let (cookie, _) = admin_cookie(&app).await;

    let response = app
        .router()
        .oneshot(
            Request::put("/api/v1/users/me/language")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(r#"{"locale":"fr"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn auth_code_requests_are_rate_limited() {
    let app = test_app().await;

    for _ in 0..3 {
        let response = app
            .router()
            .oneshot(
                Request::post("/api/v1/auth/login/request-code")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"email":"admin@minirust.local"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    let response = app
        .router()
        .oneshot(
            Request::post("/api/v1/auth/login/request-code")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"email":"admin@minirust.local"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn menu_user_endpoint_requires_authentication() {
    let app = test_app().await;

    let response = app
        .router()
        .oneshot(Request::get("/api/v1/menus").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn admin_menu_access_update_contract() {
    let app = test_app().await;
    let (cookie, _) = admin_cookie(&app).await;

    let list = app
        .router()
        .oneshot(
            Request::get("/api/v1/admin/menus")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list.status(), StatusCode::OK);

    let body = axum::body::to_bytes(list.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let dashboard = body["data"]["menus"]
        .as_array()
        .unwrap()
        .iter()
        .find(|menu| menu["path"].as_str() == Some("/app"))
        .expect("the seeded dashboard menu must be listed");
    let menu_id = dashboard["id"].as_str().unwrap().to_owned();
    assert_eq!(dashboard["allow_user"], serde_json::Value::Bool(true));
    assert_eq!(dashboard["allow_premium"], serde_json::Value::Bool(true));

    // Restricting the menu to premium accounts keeps the registry entry intact.
    let update = app
        .router()
        .oneshot(
            Request::patch(format!("/api/v1/admin/menus/{menu_id}"))
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(
                    r#"{"parent_id":null,"name":"Dashboard","path":"/app","icon":"layout-dashboard","allow_user":false,"allow_premium":true,"sort_order":10,"is_active":true}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update.status(), StatusCode::OK);

    let body = axum::body::to_bytes(update.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["data"]["allow_user"], serde_json::Value::Bool(false));
    assert_eq!(body["data"]["allow_premium"], serde_json::Value::Bool(true));

    // An admin always has full access, so the menu stays visible to the session.
    let visible = app
        .router()
        .oneshot(
            Request::get("/api/v1/menus")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(visible.status(), StatusCode::OK);
    let body = axum::body::to_bytes(visible.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(body["data"]["menus"]
        .as_array()
        .unwrap()
        .iter()
        .any(|menu| menu["id"].as_str() == Some(menu_id.as_str())));

    let invalid_path = app
        .router()
        .oneshot(
            Request::patch(format!("/api/v1/admin/menus/{menu_id}"))
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(
                    r#"{"parent_id":null,"name":"Dashboard","path":"app","icon":null,"allow_user":true,"allow_premium":true,"sort_order":10,"is_active":true}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(invalid_path.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let missing = app
        .router()
        .oneshot(
            Request::patch("/api/v1/admin/menus/00000000-0000-7000-8000-000000000001")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(
                    r#"{"parent_id":null,"name":"Missing","path":"/missing","icon":null,"allow_user":true,"allow_premium":false,"sort_order":1,"is_active":true}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn admin_menu_endpoints_require_admin_authentication() {
    let app = test_app().await;

    let requests = [
        Request::get("/api/v1/admin/menus")
            .body(Body::empty())
            .unwrap(),
        Request::patch("/api/v1/admin/menus/00000000-0000-7000-8000-000000000001")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"parent_id":null,"name":"Unauthorized","path":"/unauthorized","icon":null,"allow_user":true,"allow_premium":false,"sort_order":1,"is_active":true}"#,
            ))
            .unwrap(),
    ];

    for request in requests {
        let response = app.router().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
