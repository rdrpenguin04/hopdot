use std::sync::RwLock;

use actix_web::web::{self, Data, ServiceConfig};

use crate::handlers::{
    lobby::{self, LobbyData},
    sessions, users,
};

pub fn config_users(cfg: &mut ServiceConfig) {
    cfg.service(web::resource("").route(web::post().to(users::add_user)))
        .service(web::resource("/{user_id}").route(web::delete().to(users::remove_user)));
}

pub fn config_sessions(cfg: &mut ServiceConfig) {
    cfg.service(web::resource("").route(web::post().to(sessions::add_session)));
}

pub fn config_web_sockets(cfg: &mut ServiceConfig) {
    cfg.service(web::scope("/lobby").configure(|cfg| {
        cfg.app_data(Data::new(RwLock::new(LobbyData::default())))
            .service(web::resource("").route(web::get().to(lobby::ws)));
    }));
}

pub fn config_app(cfg: &mut ServiceConfig) {
    cfg.service(web::scope("/sessions").configure(config_sessions))
        .service(web::scope("/users").configure(config_users))
        .service(web::scope("/ws").configure(config_web_sockets));
}
