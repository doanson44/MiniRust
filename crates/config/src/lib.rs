//! Application configuration loaded from the process environment.
//!
//! A `.env` file is loaded when present. Missing `.env` is not an error.
//! The database URL targets MariaDB through SQLx's `mysql` driver.

use std::env;
use std::fmt;
use std::net::AddrParseError;
use std::net::SocketAddr;
use std::num::ParseIntError;

use minirust_core::AppError;

pub const DEFAULT_LOG_FILTER: &str = "info";
pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const DEFAULT_API_PORT: u16 = 3000;
pub const DEFAULT_WEB_PORT: u16 = 3001;

pub const ENV_ENVIRONMENT: &str = "MINIRUST_ENV";
pub const ENV_LOG: &str = "MINIRUST_LOG";
pub const ENV_API_HOST: &str = "MINIRUST_API_HOST";
pub const ENV_API_PORT: &str = "MINIRUST_API_PORT";
pub const ENV_WEB_HOST: &str = "MINIRUST_WEB_HOST";
pub const ENV_WEB_PORT: &str = "MINIRUST_WEB_PORT";
pub const ENV_DATABASE_URL: &str = "MINIRUST_DATABASE_URL";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Development,
    Production,
}

impl Environment {
    pub fn from_env() -> Self {
        match env::var(ENV_ENVIRONMENT) {
            Ok(value) if value.eq_ignore_ascii_case("production") => Self::Production,
            _ => Self::Development,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Production => "production",
            Self::Development => "development",
        }
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerBind {
    pub host: String,
    pub port: u16,
}

impl ServerBind {
    pub fn socket_addr(&self) -> Result<SocketAddr, ConfigError> {
        format!("{}:{}", self.host, self.port)
            .parse()
            .map_err(|source| ConfigError::InvalidAddress {
                host: self.host.clone(),
                port: self.port,
                source,
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerKind {
    Api,
    Web,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub environment: Environment,
    pub log_filter: String,
    pub database_url: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        match dotenvy::dotenv() {
            Ok(_) => {}
            Err(error) if error.not_found() => {}
            Err(error) => return Err(ConfigError::Dotenv(error.to_string())),
        }
        Self::from_env()
    }

    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            environment: Environment::from_env(),
            log_filter: read_or_default(ENV_LOG, DEFAULT_LOG_FILTER),
            database_url: read_optional(ENV_DATABASE_URL),
        })
    }

    pub fn database_url(&self) -> Result<&str, ConfigError> {
        self.database_url
            .as_deref()
            .ok_or_else(|| ConfigError::MissingRequired(ENV_DATABASE_URL.to_owned()))
    }

    pub fn server_bind(&self, kind: ServerKind) -> Result<ServerBind, ConfigError> {
        match kind {
            ServerKind::Api => read_server_bind(
                self.environment,
                ENV_API_HOST,
                ENV_API_PORT,
                DEFAULT_API_PORT,
            ),
            ServerKind::Web => read_server_bind(
                self.environment,
                ENV_WEB_HOST,
                ENV_WEB_PORT,
                DEFAULT_WEB_PORT,
            ),
        }
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Dotenv(String),
    MissingRequired(String),
    InvalidPort {
        name: String,
        source: ParseIntError,
    },
    InvalidAddress {
        host: String,
        port: u16,
        source: AddrParseError,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dotenv(message) => write!(formatter, "failed to load .env: {message}"),
            Self::MissingRequired(name) => {
                write!(formatter, "{name} is required when MINIRUST_ENV=production")
            }
            Self::InvalidPort { name, source } => write!(formatter, "invalid {name}: {source}"),
            Self::InvalidAddress { host, port, source } => {
                write!(formatter, "invalid server address {host}:{port}: {source}")
            }
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidPort { source, .. } => Some(source),
            Self::InvalidAddress { source, .. } => Some(source),
            Self::Dotenv(_) | Self::MissingRequired(_) => None,
        }
    }
}

impl From<ConfigError> for AppError {
    fn from(error: ConfigError) -> Self {
        AppError::config(error.to_string())
    }
}

fn read_or_default(name: &str, default: &str) -> String {
    match env::var(name) {
        Ok(value) if !value.is_empty() => value,
        _ => default.to_owned(),
    }
}

fn read_optional(name: &str) -> Option<String> {
    match env::var(name) {
        Ok(value) if !value.is_empty() => Some(value),
        _ => None,
    }
}

fn require_var(name: &str) -> Result<String, ConfigError> {
    match env::var(name) {
        Ok(value) if !value.is_empty() => Ok(value),
        _ => Err(ConfigError::MissingRequired(name.to_owned())),
    }
}

fn read_server_bind(
    environment: Environment,
    host_var: &str,
    port_var: &str,
    default_port: u16,
) -> Result<ServerBind, ConfigError> {
    match environment {
        Environment::Development => {
            let host = read_or_default(host_var, DEFAULT_HOST);
            let port = match env::var(port_var) {
                Ok(value) if !value.is_empty() => parse_port(port_var, &value)?,
                _ => default_port,
            };
            Ok(ServerBind { host, port })
        }
        Environment::Production => Ok(ServerBind {
            host: require_var(host_var)?,
            port: parse_port(port_var, &require_var(port_var)?)?,
        }),
    }
}

fn parse_port(name: &str, value: &str) -> Result<u16, ConfigError> {
    value.parse().map_err(|source| ConfigError::InvalidPort {
        name: name.to_owned(),
        source,
    })
}
