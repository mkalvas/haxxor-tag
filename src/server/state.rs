use std::sync::Arc;

use anyhow::anyhow;
use rand::Rng;
use serde::Serialize;
use tokio::sync::Mutex;

use crate::api::{FullResponse, MoveDir, PartialResponse, PlayerLocation};

pub type ServerState = Arc<Mutex<GameState>>;

#[derive(Clone, Debug, Serialize)]
pub struct GameState {
    players: Vec<Player>,
    width: i16,
    height: i16,
    stats: Stats,
}

#[derive(Clone, Debug, Serialize)]
pub struct Stats {
    tags: usize,
    requests: usize,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            tags: 0,
            requests: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Player {
    pub id: u16,
    name: String,
    is_it: bool,
    x: i16,
    y: i16,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            players: Vec::new(),
            width: 25,
            height: 10,
            stats: Stats::default(),
        }
    }
}

impl GameState {
    pub fn new_server_state() -> ServerState {
        Arc::new(Mutex::new(Self {
            players: Vec::new(),
            width: 25,
            height: 10,
            stats: Stats::default(),
        }))
    }

    pub fn gen_player(&mut self) -> Player {
        let id = rand::thread_rng().gen_range(1000..2000);
        let (x, y) = self.random_unoccupied();
        let player = Player {
            id,
            name: format!("Player {id}"),
            is_it: self.players.is_empty(),
            x,
            y,
        };
        self.players.push(player.clone());
        player
    }

    pub fn move_player(&mut self, id: u16, dir: &MoveDir) -> anyhow::Result<()> {
        let idx = self.get_player_index(id)?;
        let (dx, dy) = match dir {
            MoveDir::Up => (0, 1),
            MoveDir::Down => (0, -1),
            MoveDir::Left => (-1, 0),
            MoveDir::Right => (1, 0),
            MoveDir::None => (0, 0),
        };

        let (nx, ny) = (self.players[idx].x + dx, self.players[idx].y + dy);

        if self.occupied(nx, ny) {
            // SAFETY: just tested for player at pos, should not panic
            let j = self.get_player_index_at(nx, ny).unwrap();
            if self.players[idx].is_it || self.players[j].is_it {
                self.stats.tags += 1;
                self.players[idx].is_it = !self.players[idx].is_it;
                self.players[j].is_it = !self.players[j].is_it;
            }
        } else {
            self.players[idx].x = nx;
            self.players[idx].y = ny;
        }
        Ok(())
    }

    pub fn respond_to_player(&self, id: u16) -> anyhow::Result<FullResponse> {
        let map_height = self.height;
        let map_width = self.width;
        let players = self.get_other_players(id);
        let current_player = self.get_player(id)?;

        Ok(FullResponse {
            id: current_player.id,
            name: current_player.name.clone(),
            map_height,
            map_width,
            inner: PartialResponse {
                is_it: current_player.is_it,
                players,
                x: current_player.x,
                y: current_player.y,
            },
        })
    }

    pub fn remove_player(&mut self, id: u16) -> anyhow::Result<FullResponse> {
        let response = self.respond_to_player(id);
        let idx = self.get_player_index(id)?;
        self.players.remove(idx);
        self.random_it();
        response
    }

    pub fn occupied(&self, x: i16, y: i16) -> bool {
        self.players.iter().any(|p| p.x == x && p.y == y)
    }

    pub fn get_stats(&self) -> &GameState {
        &self
    }

    pub fn record_request(&mut self) {
        self.stats.requests += 1;
    }

    fn get_player(&self, id: u16) -> anyhow::Result<&Player> {
        self.players
            .iter()
            .find(|p| p.id == id)
            .ok_or(anyhow!("could not find player with id {id}"))
    }

    fn get_player_index(&mut self, id: u16) -> anyhow::Result<usize> {
        self.players
            .iter()
            .position(|p| p.id == id)
            .ok_or(anyhow!("could not find player with id {id}"))
    }

    fn get_player_index_at(&mut self, x: i16, y: i16) -> anyhow::Result<usize> {
        self.players
            .iter()
            .position(|p| p.x == x && p.y == y)
            .ok_or(anyhow!("could not find player at position ({x}, {y})"))
    }

    fn get_other_players(&self, id: u16) -> Vec<PlayerLocation> {
        self.players
            .iter()
            .filter(|p| p.id != id)
            .map(|p| PlayerLocation {
                is_it: p.is_it,
                x: p.x,
                y: p.y,
            })
            .collect()
    }

    fn random_it(&mut self) {
        let player_count = self.players.len();
        if player_count != 0 {
            let new_it = rand::thread_rng().gen_range(0..player_count);
            self.players[new_it].is_it = true;
        }
    }

    fn random_unoccupied(&self) -> (i16, i16) {
        let mut rng = rand::thread_rng();
        loop {
            let x = rng.gen_range(0..self.width);
            let y = rng.gen_range(0..self.height);
            if !self.occupied(x, y) {
                return (x, y);
            }
        }
    }
}
