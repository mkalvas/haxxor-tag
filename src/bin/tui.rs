use anyhow::anyhow;
use haxxor_tag::{actor_v2, tui};
use tokio::join;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    tui::setup_panic_hook();
    let state = actor_v2::Game::new_state();
    let actor = tokio::spawn(actor_v2::run(state.clone()));
    let ui = tui::run(state);
    let thing = join!(actor, ui);
    match thing {
        (Ok(_), Ok(())) => Ok(()),
        err_tuple => Err(anyhow!("{err_tuple:#?}")),
    }
}
