use minirust_database::Database;
use std::path::PathBuf;
use std::process::{Command, Output};

struct MariaDbTestEnvironment {
    project_name: String,
    port: u16,
}

impl MariaDbTestEnvironment {
    fn start() -> Result<Self, Box<dyn std::error::Error>> {
        let project_name = format!("minirust-test-{}", std::process::id());

        if let Err(error) = run_compose(&project_name, ["up", "-d", "--wait"].as_slice()) {
            let _ = cleanup_compose(&project_name);
            return Err(error);
        }

        let output = match run_compose_output(&project_name, ["port", "mariadb", "3306"].as_slice()) {
            Ok(output) if output.status.success() => output,
            Ok(output) => {
                let _ = cleanup_compose(&project_name);
                return Err(format!(
                    "docker compose port failed with status {}: {}",
                    output.status,
                    String::from_utf8_lossy(&output.stderr).trim()
                )
                .into());
            }
            Err(error) => {
                let _ = cleanup_compose(&project_name);
                return Err(error);
            }
        };

        let address = String::from_utf8(output.stdout)?;
        let port = match address
            .trim()
            .rsplit(':')
            .next()
            .and_then(|value| value.parse::<u16>().ok())
        {
            Some(port) => port,
            None => {
                let _ = cleanup_compose(&project_name);
                return Err("docker compose did not return a valid published port".into());
            }
        };

        Ok(Self { project_name, port })
    }

    fn database_url(&self) -> String {
        format!(
            "mysql://minirust:minirust@127.0.0.1:{}/minirust_test",
            self.port
        )
    }
}

impl Drop for MariaDbTestEnvironment {
    fn drop(&mut self) {
        let _ = cleanup_compose(&self.project_name);
    }
}

fn compose_file() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docker-compose.integration.yml")
}

fn run_compose(project_name: &str, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let output = run_compose_output(project_name, args)?;
    if output.status.success() {
        return Ok(());
    }

    Err(format!(
        "docker compose command failed with status {}: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr).trim()
    )
    .into())
}

fn run_compose_output(
    project_name: &str,
    args: &[&str],
) -> Result<Output, Box<dyn std::error::Error>> {
    Ok(Command::new("docker")
        .args(["compose", "-p", project_name, "-f"])
        .arg(compose_file())
        .args(args)
        .output()?)
}

fn cleanup_compose(project_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    run_compose(
        project_name,
        ["down", "--volumes", "--remove-orphans"].as_slice(),
    )
}

#[tokio::test]
async fn database_connects_to_mariadb_running_in_docker() -> Result<(), Box<dyn std::error::Error>> {
    let environment = MariaDbTestEnvironment::start()?;
    let database = Database::connect(&environment.database_url()).await?;

    database.health().await?;

    Ok(())
}
