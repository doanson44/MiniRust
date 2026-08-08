use axum::{routing::get, Router};
use std::{env, error::Error, fmt, net::SocketAddr};

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 3000;
const HOST_ENV: &str = "MINIRUST_HOST";
const PORT_ENV: &str = "MINIRUST_PORT";

#[derive(Debug, Clone, PartialEq, Eq)]
struct AppConfig {
    host: String,
    port: u16,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            host: DEFAULT_HOST.to_owned(),
            port: DEFAULT_PORT,
        }
    }
}

impl AppConfig {
    fn from_env() -> Result<Self, ConfigError> {
        let defaults = Self::default();
        let host = env::var(HOST_ENV).unwrap_or(defaults.host);
        let port = match env::var(PORT_ENV) {
            Ok(value) => value.parse().map_err(ConfigError::InvalidPort)?,
            Err(_) => defaults.port,
        };

        Ok(Self { host, port })
    }

    fn socket_addr(&self) -> Result<SocketAddr, std::io::Error> {
        format!("{}:{}", self.host, self.port)
            .parse()
            .map_err(|error| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("invalid server address: {error}"),
                )
            })
    }
}

#[derive(Debug)]
enum ConfigError {
    InvalidPort(std::num::ParseIntError),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPort(error) => {
                write!(formatter, "invalid {PORT_ENV}: {error}")
            }
        }
    }
}

impl Error for ConfigError {}

fn app() -> Router {
    Router::new().route("/", get(root))
}

async fn root() -> &'static str {
    "MiniRust"
}

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        eprintln!("failed to listen for shutdown signal: {error}");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = AppConfig::from_env()?;
    let addr = config.socket_addr()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("listening on http://{addr}");

    axum::serve(listener, app())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::sync::Mutex;
    use tower::ServiceExt;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_env<F>(values: &[(&str, Option<&str>)], test: F)
    where
        F: FnOnce(),
    {
        let _guard = ENV_LOCK.lock().unwrap();
        let previous = values
            .iter()
            .map(|(name, _)| (*name, std::env::var(name).ok()))
            .collect::<Vec<_>>();

        for (name, value) in values {
            match value {
                Some(value) => std::env::set_var(name, value),
                None => std::env::remove_var(name),
            }
        }

        test();

        for (name, value) in previous {
            match value {
                Some(value) => std::env::set_var(name, value),
                None => std::env::remove_var(name),
            }
        }
    }

    #[test]
    fn default_config_is_used_when_environment_is_missing() {
        with_env(&[(HOST_ENV, None), (PORT_ENV, None)], || {
            assert_eq!(AppConfig::from_env().unwrap(), AppConfig::default());
        });
    }

    #[test]
    fn environment_config_overrides_defaults() {
        with_env(
            &[(HOST_ENV, Some("0.0.0.0")), (PORT_ENV, Some("8080"))],
            || {
                let config = AppConfig::from_env().unwrap();
                assert_eq!(config.host, "0.0.0.0");
                assert_eq!(config.port, 8080);
            },
        );
    }

    #[test]
    fn invalid_port_returns_an_error() {
        with_env(&[(HOST_ENV, None), (PORT_ENV, Some("invalid"))], || {
            let error = AppConfig::from_env().unwrap_err();
            assert!(error.to_string().contains(PORT_ENV));
        });
    }

    #[tokio::test]
    async fn get_root_returns_ok_with_body() {
        let response = app()
            .oneshot(Request::get("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body, "MiniRust");
    }
}
