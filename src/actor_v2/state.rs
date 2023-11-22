use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::Color;
use ratatui::widgets::canvas::{Painter, Shape};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::api::FullResponse;

pub type SyncedActor = Arc<RwLock<ActorState>>;

#[derive(Debug, Clone)]
pub struct ActorState {
    pub game: Option<FullResponse>,
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
