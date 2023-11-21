use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use std::io;

mod routes;
mod state;

pub async fn serve() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let state = Data::new(state::AppState::default());

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(state.clone())
            .service(routes::register)
            .service(routes::look)
            .service(routes::movement)
            .service(routes::quit)
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
