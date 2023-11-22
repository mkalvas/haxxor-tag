use std::sync::Arc;

use anyhow::anyhow;
use rand::Rng;
use tokio::sync::Mutex;

use crate::api::{FullResponse, MoveDir, PartialResponse, PlayerLocation};

pub type ServerState = Arc<Mutex<GameState>>;

#[derive(Clone, Debug)]
pub struct GameState {
    players: Vec<Player>,
    width: i16,
    height: i16,
}

#[derive(Clone, Debug)]
pub struct Player {
    pub id: u16,
    name: String,
    is_it: bool,
    x: i16,
    y: i16,
}

impl GameState {
    pub fn new_server_state() -> ServerState {
        Arc::new(Mutex::new(Self {
            players: Vec::new(),
            width: 30,
            height: 10,
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
        let player_index = self.get_player_index(id);
        match player_index {
            None => Err(anyhow!("Could not find player {id} to move")),
            Some(idx) => {
                let (dx, dy) = match dir {
                    MoveDir::Up => (0, 1),
                    MoveDir::Down => (0, -1),
                    MoveDir::Left => (-1, 0),
                    MoveDir::Right => (1, 0),
                    MoveDir::None => (0, 0),
                };

                let (nx, ny) = (self.players[idx].x + dx, self.players[idx].y + dy);
                // TODO: tagging
                if !self.occupied(nx, ny) {
                    self.players[idx].x = nx;
                    self.players[idx].y = ny;
                }
                Ok(())
            }
        }
    }

    pub fn respond_to_player(&self, id: u16) -> Option<FullResponse> {
        let map_height = self.height;
        let map_width = self.width;
        let players = self.get_other_players(id);
        let current_player = self.get_player(id)?;

        Some(FullResponse {
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

    pub fn remove_player(&mut self, id: u16) -> Option<FullResponse> {
        let response = self.respond_to_player(id);
        let idx = self.get_player_index(id)?;
        self.players.remove(idx);
        response
    }

    fn get_player(&self, id: u16) -> Option<&Player> {
        self.players.iter().find(|p| p.id == id)
    }

    fn get_player_index(&mut self, id: u16) -> Option<usize> {
        self.players.iter().position(|p| p.id == id)
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

    pub fn occupied(&self, x: i16, y: i16) -> bool {
        self.players.iter().any(|p| p.x == x && p.y == y)
    }
}
