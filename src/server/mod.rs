use anyhow::anyhow;
use tokio::net::TcpListener;

mod routes;
mod state;

use state::GameState;

pub const URL: &str = "http://localhost:3000";

pub async fn serve() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let state = GameState::new_server_state();
    let router = routes::build_router(state);

    match TcpListener::bind("127.0.0.1:3000").await {
        Ok(listener) => {
            tracing::info!("listening on localhost:3000");
            let result = axum::serve(listener, router).await;
            result.map_err(|e| anyhow!(e))
        }
        Err(e) => {
            tracing::error!("failed to bind tcp listener on 0.0.0.0:3000");
            Err(anyhow!(e))
        }
    }
}
