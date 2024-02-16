use anyhow::anyhow;
use haxxor_tag::{actor, tui};
use tokio::join;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    tui::setup_panic_hook();
    let state = actor::Game::new_state();
    let actor = tokio::spawn(actor::run(state.clone()));
    // spawn_enemies();
    let ui = tui::run(state);
    let thing = join!(actor, ui);
    match thing {
        (Ok(_), Ok(())) => Ok(()),
        err_tuple => Err(anyhow!("{err_tuple:#?}")),
    }
}

// fn spawn_enemies() {
//     for _ in 0..4 {
//         tokio::spawn(async move {
//             let state = haxxor_tag::actor::Game::new_state();
//             haxxor_tag::actor::run(state).await
//         });
//     }
// }
