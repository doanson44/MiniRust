//! Shared domain types and error values used by MiniRust applications.
//!
//! This crate must stay free of web frameworks and infrastructure SDKs.

mod error;

pub use error::AppError;

pub const APP_NAME: &str = "MiniRust";

/// Maximum allowed length of an echo message in characters.
pub const ECHO_MAX_CHARS: usize = 500;

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

/// Validated echo input.
///
/// Can only be constructed through [`EchoInput::parse`], which enforces:
/// - the message is non-empty after trimming
/// - the message does not exceed [`ECHO_MAX_CHARS`] characters
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EchoInput {
    pub message: String,
}

impl EchoInput {
    /// Trim, validate, and wrap a raw message string.
    ///
    /// Returns [`AppError::Validation`] when the message is empty or too long.
    pub fn parse(raw: String) -> Result<Self, AppError> {
        let message = raw.trim().to_owned();

        if message.is_empty() {
            return Err(AppError::validation("message must not be empty"));
        }

        if message.chars().count() > ECHO_MAX_CHARS {
            return Err(AppError::validation(format!(
                "message must not exceed {ECHO_MAX_CHARS} characters"
            )));
        }

        Ok(Self { message })
    }
}

/// Successful echo result produced by [`EchoInput`] processing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Echo {
    pub echo: String,
}

impl Echo {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            echo: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn echo_input_accepts_a_valid_message() {
        let input = EchoInput::parse("Hello World".to_owned()).unwrap();
        assert_eq!(input.message, "Hello World");
    }

    #[test]
    fn echo_input_trims_surrounding_whitespace() {
        let input = EchoInput::parse("  hello  ".to_owned()).unwrap();
        assert_eq!(input.message, "hello");
    }

    #[test]
    fn echo_input_rejects_empty_message() {
        let err = EchoInput::parse(String::new()).unwrap_err();
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn echo_input_rejects_whitespace_only_message() {
        let err = EchoInput::parse("   ".to_owned()).unwrap_err();
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn echo_input_rejects_message_exceeding_max_length() {
        let long = "a".repeat(ECHO_MAX_CHARS + 1);
        let err = EchoInput::parse(long).unwrap_err();
        assert!(err.to_string().contains("500"));
    }

    #[test]
    fn echo_input_accepts_message_at_exact_max_length() {
        let at_limit = "a".repeat(ECHO_MAX_CHARS);
        let input = EchoInput::parse(at_limit.clone()).unwrap();
        assert_eq!(input.message, at_limit);
    }
}
