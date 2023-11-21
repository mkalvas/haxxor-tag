use actix_web::web::{Data, Path};
use actix_web::{get, HttpResponse, Responder};

use crate::actor::MoveDir;

use super::state::AppState;

#[get("/register")]
pub async fn register(data: Data<AppState>) -> impl Responder {
    let mut state = data.inner.write().await;
    let new_player = state.gen_player();
    match state.respond_to_player(new_player.id) {
        Some(res) => HttpResponse::Ok().json(res),
        None => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/look/{pid}")]
pub async fn look(data: Data<AppState>, pid: Path<u16>) -> impl Responder {
    let state = data.inner.read().await;
    match state.respond_to_player(*pid) {
        Some(res) => HttpResponse::Ok().json(res),
        None => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/move{direction}/{pid}")]
pub async fn movement(data: Data<AppState>, path: Path<(String, u16)>) -> impl Responder {
    let mut state = data.inner.write().await;
    if state.move_player(path.1, &MoveDir::from(&path.0)).is_err() {
        return HttpResponse::InternalServerError().finish();
    };

    match state.respond_to_player(path.1) {
        Some(res) => HttpResponse::Ok().json(res),
        None => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/quit/{pid}")]
pub async fn quit(data: Data<AppState>, pid: Path<u16>) -> impl Responder {
    let mut state = data.inner.write().await;
    match state.remove_player(*pid) {
        Some(res) => HttpResponse::Ok().json(res),
        None => HttpResponse::InternalServerError().finish(),
    }
}
