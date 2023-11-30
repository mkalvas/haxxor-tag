use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, Request, Response, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::classify::ServerErrorsFailureClass;
use tower_http::trace::TraceLayer;
use tracing::Span;

use crate::api::MoveDir;

use super::state::ServerState;

pub fn build_router(state: ServerState) -> Router {
    Router::new()
        .route("/register", get(register))
        .route("/look/:pid", get(look))
        .route("/move:dir/:pid", get(movement))
        .route("/quit/:pid", get(quit))
        .with_state(state)
        .with_middleware()
}

trait WithMiddleware {
    fn with_middleware(self) -> Self;
}

impl WithMiddleware for Router {
    fn with_middleware(self) -> Self {
        let tracer = TraceLayer::new_for_http()
            .make_span_with(|_req: &Request<_>| tracing::info_span!("http-request"))
            .on_request(|req: &Request<_>, _span: &Span| {
                tracing::info!("started {} {}", req.method(), req.uri().path());
            })
            .on_response(|_response: &Response<_>, latency: Duration, _span: &Span| {
                tracing::info!("response generated in {:#?}", latency);
            })
            .on_body_chunk(|chunk: &Bytes, _latency: Duration, _span: &Span| {
                tracing::debug!("sending {} bytes", chunk.len());
            })
            .on_eos(
                |_trailers: Option<&HeaderMap>, stream_duration: Duration, _span: &Span| {
                    tracing::debug!("stream closed after {:?}", stream_duration);
                },
            )
            .on_failure(
                |err: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                    tracing::info!("something went wrong: {}", err);
                },
            );

        self.layer(ServiceBuilder::new().layer(tracer))
    }
}

pub async fn register(State(data): State<ServerState>) -> impl IntoResponse {
    let mut state = data.lock().await;
    let new_player = state.gen_player();
    match state.respond_to_player(new_player.id) {
        Some(res) => Json(res).into_response(),
        None => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

pub async fn look(State(data): State<ServerState>, Path(pid): Path<u16>) -> impl IntoResponse {
    let state = data.lock().await;
    match state.respond_to_player(pid) {
        Some(res) => Json(res).into_response(),
        None => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

pub async fn movement(
    State(data): State<ServerState>,
    Path((dir, pid)): Path<(String, u16)>,
) -> impl IntoResponse {
    let mut state = data.lock().await;
    if state.move_player(pid, &MoveDir::from(&dir)).is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
    };

    match state.respond_to_player(pid) {
        Some(res) => Json(res).into_response(),
        None => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

pub async fn quit(State(data): State<ServerState>, Path(pid): Path<u16>) -> impl IntoResponse {
    let mut state = data.lock().await;
    match state.remove_player(pid) {
        Some(res) => Json(res).into_response(),
        None => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}
