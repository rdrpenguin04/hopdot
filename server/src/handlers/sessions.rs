use actix_web::{Responder, web};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::session::{SessionData, new_session};

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct UserLoginInfo {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct SessionCreateSuccess {
    status: &'static str,
    session: String,
}

pub async fn add_session(_body: web::Json<UserLoginInfo>) -> impl Responder {
    // All temporary data
    let session_data = SessionData {
        user_id: Uuid::now_v7(),
        guest: false,
    };
    new_session(session_data)
}
