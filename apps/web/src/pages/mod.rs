mod admin;
mod app;
mod auth;
mod home;
mod layouts;

pub use admin::AdminPage;
pub use app::{AppPage, AppPage as ProfilePage};
pub use auth::{LoginPage, RegisterPage};
pub use home::HomePage;
pub use layouts::{AppLayout, AuthLayout};
