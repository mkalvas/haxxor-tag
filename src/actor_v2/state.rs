use anyhow::anyhow;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::Color;
use ratatui::widgets::canvas::{Painter, Shape};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::api::FullResponse;

pub type GameState = Arc<Mutex<Game>>;

#[derive(Debug, Clone)]
pub struct Game {
    pub game: Option<FullResponse>,
    pub retries: i16,
    pub should_quit: bool,
}

impl Game {
    pub fn new_state() -> GameState {
        Arc::new(Mutex::new(Self {
            game: None,
            retries: 2,
            should_quit: false,
        }))
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        match key.code {
            KeyCode::Char('q') => self.quit(),
            KeyCode::Char('c') => {
                if key.modifiers == KeyModifiers::CONTROL {
                    self.quit()
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }

    pub fn on_tick(&mut self) {}

    fn quit(&mut self) -> anyhow::Result<()> {
        self.should_quit = true;
        Err(anyhow!("Quitting"))
    }
}

impl Shape for Game {
    fn draw(&self, painter: &mut Painter) {
        if let Some(game) = &self.game {
            if let Some((x, y)) = painter.get_point(game.inner.x as f64, game.inner.y as f64) {
                let color = if game.inner.is_it {
                    Color::Red
                } else {
                    Color::LightGreen
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
