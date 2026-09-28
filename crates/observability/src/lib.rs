use std::fs;
use std::io;
use std::path::Path;

use minirust_config::Environment;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::EnvFilter;

#[derive(Debug)]
pub enum LoggingError {
    CreateLogDirectory(io::Error),
    SetGlobalDefault(tracing::subscriber::SetGlobalDefaultError),
}

impl std::fmt::Display for LoggingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CreateLogDirectory(error) => {
                write!(formatter, "failed to create log directory: {error}")
            }
            Self::SetGlobalDefault(error) => {
                write!(formatter, "failed to initialize global logger: {error}")
            }
        }
    }
}

impl std::error::Error for LoggingError {}

pub struct LoggingGuard(Option<WorkerGuard>);

impl LoggingGuard {
    pub fn new(guard: WorkerGuard) -> Self {
        Self(Some(guard))
    }
}

impl Drop for LoggingGuard {
    fn drop(&mut self) {
        let _ = self.0.take();
    }
}

pub fn init(
    environment: Environment,
    log_filter: &str,
    log_directory: impl AsRef<Path>,
    service_name: &str,
) -> Result<LoggingGuard, LoggingError> {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_filter));

    match environment {
        Environment::Development => {
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_target(true)
                .compact()
                .try_init()
                .map_err(LoggingError::SetGlobalDefault)?;

            let (non_blocking, guard) = tracing_appender::non_blocking(io::stdout());
            let _ = non_blocking;
            Ok(LoggingGuard::new(guard))
        }
        Environment::Production => {
            let directory = log_directory.as_ref();
            fs::create_dir_all(directory).map_err(LoggingError::CreateLogDirectory)?;

            let file_appender = rolling::daily(directory, format!("{service_name}.log"));
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_target(true)
                .with_ansi(false)
                .compact()
                .with_writer(non_blocking)
                .try_init()
                .map_err(LoggingError::SetGlobalDefault)?;

            Ok(LoggingGuard::new(guard))
        }
    }
}
