use minirust_api::{router, AppState};
use minirust_config::{Config, ServerKind};
use minirust_database::Database;
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

    let database_url = config.database_url()?;
    let database = Database::connect(database_url).await?;
    let bind = config.server_bind(ServerKind::Api)?;
    let addr = bind.socket_addr()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!(environment = %config.environment, address = %addr, "starting MiniRust API");

    axum::serve(listener, router(AppState::new(database)))
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("MiniRust API stopped");
    Ok(())
}
