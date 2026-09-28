use minirust_database::Database;
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
            Err(e) if retries > 0 => {
                retries -= 1;
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
            Err(e) => return Err(e.into()),
        }
    };
    
    database.health().await?;

    Ok(())
}
