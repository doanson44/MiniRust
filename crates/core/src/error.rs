use std::fmt;

/// Internal application error.
///
/// HTTP and UI layers map this type to user-facing responses. Do not expose
/// internal details such as database messages or secrets to clients.
///
/// - `Config`     — startup/configuration error; never reaches normal request handling.
/// - `Io`         — OS-level I/O failure; log the detail, return a generic 500.
/// - `Validation` — invalid user input; safe to return the message to the caller as-is.
#[derive(Debug)]
pub enum AppError {
    Config(String),
    Io(std::io::Error),
    Validation(String),
}

impl AppError {
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    /// Create a validation error whose message is safe to return to API callers.
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(message) => formatter.write_str(message),
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Validation(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Config(_) | Self::Validation(_) => None,
            Self::Io(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
