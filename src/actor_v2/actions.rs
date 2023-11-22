use anyhow::anyhow;
use pathfinding::prelude::astar;

use crate::api::{ApiClient, FullResponse, MoveDir};

use super::position::Pos;
use super::state::Game;

pub enum Action {
    Look,
    Move(MoveDir),
    Register,
}

pub async fn try_quit(client: &ApiClient, state: &mut Game) -> anyhow::Result<()> {
    match &mut state.game {
        None => Err(anyhow!("Cannot quit before registering as a player")),
        Some(s) => {
            let new_partial = client.quit(s.id).await?;
            s.inner = new_partial;
            Ok(())
        }
    }
}

/// Determine the best course of action and take it.
pub async fn take_action(client: &ApiClient, state: &mut Game) -> anyhow::Result<()> {
    match determine_action(state) {
        Action::Register => {
            let new_state = client.register().await?;
            state.game = Some(new_state);
        }
        Action::Look => match &mut state.game {
            None => return Err(anyhow!("Cannot look before registering as a player")),
            Some(s) => {
                let new_partial = client.look(s.id).await?;
                s.inner = new_partial;
            }
        },
        Action::Move(dir) => match &mut state.game {
            None => return Err(anyhow!("Cannot move before registering as a player")),
            Some(s) => {
                let new_partial = client.mv(s.id, dir.clone()).await?;
                s.inner = new_partial;
            }
        },
    }
    Ok(())
}

fn determine_action(state: &Game) -> Action {
    match &state.game {
        None => Action::Register,
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

fn chase_dir(game: &FullResponse) -> MoveDir {
    let me = Pos(game.inner.x, game.inner.y);
    let target = closest_player(game, &me);
    let path = astar(
        &me,
        |p| p.successors(&game, true),
        |p| p.distance(&target),
        |p| *p == target,
    );
    dir_from_path(&me, path)
}

fn flee_dir(game: &FullResponse) -> MoveDir {
    let me = Pos(game.inner.x, game.inner.y);
    let it = it_player_pos(game);
    let target = max_square(game, &it);
    let path = astar(
        &me,
        |p| p.successors(&game, false),
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

fn max_square(game: &FullResponse, it: &Pos) -> Pos {
    let mut target = Pos(game.inner.x, game.inner.y);
    let mut max = 0;
    for x in 0..game.map_width {
        for y in 0..game.map_height {
            let pt = Pos(x, y);
            let d = pt.distance(it);
            let is_me = game.inner.x == x && game.inner.y == y;
            if d > max && (!game.occupied(x, y) || is_me) {
                max = d;
                target = pt;
            }
        }
    }
    target
}

fn it_player_pos(game: &FullResponse) -> Pos {
    match game.inner.players.iter().find(|p| p.is_it) {
        Some(p) => Pos(p.x, p.y),
        None => Pos(game.inner.x, game.inner.y),
    }
}

fn closest_player(game: &FullResponse, me: &Pos) -> Pos {
    let mut closest = None;
    for p in &game.inner.players {
        let d = me.distance(&Pos(p.x, p.y));
        match closest {
            None => {
                closest = Some((p.x, p.y, d));
            }
            Some(c) => {
                if d < c.2 {
                    closest = Some((p.x, p.y, d));
                }
            }
        }
    }
    match closest {
        Some(c) => Pos(c.0, c.1),
        None => Pos(game.inner.x, game.inner.y), // stand still if no one exists
    }
}
