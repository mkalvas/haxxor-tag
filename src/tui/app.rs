use anyhow::anyhow;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::Color;
use ratatui::widgets::canvas::{Painter, Shape};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::actor::Pos;

pub type AppState = Arc<Mutex<App>>;

pub struct App {
    players: HashSet<Pos>,
}

impl App {
    pub fn new() -> AppState {
        Arc::new(Mutex::new(Self {
            players: HashSet::new(),
        }))
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        match key.code {
            KeyCode::Char('q') => return Err(anyhow!("quitting")),
            KeyCode::Char('c') => {
                if key.modifiers == KeyModifiers::CONTROL {
                    return Err(anyhow!("quitting"));
                }
            }
            _ => {} // all other keys unbound
        };
        Ok(())
    }

    pub fn on_tick(&mut self) {}
}

impl Shape for App {
    fn draw(&self, painter: &mut Painter) {
        for player in &self.players {
            if let Some((x, y)) = painter.get_point(player.0 as f64, player.1 as f64) {
                painter.paint(x, y, Color::LightGreen);
            }
        }
    }
}
