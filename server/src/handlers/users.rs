use actix_web::{HttpResponse, Responder, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::session::extract_session;

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

pub async fn add_user(_body: web::Json<UserSignupInfo>) -> impl Responder {
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
