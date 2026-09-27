use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    MessageRequired,
    MessageTooLong { max: usize },
}

impl ValidationError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::MessageRequired => "MESSAGE_REQUIRED",
            Self::MessageTooLong { .. } => "MESSAGE_TOO_LONG",
        }
    }

    pub const fn message_key(&self) -> &'static str {
        match self {
            Self::MessageRequired => "errors.validation.message_required",
            Self::MessageTooLong { .. } => "errors.validation.message_too_long",
        }
    }
}

#[derive(Debug)]
pub enum AppError {
    Config(String),
    Io(std::io::Error),
    Validation(ValidationError),
}

impl AppError {
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    pub const fn validation(error: ValidationError) -> Self {
        Self::Validation(error)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(message) => formatter.write_str(message),
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Validation(error) => formatter.write_str(error.code()),
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
