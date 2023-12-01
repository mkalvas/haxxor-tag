use anyhow::anyhow;
use tokio::net::TcpListener;

mod routes;
mod state;

use state::GameState;

pub fn url() -> String {
    std::env::var("HAXXOR_URL").unwrap_or("http://127.0.0.1:3000".into())
}

fn host() -> String {
    std::env::var("HAXXOR_HOST").unwrap_or("127.0.0.1:3000".into())
}

pub async fn serve() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let state = GameState::new_server_state();
    let router = routes::build_router(state);
    let host = host();

    match TcpListener::bind(&host).await {
        Ok(listener) => {
            tracing::info!("listening on {host}");
            let result = axum::serve(listener, router).await;
            result.map_err(|e| anyhow!(e))
        }
        Err(e) => {
            tracing::error!("failed to bind tcp listener on {host}");
            Err(anyhow!(e))
        }
    }
}
