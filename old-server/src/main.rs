pub mod app_config;
pub mod db;
pub mod handlers;
pub mod security;
pub mod session;

use std::net::Ipv4Addr;

use actix_web::{App, HttpServer, middleware, web::Data};
use rusqlite::Connection;

use crate::app_config::config_app;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting HTTP server at http://localhost:8080");

    HttpServer::new(move || {
        let connection = Connection::open("./db.db3").unwrap();
        App::new()
            .configure(config_app)
            .wrap(middleware::NormalizePath::trim())
            .wrap(middleware::Logger::default())
            .app_data(Data::new(connection))
    })
    .bind((Ipv4Addr::UNSPECIFIED, 8080))?
    .run()
    .await?;

    Ok(())
}
