use minirust_core::{Greeting, APP_NAME};

use crate::cqrs::{Query, QueryHandler};

/// Query for the current application greeting.
#[derive(Debug, Clone, Copy, Default)]
pub struct GreetingQuery;

impl Query for GreetingQuery {
    type Output = Greeting;
}

/// Handles greeting queries without mutating application state.
#[derive(Debug, Clone, Copy, Default)]
pub struct GreetingQueryHandler;

impl QueryHandler<GreetingQuery> for GreetingQueryHandler {
    fn handle(&self, _query: GreetingQuery) -> Greeting {
        Greeting::new(format!("Hello from {APP_NAME}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handler_returns_the_application_greeting() {
        let result = GreetingQueryHandler.handle(GreetingQuery);
        assert_eq!(result.message, "Hello from MiniRust");
    }
}
