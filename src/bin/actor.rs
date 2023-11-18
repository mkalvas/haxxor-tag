use anyhow::anyhow;
use serde::Deserialize;
use tokio::time::{interval, Duration};
use xor_tag::{CommandResult, MoveDir, RegisterResult, URL};

#[derive(Debug)]
struct ActorState {
    game: Option<RegisterResult>,
    retries: i8,
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let mut state = ActorState {
        game: None,
        retries: 2,
    };

    let mut interval = interval(Duration::from_secs(1));
    loop {
        interval.tick().await;
        let res = take_action(&mut state).await;
        if res.is_err() {
            if state.retries > 0 {
                state.retries -= 1;
            } else {
                // take_action will quit when appropriate, so we call it again
                // making sure we're in the state that will ask it to quit
                state.retries -= 1;
                let msg = match take_action(&mut state).await {
                    Ok(_) => "successful",
                    Err(_) => "unsuccessful",
                };
                return Err(anyhow!("Exhausted retries. Graceful quit was {msg}"));
            }
        } else {
            // reset retries after each success
            state.retries = 2;
        };
        println!("{state:#?}");
    }
}

enum Action {
    Look,
    Move(MoveDir),
    Register,
    Quit,
}

/// Determine the best course of action and take it.
async fn take_action(state: &mut ActorState) -> anyhow::Result<()> {
    match determine_action(state) {
        Action::Register => {
            println!("registering");
            let new_state = register().await?;
            state.game = Some(new_state);
        }
        Action::Look => {
            println!("looking");
            match &mut state.game {
                None => return Err(anyhow!("Cannot look before registering as a player")),
                Some(s) => {
                    let new_partial = look(s.id).await?;
                    s.inner = new_partial;
                }
            }
        }
        Action::Move(_) => {
            println!("moving");
            match &mut state.game {
                None => return Err(anyhow!("Cannot look before registering as a player")),
                Some(s) => {
                    let new_partial = look(s.id).await?;
                    s.inner = new_partial;
                }
            }
        }
        Action::Quit => {
            println!("quitting");
            match &mut state.game {
                None => return Err(anyhow!("Cannot quit before registering as a player")),
                Some(s) => {
                    let new_partial = quit(s.id).await?;
                    s.inner = new_partial;
                }
            }
        }
    }
    Ok(())
}

fn determine_action(state: &mut ActorState) -> Action {
    if state.game.is_none() && state.retries >= 0 {
        Action::Register
    } else if state.retries < 0 {
        Action::Quit
    } else {
        Action::Look
    }
}

/// Make a call to a player action endpoint
async fn call<T: for<'de> Deserialize<'de>>(url: &str) -> anyhow::Result<T> {
    Ok(reqwest::get(url).await?.json::<T>().await?)
}

/// This is the first step you'll need to do.
///
/// When you register the game will create your player, assign you an id, pick a
/// name for you and put your player on the map.
///
/// When you register you'll get back a JSON object `Res` that
/// will let you know what your id is and where your player is.
///
/// To register you need to make an HTTP request to the following url:
///     `http://xortag.apphb.com/register`
async fn register() -> anyhow::Result<RegisterResult> {
    call(&format!("{URL}/register")).await
}

/// Once you are registered you can start moving your player around. This is the
/// heart of tag.
///
/// If you are "it", try to move towards other players and tag them.
///
/// If you aren't "it", try to run away from the player who is.
///
/// If you move to the same space where another player is and one of you is it,
/// that counts as a tag. If neither of you are it, you won't go anywhere. No
/// piggybacking here.
async fn mv(id: u16, dir: MoveDir) -> anyhow::Result<CommandResult> {
    call(&format!("{URL}/move{dir}/{id}")).await
}

/// If you want to get an update on what's going on in the world, but don't want
/// to lose the sweet spot you have claimed, you can do that by looking. To
/// look, make an HTTP request to the following url:
///     `http://xortag.apphb.com/look/{your_player_id}`
///
/// As with moving, make sure to supply your user id. Also, in response to your
/// request you'll receive back an updated JSON object.
async fn look(id: u16) -> anyhow::Result<CommandResult> {
    call(&format!("{URL}/look/{id}")).await
}

/// Attempt to quit from the game. If this call succeeds, the server will remove
/// the player and return the final state that the player would have seen.
async fn quit(id: u16) -> anyhow::Result<CommandResult> {
    call(&format!("{URL}/quit/{id}")).await
}
