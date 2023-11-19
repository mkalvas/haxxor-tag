use anyhow::anyhow;
use pathfinding::prelude::astar;
use serde::Deserialize;
use tokio::time::{interval, Duration};
use xor_tag::{CommandResult, MoveDir, Pos, RegisterResult, URL};

#[derive(Debug)]
struct ActorState {
    game: Option<RegisterResult>,
    retries: i16,
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
        print_board(&state.game);
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
            println!("{new_state:#?}");
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
        Action::Move(dir) => {
            println!("moving {dir}");
            match &mut state.game {
                None => return Err(anyhow!("Cannot move before registering as a player")),
                Some(s) => {
                    let new_partial = mv(s.id, dir).await?;
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
    match &state.game {
        None => {
            if state.retries >= 0 {
                Action::Register
            } else {
                Action::Quit
            }
        }
        Some(game) => {
            let dir = if game.inner.is_it {
                chase_dir(game)
            } else {
                flee_dir(game)
            };

            if dir == MoveDir::None {
                Action::Look
            } else {
                Action::Move(dir)
            }
        }
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

fn chase_dir(game: &RegisterResult) -> MoveDir {
    let me = Pos(game.inner.x, game.inner.y);
    let target = closest_player(game, &me);
    let path = astar(
        &me,
        |p| p.successors(), // TODO occupied tiles
        |p| p.distance(&target),
        |p| *p == target,
    );
    dir_from_path(&me, path)
}

fn flee_dir(game: &RegisterResult) -> MoveDir {
    let me = Pos(game.inner.x, game.inner.y);
    let it = it_player_pos(game);
    let target = max_square(game, &it);
    let path = astar(
        &me,
        |p| p.successors(), // TODO occupied tiles
        |p| p.distance(&target),
        |p| *p == target,
    );
    dir_from_path(&me, path)
}

fn dir_from_path(me: &Pos, path: Option<(Vec<Pos>, u16)>) -> MoveDir {
    match path {
        None => MoveDir::None,
        Some((steps, _)) => {
            // steps[0] is current square, steps[1] is target
            // if length is 1, we're on optimal square already
            if steps.len() == 1 {
                return MoveDir::None;
            }

            let delta = (steps[1].0 - me.0, steps[1].1 - me.1);
            match delta {
                (1, 0) => MoveDir::Right,
                (-1, 0) => MoveDir::Left,
                (0, 1) => MoveDir::Up,
                (0, -1) => MoveDir::Down,
                _ => MoveDir::None,
            }
        }
    }
}

fn max_square(game: &RegisterResult, it: &Pos) -> Pos {
    let mut target = Pos(game.inner.x, game.inner.y);
    let mut max = 0;
    for x in 0..game.map_width {
        for y in 0..game.map_height {
            let pt = Pos(x, y);
            let d = pt.distance(it);
            if d > max {
                max = d;
                target = pt;
            }
        }
    }
    target.clone()
}

fn it_player_pos(game: &RegisterResult) -> Pos {
    match game.inner.players.iter().find(|p| p.is_it) {
        Some(p) => Pos(p.x, p.y),
        None => Pos(game.inner.x, game.inner.y),
    }
}

fn closest_player(game: &RegisterResult, me: &Pos) -> Pos {
    let mut closest = None;
    for p in &game.inner.players {
        let d = me.distance(&Pos(p.x, p.y));
        match closest {
            None => closest = Some((p.x, p.y, d)),
            Some(c) => {
                if d < c.2 {
                    closest = Some((p.x, p.y, d))
                }
            }
        }
    }
    match closest {
        Some(c) => Pos(c.0, c.1),
        None => Pos(game.inner.x, game.inner.y), // stand still if no one exists
    }
}

fn print_board(maybe_game: &Option<RegisterResult>) {
    match maybe_game {
        None => println!("no board"),
        Some(game) => {
            let mut s = "|".to_string();
            s.push_str("-".repeat(game.map_width as usize).as_str());
            s.push_str("|");
            println!("{s}");

            for y in (0..game.map_height).rev() {
                s = "|".to_string();
                for x in 0..game.map_width {
                    s.push_str(symbol(game, x, y).as_str());
                }
                s.push_str("|");
                println!("{s}");
            }

            s = "|".to_string();
            s.push_str("-".repeat(game.map_width as usize).as_str());
            s.push_str("|");
            println!("{s}");
        }
    }
}

// TODO: TUI
fn symbol(game: &RegisterResult, x: i16, y: i16) -> String {
    let is_me = game.inner.x == x && game.inner.y == y;
    let is_other = game.inner.players.iter().find(|p| p.x == x && p.y == y);
    if is_me && game.inner.is_it {
        "X".to_string()
    } else if is_other.is_some_and(|p| p.is_it) {
        "X".to_string()
    } else if is_me {
        "M".to_string()
    } else if is_other.is_some() {
        "O".to_string()
    } else {
        " ".to_string()
    }
}
