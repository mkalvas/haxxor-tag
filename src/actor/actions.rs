use anyhow::anyhow;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use pathfinding::prelude::astar;
use ratatui::style::Color;
use ratatui::widgets::canvas::{Painter, Shape};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::api::{self, FullState, MoveDir, Pos};

pub type SyncedActor = Arc<RwLock<ActorState>>;

#[derive(Debug, Clone)]
pub struct ActorState {
    pub game: Option<FullState>,
    pub retries: i16,
    pub dead: bool,
}

impl Default for ActorState {
    fn default() -> Self {
        Self {
            game: None,
            retries: 2,
            dead: false,
        }
    }
}

impl ActorState {
    pub fn new_synced() -> SyncedActor {
        Arc::new(RwLock::new(Self::default()))
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        match key.code {
            KeyCode::Char('q') => self.dead = true,
            KeyCode::Char('c') => {
                if key.modifiers == KeyModifiers::CONTROL {
                    self.dead = true;
                }
            }
            _ => {} // all other keys unbound
        };
        Ok(())
    }

    pub fn on_tick(&mut self) {}
}

impl Shape for ActorState {
    fn draw(&self, painter: &mut Painter) {
        if let Some(game) = &self.game {
            if let Some((x, y)) = painter.get_point(game.inner.x as f64, game.inner.y as f64) {
                let color = if game.inner.is_it {
                    Color::Red
                } else {
                    Color::Cyan
                };
                painter.paint(x, y, color);
            }
            for player in &game.inner.players {
                if let Some((x, y)) = painter.get_point(player.x as f64, player.y as f64) {
                    let color = if player.is_it {
                        Color::Red
                    } else {
                        Color::Cyan
                    };
                    painter.paint(x, y, color);
                };
            }
        }
    }
}

enum Action {
    Look,
    Move(MoveDir),
    Register,
}

pub async fn try_quit(state: &mut ActorState) -> anyhow::Result<()> {
    // println!("quitting");
    match &mut state.game {
        None => Err(anyhow!("Cannot quit before registering as a player")),
        Some(s) => {
            let new_partial = api::quit(s.id).await?;
            s.inner = new_partial;
            Ok(())
        }
    }
}

/// Determine the best course of action and take it.
pub async fn take_action(state: &mut ActorState) -> anyhow::Result<()> {
    match determine_action(state) {
        Action::Register => {
            // println!("registering");
            let new_state = api::register().await?;
            // println!("{new_state:#?}");
            state.game = Some(new_state);
        }
        Action::Look => {
            // println!("looking");
            match &mut state.game {
                None => return Err(anyhow!("Cannot look before registering as a player")),
                Some(s) => {
                    let new_partial = api::look(s.id).await?;
                    s.inner = new_partial;
                }
            }
        }
        Action::Move(dir) => {
            // println!("moving {dir}");
            match &mut state.game {
                None => return Err(anyhow!("Cannot move before registering as a player")),
                Some(s) => {
                    let new_partial = api::mv(s.id, dir).await?;
                    s.inner = new_partial;
                }
            }
        }
    }
    Ok(())
}

fn determine_action(state: &ActorState) -> Action {
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

fn chase_dir(game: &FullState) -> MoveDir {
    let me = Pos(game.inner.x, game.inner.y);
    let target = closest_player(game, &me);
    let path = astar(
        &me,
        |p| p.successors(&game.inner),
        |p| p.distance(&target),
        |p| *p == target,
    );
    dir_from_path(&me, path)
}

fn flee_dir(game: &FullState) -> MoveDir {
    let me = Pos(game.inner.x, game.inner.y);
    let it = it_player_pos(game);
    let target = max_square(game, &it);
    let path = astar(
        &me,
        |p| p.successors(&game.inner),
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

fn max_square(game: &FullState, it: &Pos) -> Pos {
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
    target
}

fn it_player_pos(game: &FullState) -> Pos {
    match game.inner.players.iter().find(|p| p.is_it) {
        Some(p) => Pos(p.x, p.y),
        None => Pos(game.inner.x, game.inner.y),
    }
}

fn closest_player(game: &FullState, me: &Pos) -> Pos {
    let mut closest = None;
    for p in &game.inner.players {
        let d = me.distance(&Pos(p.x, p.y));
        match closest {
            None => closest = Some((p.x, p.y, d)),
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
