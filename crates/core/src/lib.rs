//! Shared domain types and error values used by MiniRust applications.
//!
//! This crate must stay free of web frameworks and infrastructure SDKs.

mod error;

pub use error::AppError;

pub const APP_NAME: &str = "MiniRust";

/// Process health used by application services.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthStatus {
    pub status: &'static str,
}

impl HealthStatus {
    pub fn ok() -> Self {
        Self { status: "ok" }
    }
}

/// Simple greeting produced by application services.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Greeting {
    pub message: String,
}

impl Greeting {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}
