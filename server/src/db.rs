pub mod model;

use std::sync::Once;

use model::User;
use rusqlite::{Connection, Result};
use uuid::Uuid;

pub fn create_user(
    conn: &Connection,
    email: String,
    username: String,
    password: Vec<u8>,
    salt: Vec<u8>,
) -> Result<usize> {
    init(conn);

    let user = User {
        id: Uuid::now_v7(),
        email,
        username,
        password,
        salt,
    };

    conn.execute(
        "INSERT INTO users (id, email, username, password, salt) VALUES (?1, ?2, ?3, ?4, ?5)",
        (user.id, user.email, user.username, user.password, user.salt),
    )
}

pub fn init(conn: &Connection) {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        conn.execute_batch(
            "
            BEGIN;
            CREATE TABLE IF NOT EXISTS users (
                id BLOB PRIMARY KEY,
                email TEXT NOT NULL,
                username TEXT NOT NULL,
                password BLOB NOT NULL,
                salt BLOB NOT NULL
            ) WITHOUT ROWID;
            CREATE UNIQUE INDEX users_by_email ON users(email);
            CREATE UNIQUE INDEX users_by_username ON users(username);
            COMMIT;",
        )
        .unwrap();
    });
}
