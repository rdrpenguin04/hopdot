pub mod app_config;
pub mod handlers;
pub mod security;
pub mod session;

use std::io;
use std::net::Ipv4Addr;

use actix_web::{App, HttpServer, middleware};

use crate::app_config::config_app;

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting HTTP server at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .configure(config_app)
            .wrap(middleware::NormalizePath::trim())
            .wrap(middleware::Logger::default())
    })
    .bind((Ipv4Addr::UNSPECIFIED, 8080))?
    .run()
    .await
}
