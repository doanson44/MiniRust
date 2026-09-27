//! Shared CQRS contracts for the application layer.

/// Marker trait for a state-changing application request.
pub trait Command {
    type Output;
    type Error;
}

/// Handles one command type.
pub trait CommandHandler<C: Command> {
    fn handle(&self, command: C) -> Result<C::Output, C::Error>;
}

/// Marker trait for a read-only application request.
pub trait Query {
    type Output;
}

/// Handles one query type without mutating application state.
pub trait QueryHandler<Q: Query> {
    fn handle(&self, query: Q) -> Q::Output;
}

/// Handles an asynchronous command at the application boundary.
pub trait AsyncCommandHandler<C: Command> {
    async fn handle(&self, command: C) -> Result<C::Output, C::Error>;
}

/// Handles an asynchronous read-side query at the application boundary.
pub trait AsyncQueryHandler<Q: Query> {
    async fn handle(&self, query: Q) -> Q::Output;
}
