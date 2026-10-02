use std::path::PathBuf;
use std::sync::Once;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use minirust_api::{router, AppState, PublicSmtpEmailSender};
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

pub struct TestApp {
    router: Router,
}

impl TestApp {
    pub fn router(&self) -> Router {
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

pub fn temp_upload_directory() -> PathBuf {
    std::env::temp_dir().join(format!("minirust-uploads-{}", Uuid::now_v7().simple()))
}

pub async fn test_app() -> TestApp {
    test_app_with(temp_upload_directory()).await
}

pub async fn test_app_with(upload_directory: impl Into<PathBuf>) -> TestApp {
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

    let state = AppState::with_email_sender(
        database,
        vec![b'a'; 32],
        false,
        PublicSmtpEmailSender::local_for_tests("http://localhost:3001".to_owned()),
        upload_directory,
    )
    .expect("test authentication secret must be valid");

    TestApp {
        router: router(state),
    }
}

pub async fn admin_cookie(app: &TestApp) -> (String, String) {
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
