#[cfg(feature = "ssr")]
use leptos::config::{Env as LeptosEnv, LeptosOptions};
#[cfg(feature = "ssr")]
use minirust_config::{Config, ServerKind};
#[cfg(feature = "ssr")]
use minirust_observability::init as init_logging;
#[cfg(feature = "ssr")]
use minirust_web::{router, AppState};

#[cfg(feature = "ssr")]
async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        tracing::error!(%error, "failed to listen for shutdown signal");
    }
}

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let _logging_guard = init_logging(
        config.environment,
        &config.log_filter,
        &config.log_directory,
        "minirust-web",
    )?;

    let bind = config.server_bind(ServerKind::Web)?;
    let addr = bind.socket_addr()?;

    let leptos_environment = match config.environment {
        minirust_config::Environment::Production => LeptosEnv::PROD,
        minirust_config::Environment::Development => LeptosEnv::DEV,
    };
    let site_root = std::env::var("LEPTOS_SITE_ROOT").unwrap_or_else(|_| "target/site".to_owned());
    let site_pkg_dir = std::env::var("LEPTOS_SITE_PKG_DIR").unwrap_or_else(|_| "pkg".to_owned());
    let leptos_options = LeptosOptions::builder()
        .output_name("minirust-web")
        .site_root(site_root)
        .site_pkg_dir(site_pkg_dir)
        .site_addr(addr)
        .reload_port(3002)
        .env(leptos_environment)
        .build();

    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!(
        environment = %config.environment,
        address = %addr,
        database_configured = config.database_url.is_some(),
        "starting MiniRust web"
    );

    axum::serve(
        listener,
        router(AppState::new().with_leptos_options(leptos_options)),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    tracing::info!("MiniRust web stopped");
    Ok(())
}

#[cfg(not(feature = "ssr"))]
fn main() {}
