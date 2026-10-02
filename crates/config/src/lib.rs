//! Application configuration loaded from the process environment.
//!
//! A .env file is loaded when present. Missing .env is not an error.
//! The database URL targets MariaDB through SQLx's mysql driver.

use std::env;
use std::fmt;
use std::net::AddrParseError;
use std::net::SocketAddr;
use std::num::ParseIntError;

use minirust_core::AppError;

pub const DEFAULT_LOG_FILTER: &str = "info";
pub const DEFAULT_LOG_DIRECTORY: &str = "logs";
pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const DEFAULT_API_PORT: u16 = 3000;
pub const DEFAULT_WEB_PORT: u16 = 3001;
pub const DEFAULT_DB_HOST: &str = "127.0.0.1";
pub const DEFAULT_DB_PORT: u16 = 3306;
pub const DEFAULT_DB_NAME: &str = "minirust";
pub const DEFAULT_DB_USER: &str = "minirust";
pub const DEFAULT_DB_PASSWORD: &str = "minirust";
pub const DEFAULT_DB_MAX_CONNECTIONS: u32 = 10;

pub const ENV_ENVIRONMENT: &str = "MINIRUST_ENV";
pub const ENV_LOG: &str = "MINIRUST_LOG";
pub const ENV_LOG_DIRECTORY: &str = "MINIRUST_LOG_DIR";
pub const ENV_API_HOST: &str = "MINIRUST_API_HOST";
pub const ENV_API_PORT: &str = "MINIRUST_API_PORT";
pub const ENV_WEB_HOST: &str = "MINIRUST_WEB_HOST";
pub const ENV_WEB_PORT: &str = "MINIRUST_WEB_PORT";
pub const ENV_DATABASE_URL: &str = "MINIRUST_DATABASE_URL";
pub const ENV_DB_HOST: &str = "MINIRUST_DB_HOST";
pub const ENV_DB_PORT: &str = "MINIRUST_DB_PORT";
pub const ENV_DB_NAME: &str = "MINIRUST_DB_NAME";
pub const ENV_DB_USER: &str = "MINIRUST_DB_USER";
pub const ENV_DB_PASSWORD: &str = "MINIRUST_DB_PASSWORD";
pub const ENV_DB_MAX_CONNECTIONS: &str = "MINIRUST_DB_MAX_CONNECTIONS";
pub const ENV_AUTH_SECRET: &str = "MINIRUST_AUTH_SECRET";
pub const ENV_ADMIN_EMAIL: &str = "MINIRUST_ADMIN_EMAIL";
pub const ENV_ADMIN_OTP: &str = "MINIRUST_ADMIN_OTP";
pub const ENV_SMTP_HOST: &str = "MINIRUST_SMTP_HOST";
pub const ENV_SMTP_PORT: &str = "MINIRUST_SMTP_PORT";
pub const ENV_SMTP_USERNAME: &str = "MINIRUST_SMTP_USERNAME";
pub const ENV_SMTP_PASSWORD: &str = "MINIRUST_SMTP_PASSWORD";
pub const ENV_SMTP_FROM_EMAIL: &str = "MINIRUST_SMTP_FROM_EMAIL";
pub const ENV_SMTP_FROM_NAME: &str = "MINIRUST_SMTP_FROM_NAME";
pub const DEFAULT_SMTP_HOST: &str = "smtp.gmail.com";
pub const DEFAULT_SMTP_PORT: u16 = 587;

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

/// MariaDB connection settings assembled from the `MINIRUST_DB_*` variables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    pub password: String,
    pub max_connections: u32,
}

impl DatabaseConfig {
    /// Builds the SQLx MySQL URL. Credentials are percent-encoded because SQLx
    /// percent-decodes them, so passwords with reserved characters survive.
    pub fn url(&self) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            encode_url_component(&self.user),
            encode_url_component(&self.password),
            self.host,
            self.port,
            encode_url_component(&self.name),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_email: String,
    pub from_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub environment: Environment,
    pub log_filter: String,
    pub log_directory: String,
    pub database: DatabaseConfig,
    pub auth_secret: Option<String>,
    pub admin_email: String,
    pub admin_otp: String,
    pub smtp: Option<SmtpConfig>,
    database_url_override: Option<String>,
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
        let environment = Environment::from_env();

        Ok(Self {
            environment,
            log_filter: read_or_default(ENV_LOG, DEFAULT_LOG_FILTER),
            log_directory: read_or_default(ENV_LOG_DIRECTORY, DEFAULT_LOG_DIRECTORY),
            database: read_database(environment)?,
            database_url_override: read_optional(ENV_DATABASE_URL),
            auth_secret: read_optional(ENV_AUTH_SECRET),
            admin_email: match environment {
                Environment::Development => {
                    read_or_default(ENV_ADMIN_EMAIL, "admin@minirust.local")
                }
                Environment::Production => require_var(ENV_ADMIN_EMAIL)?,
            },
            admin_otp: match environment {
                Environment::Development => read_or_default(ENV_ADMIN_OTP, "123456"),
                Environment::Production => require_var(ENV_ADMIN_OTP)?,
            },
            smtp: read_smtp()?,
        })
    }

    /// Connection URL. `MINIRUST_DATABASE_URL`, when set, overrides the
    /// `MINIRUST_DB_*` parts entirely.
    pub fn database_url(&self) -> String {
        match self.database_url_override.as_deref() {
            Some(url) => url.to_owned(),
            None => self.database.url(),
        }
    }

    pub fn database_max_connections(&self) -> u32 {
        self.database.max_connections
    }

    pub fn admin_email(&self) -> &str {
        &self.admin_email
    }

    pub fn admin_otp(&self) -> &str {
        &self.admin_otp
    }

    pub fn auth_secret(&self) -> Result<&str, ConfigError> {
        self.auth_secret
            .as_deref()
            .ok_or_else(|| ConfigError::MissingRequired(ENV_AUTH_SECRET.to_owned()))
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
    InvalidMaxConnections {
        name: String,
        value: String,
    },
    IncompleteSmtpConfig(String),
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
            Self::IncompleteSmtpConfig(name) => write!(
                formatter,
                "incomplete SMTP configuration: {name} is required"
            ),
            Self::InvalidMaxConnections { name, value } => {
                write!(
                    formatter,
                    "invalid {name}: expected a positive integer, got {value}"
                )
            }
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidPort { source, .. } => Some(source),
            Self::InvalidAddress { source, .. } => Some(source),
            Self::Dotenv(_)
            | Self::MissingRequired(_)
            | Self::InvalidMaxConnections { .. }
            | Self::IncompleteSmtpConfig(_) => None,
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

fn read_database(environment: Environment) -> Result<DatabaseConfig, ConfigError> {
    let (host, name, user, password) = match environment {
        Environment::Development => (
            read_or_default(ENV_DB_HOST, DEFAULT_DB_HOST),
            read_or_default(ENV_DB_NAME, DEFAULT_DB_NAME),
            read_or_default(ENV_DB_USER, DEFAULT_DB_USER),
            read_or_default(ENV_DB_PASSWORD, DEFAULT_DB_PASSWORD),
        ),
        Environment::Production => (
            require_var(ENV_DB_HOST)?,
            require_var(ENV_DB_NAME)?,
            require_var(ENV_DB_USER)?,
            require_var(ENV_DB_PASSWORD)?,
        ),
    };

    let port = match env::var(ENV_DB_PORT) {
        Ok(value) if !value.is_empty() => parse_port(ENV_DB_PORT, &value)?,
        _ => DEFAULT_DB_PORT,
    };

    Ok(DatabaseConfig {
        host,
        port,
        name,
        user,
        password,
        max_connections: read_max_connections()?,
    })
}

fn read_smtp() -> Result<Option<SmtpConfig>, ConfigError> {
    let username = read_optional(ENV_SMTP_USERNAME);
    let password = read_optional(ENV_SMTP_PASSWORD);
    let from_email = read_optional(ENV_SMTP_FROM_EMAIL);
    if username.is_none() && password.is_none() && from_email.is_none() {
        return Ok(None);
    }
    let username =
        username.ok_or_else(|| ConfigError::IncompleteSmtpConfig(ENV_SMTP_USERNAME.to_owned()))?;
    let password =
        password.ok_or_else(|| ConfigError::IncompleteSmtpConfig(ENV_SMTP_PASSWORD.to_owned()))?;
    let from_email = from_email
        .ok_or_else(|| ConfigError::IncompleteSmtpConfig(ENV_SMTP_FROM_EMAIL.to_owned()))?;
    let host = read_or_default(ENV_SMTP_HOST, DEFAULT_SMTP_HOST);
    let port = match env::var(ENV_SMTP_PORT) {
        Ok(value) if !value.is_empty() => parse_port(ENV_SMTP_PORT, &value)?,
        _ => DEFAULT_SMTP_PORT,
    };
    Ok(Some(SmtpConfig {
        host,
        port,
        username,
        password,
        from_email,
        from_name: read_optional(ENV_SMTP_FROM_NAME),
    }))
}

fn read_max_connections() -> Result<u32, ConfigError> {
    match env::var(ENV_DB_MAX_CONNECTIONS) {
        Ok(value) if !value.is_empty() => match value.parse::<u32>() {
            Ok(parsed) if parsed > 0 => Ok(parsed),
            _ => Err(ConfigError::InvalidMaxConnections {
                name: ENV_DB_MAX_CONNECTIONS.to_owned(),
                value,
            }),
        },
        _ => Ok(DEFAULT_DB_MAX_CONNECTIONS),
    }
}

fn encode_url_component(value: &str) -> String {
    const HEX: [u8; 16] = *b"0123456789ABCDEF";
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(char::from(byte));
            }
            _ => {
                encoded.push('%');
                encoded.push(char::from(HEX[usize::from(byte >> 4)]));
                encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
            }
        }
    }
    encoded
}
