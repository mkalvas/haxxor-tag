#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let state = haxxor_tag::actor::Game::new_state();
    haxxor_tag::actor::run(state).await
}
