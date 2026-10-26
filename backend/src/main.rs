mod alex;
mod config;
mod db;
mod draft_worker;
mod error;
mod models;
mod product_vision;
mod routes;
mod state;
mod worker;

use config::Config;
use state::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
    if let Err(error) = run().await {
        eprintln!("Startup or server failed: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    let port = config.port;
    // Return only sanitized errors from startup, too.
    let state = AppState::connect(config).await?;
    db::indexes::ensure(&state).await?;
    let router = routes::router(state.clone())?;
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::UNSPECIFIED, port)).await?;
    let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);
    let shutdown_tx = stop_tx.clone();
    let draft_worker = tokio::spawn(draft_worker::run(state.clone(), stop_rx.clone()));
    let worker = tokio::spawn(worker::run(state, stop_rx));
    tracing::info!(port, "LINE inquiry receiver listening");
    let result = axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            shutdown().await;
            let _ = shutdown_tx.send(true);
        })
        .await;
    let _ = stop_tx.send(true);
    worker.await?;
    draft_worker.await?;
    result?;
    Ok(())
}

async fn shutdown() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut signal) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            signal.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => (), _ = terminate => () }
}

#[cfg(test)]
mod tests;
