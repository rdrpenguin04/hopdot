use actix_web::{
    HttpResponse, Responder,
    web::{self, Data},
};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use check_if_email_exists::{CheckEmailInput, Reachable, check_email};
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
    id: Uuid,
}

#[derive(Serialize)]
pub struct BadRequest {
    status: &'static str,
}

#[derive(Serialize)]
pub struct SqlError {
    status: String,
}

pub async fn add_user(
    auth: Option<BearerAuth>,
    conn: Data<Connection>,
    body: web::Json<UserSignupInfo>,
) -> impl Responder {
    // Basic validity checks
    if body.password.len() < 8 {
        return HttpResponse::BadRequest().json(BadRequest {
            status: "password too short",
        });
    } else if body
        .username
        .contains(|x: char| x.is_whitespace() || x.is_control())
    {
        return HttpResponse::BadRequest().json(BadRequest {
            status: "username contains non-printable characters",
        });
    }

    // Make sure we have a shot of being able to reach the given email
    let email_result = check_email(&CheckEmailInput::new(body.email.clone())).await;
    if email_result.is_reachable == Reachable::Invalid {
        return HttpResponse::BadRequest().json(BadRequest {
            status: "invalid email",
        });
    }

    // Check for a duplicate email or username
    if db::get_user_by_email(&conn, &body.email).is_ok() {
        return HttpResponse::BadRequest().json(BadRequest {
            status: "email in use",
        });
    } else if db::get_user_by_username(&conn, &body.username).is_ok() {
        return HttpResponse::BadRequest().json(BadRequest {
            status: "username in use",
        });
    }

    // Hash the given password
    let argon2 = Argon2::default();
    let salt = SaltString::generate(OsRng);

    let password_hash = argon2
        .hash_password(body.password.as_bytes(), &salt)
        .unwrap();
    let salt = password_hash.salt.unwrap().as_str().as_bytes();
    let password = password_hash.hash.unwrap();

    // If a guest is logged in, reuse the temp ID.
    let id = auth
        .and_then(|x| extract_session(x.token()).ok())
        .and_then(|x| if x.guest { Some(x.user_id) } else { None })
        .unwrap_or_else(Uuid::now_v7);

    match db::create_user(
        &conn,
        id,
        &body.email,
        &body.username,
        password.as_bytes(),
        salt,
    ) {
        Ok(_) => HttpResponse::Created().json(CreateSuccess {
            status: "success",
            id,
        }),
        Err(x) => {
            eprintln!("SQLite Error: {x}");
            HttpResponse::InternalServerError().json(SqlError {
                status: x.to_string(),
            })
        }
    }
}

pub async fn remove_user(
    auth: BearerAuth,
    conn: Data<Connection>,
    user_id: web::Path<Uuid>,
) -> actix_web::Result<impl Responder> {
    let session = extract_session(auth.token())?;
    if session.user_id == *user_id {
        match db::delete_user_by_id(&conn, *user_id) {
            Ok(_) => Ok(HttpResponse::NoContent().finish()),
            Err(x) => {
                eprintln!("SQLite Error: {x}");
                Ok(HttpResponse::InternalServerError().json(SqlError {
                    status: x.to_string(),
                }))
            }
        }
    } else {
        Ok(HttpResponse::Forbidden().finish())
    }
}
