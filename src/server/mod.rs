use anyhow::anyhow;
use std::net::SocketAddr;
mod routes;
mod state;

use state::GameState;

pub const URL: &str = "http://localhost:3000";

pub async fn serve() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let state = GameState::new();
    let router = routes::build_router(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    let result = axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await;

    result.map_err(|e| anyhow!(e))
}
