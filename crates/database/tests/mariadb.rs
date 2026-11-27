use minirust_core::EntityId;
use minirust_database::Database;
use minirust_services::{NewUserAccess, UserAdminError, UserAdminRepository};
use sqlx::{Connection, Executor, MySqlConnection};
use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    GenericImage, ImageExt,
};

#[tokio::test]
async fn database_connects_to_mariadb_running_in_docker() -> Result<(), Box<dyn std::error::Error>>
{
    let container = GenericImage::new("mariadb", "11")
        .with_wait_for(WaitFor::message_on_stderr("ready for connections"))
        .with_exposed_port(3306.tcp())
        .with_env_var("MARIADB_DATABASE", "minirust_test")
        .with_env_var("MARIADB_USER", "minirust")
        .with_env_var("MARIADB_PASSWORD", "minirust")
        .with_env_var("MARIADB_ROOT_PASSWORD", "root")
        .start()
        .await?;

    let host = container.get_host().await?;
    let port = container.get_host_port_ipv4(3306).await?;
    let url = format!("mysql://minirust:minirust@{host}:{port}/minirust_test");

    let mut retries = 5;
    let database = loop {
        match Database::connect(&url).await {
            Ok(db) => break db,
            Err(_e) if retries > 0 => {
                retries -= 1;
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
            Err(e) => return Err(e.into()),
        }
    };

    database.health().await?;
    database.migrate().await?;

    let root_url = format!("mysql://root:root@{host}:{port}/minirust_test");
    let mut root = MySqlConnection::connect(&root_url).await?;
    root.execute(
        "CREATE TRIGGER fail_user_entitlement
         BEFORE INSERT ON user_entitlements
         FOR EACH ROW
         SIGNAL SQLSTATE '45000'
         SET MESSAGE_TEXT = 'forced integration-test failure'",
    )
    .await?;

    let result = database
        .create_user_with_access(NewUserAccess {
            id: EntityId::new(),
            email: "rollback@example.com".to_owned(),
            role: minirust_services::AdminUserRole::Admin,
            premium_active: true,
            premium_expires_at: None,
            is_locked: false,
            created_at: 1_800_000_000,
        })
        .await;

    assert!(matches!(result, Err(UserAdminError::Persistence)));
    assert!(database.find_user("rollback@example.com").await?.is_none());

    Ok(())
}
