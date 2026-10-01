//! Application layer for MiniRust.
//!
//! The crate name is retained for workspace compatibility, but its internal
//! structure is CQRS-oriented. Commands change state; queries only read state.

pub mod auth;
pub mod commands;
pub mod cqrs;
pub mod menu;
pub mod queries;
pub mod query;
pub mod user_admin;

pub use commands::echo::{EchoCommand, EchoCommandHandler, EchoCommandResult};
pub use queries::greeting::{GreetingQuery, GreetingQueryHandler};
pub use query::{Page, Pagination, PaginationError, PaginationMeta, PaginationRequest};

pub use auth::{
    AuthCommand, AuthCommandHandler, AuthCommandResult, AuthError, AuthQueryHandler,
    AuthRepository, AuthService, Challenge, ChallengePurpose, ChallengeRef, CodeRequestAccepted,
    ConfiguredEmailSender, CurrentSessionQuery, EmailSender, RequireAdminQuery, Session,
    UnavailableEmailSender, UserAccess, UserLocale,
};

pub use menu::{
    Menu, MenuAccess, MenuAccessDecision, MenuCommand, MenuCommandHandler, MenuError, MenuQuery,
    MenuQueryHandler, MenuRepository, MenuService, UpdateMenu,
};

pub use user_admin::{
    AdminUserRole, PremiumEntitlement, UserAdminCommand, UserAdminCommandHandler,
    UserAdminCommandResult, UserAdminError, UserAdminQuery, UserAdminQueryHandler,
    UserAdminQueryResult, UserAdminRepository, UserAdminService,
};
