//! Application layer for MiniRust.
//!
//! The crate name is retained for workspace compatibility, but its internal
//! structure is CQRS-oriented. Commands change state; queries only read state.

pub mod auth;
pub mod user_admin;
pub mod commands;
pub mod cqrs;
pub mod queries;

pub use commands::echo::{EchoCommand, EchoCommandHandler, EchoCommandResult};
pub use queries::greeting::{GreetingQuery, GreetingQueryHandler};

pub use auth::{AuthError, AuthService, ChallengePurpose, CodeRequestAccepted, EmailSender, Session, UnavailableEmailSender, UserAccess};

pub use user_admin::{AdminUserRole, PremiumEntitlement, UserAdminError, UserAdminRepository, UserAdminService};
