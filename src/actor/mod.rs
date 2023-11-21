use anyhow::anyhow;
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};

mod actions;
mod api;

pub use actions::{ActorState, SyncedActor};
pub use api::{FullState, MoveDir, PartialState, PlayerLocation, Pos};

pub async fn run() -> anyhow::Result<()> {
    let mut state = ActorState::default();
    let mut interval = interval(Duration::from_millis(500));
    loop {
        interval.tick().await;
        let res = actions::take_action(&mut state).await;
        if res.is_err() || state.dead {
            if state.retries > 0 && !state.dead {
                state.retries -= 1;
            } else {
                state.dead = true;
                let msg = match actions::try_quit(&mut state).await {
                    Ok(()) => "successful",
                    Err(_) => "unsuccessful",
                };
                return Err(anyhow!("Exhausted retries. Graceful quit was {msg}"));
            }
        } else {
            // reset retries after each successful action
            state.retries = 2;
        };
        // print_board(&lock.game);
    }
}

pub async fn run_sync(state: actions::SyncedActor) -> anyhow::Result<()> {
    let mut interval = interval(Duration::from_millis(500));
    loop {
        interval.tick().await;
        let mut lock = state.write().await;
        let res = actions::take_action(&mut lock).await;
        println!("dead: {}", lock.dead);
        if res.is_err() || lock.dead {
            if lock.retries > 0 && !lock.dead {
                lock.retries -= 1;
            } else {
                println!("quitting");
                lock.dead = true;
                let msg = match actions::try_quit(&mut lock).await {
                    Ok(()) => "successful",
                    Err(_) => "unsuccessful",
                };
                drop(lock);
                return Err(anyhow!("Exhausted retries. Graceful quit was {msg}"));
            }
        } else {
            // reset retries after each successful action
            lock.retries = 2;
        };
        // print_board(&lock.game);
    }
}

pub fn run_detached() -> JoinHandle<Result<(), anyhow::Error>> {
    tokio::spawn(run())
}

pub fn run_detached_sync(app: actions::SyncedActor) -> JoinHandle<Result<(), anyhow::Error>> {
    tokio::spawn(run_sync(app))
}
