use actix_web::middleware::Logger;
use actix_web::web::{Data, Path};
use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use anyhow::anyhow;
use rand::Rng;
use tokio::sync::Mutex;

use xor_tag::{CommandResult, MoveDir, PlayerLocation, RegisterResult};

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let state = Data::new(AppState {
        inner: Mutex::new(GameState::new()),
    });

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(state.clone())
            .service(register)
            .service(look)
            .service(movement)
            .service(quit)
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}

#[get("/register")]
async fn register(data: Data<AppState>) -> impl Responder {
    let mut state = data.inner.lock().await;
    let new_player = state.gen_player();
    match state.respond_to_player(&new_player.id) {
        Some(res) => HttpResponse::Ok().json(res),
        None => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/look/{pid}")]
async fn look(data: Data<AppState>, pid: Path<u16>) -> impl Responder {
    let mut state = data.inner.lock().await;
    match state.respond_to_player(&pid) {
        Some(res) => HttpResponse::Ok().json(res),
        None => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/move{direction}/{pid}")]
async fn movement(data: Data<AppState>, path: Path<(MoveDir, u16)>) -> impl Responder {
    let mut state = data.inner.lock().await;
    if state.move_player(&path.1, &path.0).is_err() {
        return HttpResponse::InternalServerError().finish();
    };

    match state.respond_to_player(&path.1) {
        Some(res) => HttpResponse::Ok().json(res),
        None => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/quit/{pid}")]
async fn quit(data: Data<AppState>, pid: Path<u16>) -> impl Responder {
    let mut state = data.inner.lock().await;
    match state.remove_player(&pid) {
        Some(res) => HttpResponse::Ok().json(res),
        None => HttpResponse::InternalServerError().finish(),
    }
}

#[derive(Debug)]
pub struct AppState {
    inner: Mutex<GameState>,
}

#[derive(Clone, Debug)]
pub struct Player {
    id: u16,
    name: String,
    is_it: bool,
    x: i8,
    y: i8,
}

#[derive(Debug)]
pub struct GameState {
    players: Vec<Player>,
    width: i8,
    height: i8,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            players: Vec::new(),
            width: 50,
            height: 30,
        }
    }

    pub fn gen_player(&mut self) -> Player {
        let id = rand::thread_rng().gen_range(1000..2000);
        let (x, y) = self.random_unoccupied();
        let player = Player {
            id,
            name: format!("Player {id}"),
            is_it: self.players.len() == 0,
            x,
            y,
        };
        self.players.push(player.clone());
        player
    }
    pub fn move_player(&mut self, id: &u16, dir: &MoveDir) -> anyhow::Result<()> {
        let player_index = self.get_player_index(id);
        match player_index {
            None => Err(anyhow!("Could not find player {id} to move")),
            Some(idx) => {
                let (dx, dy) = match dir {
                    MoveDir::Up => (0, 1),
                    MoveDir::Down => (0, -1),
                    MoveDir::Left => (-1, 0),
                    MoveDir::Right => (0, 1),
                };

                let (nx, ny) = (self.players[idx].x + dx, self.players[idx].y + dy);
                if !self.occupied(nx, ny) {
                    self.players[idx].x = nx;
                    self.players[idx].y = ny;
                }
                Ok(())
            }
        }
    }

    pub fn respond_to_player(&mut self, id: &u16) -> Option<RegisterResult> {
        let map_height = self.height;
        let map_width = self.width;
        let players = self.get_other_players(id);
        let current_player = self.get_player(id)?;

        Some(RegisterResult {
            id: current_player.id,
            name: current_player.name.clone(),
            map_height,
            map_width,
            inner: CommandResult {
                is_it: current_player.is_it,
                players,
                x: current_player.x,
                y: current_player.y,
            },
        })
    }

    pub fn remove_player(&mut self, id: &u16) -> Option<RegisterResult> {
        let response = self.respond_to_player(id);
        let idx = self.get_player_index(id)?;
        self.players.remove(idx);
        response
    }

    fn get_player(&mut self, id: &u16) -> Option<&Player> {
        self.players.iter().find(|p| &p.id == id)
    }

    fn get_player_index(&mut self, id: &u16) -> Option<usize> {
        self.players.iter().position(|p| &p.id == id)
    }

    fn get_other_players(&self, id: &u16) -> Vec<PlayerLocation> {
        self.players
            .iter()
            .filter(|p| &p.id != id)
            .map(|p| PlayerLocation {
                is_it: p.is_it,
                x: p.x,
                y: p.y,
            })
            .collect()
    }

    fn random_unoccupied(&self) -> (i8, i8) {
        let mut rng = rand::thread_rng();
        loop {
            let x = rng.gen_range(0..self.width);
            let y = rng.gen_range(0..self.height);
            if !self.occupied(x, y) {
                return (x, y);
            }
        }
    }

    fn occupied(&self, x: i8, y: i8) -> bool {
        self.players.iter().any(|p| p.x == x && p.y == y)
    }
}
