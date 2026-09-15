//! Application layer for MiniRust.
//!
//! The crate name is retained for workspace compatibility, but its internal
//! structure is CQRS-oriented. Commands change state; queries only read state.

pub mod commands;
pub mod queries;

pub use commands::echo::{EchoCommand, EchoCommandHandler, EchoCommandResult};
pub use queries::greeting::{GreetingQuery, GreetingQueryHandler};
