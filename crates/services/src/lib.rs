//! Application services.
//!
//! Handlers should call these types instead of embedding business rules.

use minirust_core::{Greeting, HealthStatus, APP_NAME};

/// Reports process liveness for health checks.
#[derive(Debug, Clone, Copy, Default)]
pub struct HealthService;

impl HealthService {
    pub fn status(&self) -> HealthStatus {
        HealthStatus::ok()
    }
}

/// Builds the baseline greeting used by the API and the SSR page.
#[derive(Debug, Clone, Copy, Default)]
pub struct GreetingService;

impl GreetingService {
    pub fn hello(&self) -> Greeting {
        Greeting::new(format!("Hello from {APP_NAME}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_service_reports_ok() {
        let status = HealthService.status();
        assert_eq!(status.status, "ok");
    }

    #[test]
    fn greeting_service_uses_the_application_name() {
        let greeting = GreetingService.hello();
        assert_eq!(greeting.message, "Hello from MiniRust");
    }
}
