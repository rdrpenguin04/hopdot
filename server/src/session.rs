use std::{fmt::Display, sync::LazyLock};

use actix_web::{HttpResponse, ResponseError};
use jws::{
    compact::{decode_verify, encode_sign},
    hmac::{HmacVerifier, Hs256Signer},
    json_object,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct SessionData {
    pub user_id: Uuid, // uuidv7
    pub guest: bool,
}

#[derive(Debug)]
pub struct SessionError(String);

impl Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl ResponseError for SessionError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::Unauthorized().body(self.0.clone())
    }
}

impl From<String> for SessionError {
    fn from(value: String) -> Self {
        Self(value)
    }
}

const SECRET: LazyLock<Vec<u8>> = LazyLock::new(|| b"temp key".into());

pub fn new_session(session_data: SessionData) -> Result<String, SessionError> {
    let header = json_object! {
        "alg": "HS256",
    };
    let mut payload = Vec::new();
    serde_json::to_writer(&mut payload, &session_data).map_err(|e| e.to_string())?;
    let encoded =
        encode_sign(header, &payload, &Hs256Signer::new(&*SECRET)).map_err(|e| e.to_string())?;
    Ok(encoded.into_data())
}

pub fn extract_session(token: &str) -> Result<SessionData, SessionError> {
    let decoded =
        decode_verify(token.as_bytes(), &HmacVerifier::new(&*SECRET)).map_err(|e| e.to_string())?;
    serde_json::from_slice(&decoded.payload).map_err(|e| e.to_string().into())
}
