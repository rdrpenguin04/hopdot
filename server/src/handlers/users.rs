use actix_web::{
    HttpResponse, Responder,
    web::{self, Data},
};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{db, session::extract_session};

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct UserSignupInfo {
    email: String,
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct CreateSuccess {
    status: &'static str,
}

pub async fn add_user(conn: Data<Connection>, _body: web::Json<UserSignupInfo>) -> impl Responder {
    // TODO: call db::create_user and check result
    db::init(&conn);
    HttpResponse::Created().json(CreateSuccess { status: "success" })
}

pub async fn remove_user(
    auth: BearerAuth,
    user_id: web::Path<Uuid>,
) -> actix_web::Result<impl Responder> {
    let session = extract_session(auth.token())?;
    if session.user_id == user_id.into_inner() {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Ok(HttpResponse::Forbidden().finish())
    }
}
