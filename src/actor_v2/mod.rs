use rand::seq::SliceRandom;
use tokio::time::{interval, Duration};

use crate::api::ApiClient;

mod actions;
mod position;
mod state;

pub use state::{Game, GameState};

pub async fn run(state: GameState) -> anyhow::Result<()> {
    let client = ApiClient::default();

    // only returns none when slice is empty
    let speed = [10u64].choose(&mut rand::thread_rng()).unwrap();
    let mut interval = interval(Duration::from_millis(*speed));

    loop {
        interval.tick().await;
        let mut lock = state.lock().await;
        if lock.should_quit {
            return actions::try_quit(&client, &mut lock).await;
        }

        let res = actions::take_action(&client, &mut lock).await;
        if res.is_err() || lock.should_quit {
            if lock.retries > 0 {
                lock.retries -= 1;
            } else {
                lock.should_quit = true;
            }
        } else {
            // reset retries after each successful action
            lock.retries = 2;
        };
    }
}
