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
pub const ENV_AUTH_SECRET: &str = "MINIRUST_AUTH_SECRET";
pub const ENV_ADMIN_EMAIL: &str = "MINIRUST_ADMIN_EMAIL";
pub const ENV_ADMIN_OTP: &str = "MINIRUST_ADMIN_OTP";

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
    pub auth_secret: Option<String>,
    pub admin_email: String,
    pub admin_otp: String,
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
            database_url: read_optional(ENV_DATABASE_URL),
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
        })
    }

    pub fn database_url(&self) -> Result<&str, ConfigError> {
        self.database_url
            .as_deref()
            .ok_or_else(|| ConfigError::MissingRequired(ENV_DATABASE_URL.to_owned()))
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
