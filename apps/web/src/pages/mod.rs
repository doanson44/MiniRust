mod admin;
mod app;
mod auth;
mod layouts;
mod menus;
mod profile;

pub use admin::AdminPage;
pub use app::AppPage;
pub use auth::{LoginPage, RegisterPage, RegisterVerifyPage};
pub use layouts::{AppLayout, AuthLayout};
pub use menus::MenuAdminPage;
pub use profile::ProfilePage;
