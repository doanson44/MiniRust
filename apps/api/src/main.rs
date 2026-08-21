use minirust_api::router;
use minirust_api::AppState;
use minirust_config::{Config, ServerKind};
use tracing_subscriber::EnvFilter;

fn init_tracing(log_filter: &str) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_filter));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .compact()
        .init();
}

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        tracing::error!(%error, "failed to listen for shutdown signal");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    init_tracing(&config.log_filter);

    let bind = config.server_bind(ServerKind::Api)?;
    let addr = bind.socket_addr()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!(
        environment = %config.environment,
        address = %addr,
        database_configured = config.database_url.is_some(),
        redis_configured = config.redis_url.is_some(),
        "starting MiniRust API"
    );

    axum::serve(listener, router(AppState::new()))
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("MiniRust API stopped");
    Ok(())
}
