// use rand::seq::SliceRandom;
// use rand::Rng;
use tokio::time::{interval, Duration};

use crate::api::ApiClient;

mod actions;
mod position;
mod state;

pub use state::{Game, GameState};

pub async fn run(state: GameState) -> anyhow::Result<()> {
    let client = ApiClient::default();

    // let speed = [100u64].choose(&mut rand::thread_rng()).unwrap();
    // let speed = rand::thread_rng().gen_range(200);
    // let mut interval = interval(Duration::from_micros(100u64));
    let mut interval = interval(Duration::from_millis(1010u64));

    loop {
        interval.tick().await;
        // random hiccups to make the game more interesting
        // if rand::thread_rng().gen_bool(1.0 / 2.0) {
        //     interval.tick().await;
        // }

        let mut lock = state.lock().await;
        if lock.should_quit {
            // println!("quitting");
            return actions::try_quit(&client, &mut lock).await;
        }

        let res = actions::take_action(&client, &mut lock).await;
        if res.is_err() || lock.should_quit {
            // println!("error {res:#?}");
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
