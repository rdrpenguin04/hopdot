use actix_web::{
    HttpResponse,
    web::{self, Data},
};
use argon2::{
    ARGON2ID_IDENT, Argon2, PasswordHash, PasswordVerifier,
    password_hash::{self, Output, ParamsString},
};
use common::GUEST_USERNAME_SENTINEL;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    db,
    session::{SessionData, SessionError, new_session},
};

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

#[derive(Serialize)]
pub struct BadRequest {
    status: &'static str,
}

#[derive(Serialize)]
pub struct BadPassword {
    status: &'static str,
}

#[derive(Serialize)]
pub struct SqlError {
    status: String,
}

pub async fn add_session(
    conn: Data<Connection>,
    body: web::Json<UserLoginInfo>,
) -> Result<HttpResponse, SessionError> {
    let session_data = if body.username == GUEST_USERNAME_SENTINEL {
        SessionData {
            user_id: Uuid::now_v7(),
            guest: true,
        }
    } else {
        let user = match db::get_user_by_username(&conn, &body.username) {
            // TODO: Special-case this error to clean up the API
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                return Ok(HttpResponse::BadRequest().json(BadRequest {
                    status: "user does not exist",
                }));
            }
            Err(x) => {
                eprintln!("SQLite Error: {x}");
                return Ok(HttpResponse::InternalServerError().json(SqlError {
                    status: x.to_string(),
                }));
            }
            Ok(user) => user,
        };
        let existing_hash = PasswordHash {
            algorithm: ARGON2ID_IDENT,
            version: Some(19),
            params: ParamsString::try_from(argon2::Params::default()).unwrap(),
            salt: Some(str::from_utf8(&user.salt).unwrap().try_into().unwrap()),
            hash: Some(Output::try_from(&*user.password).unwrap()),
        };
        match Argon2::default().verify_password(body.password.as_bytes(), &existing_hash) {
            Ok(()) => SessionData {
                user_id: user.id,
                guest: false,
            },
            Err(password_hash::Error::Password) => {
                return Ok(HttpResponse::Forbidden().json(BadPassword {
                    status: "incorrecct password",
                }));
            }
            Err(x) => {
                eprintln!("Verification Error: {x}");
                return Ok(HttpResponse::InternalServerError().json(SqlError {
                    status: x.to_string(),
                }));
            }
        }
    };
    Ok(HttpResponse::Created().json(SessionCreateSuccess {
        status: "success",
        session: new_session(session_data)?,
    }))
}
