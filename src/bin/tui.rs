use xor_tag::actor::ActorState;
use xor_tag::{actor, tui};

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    tui::setup_panic_hook();
    let app = ActorState::new_synced();
    actor::run_detached_sync(app.clone());
    tui::run(app).await
}
