pub mod model;

use std::sync::Once;

use rusqlite::{Connection, Result};
use uuid::Uuid;

use crate::db::model::User;

pub fn create_user(
    conn: &Connection,
    id: Uuid,
    email: &str,
    username: &str,
    password: &[u8],
    salt: &[u8],
) -> Result<usize> {
    init(conn);

    conn.execute(
        "INSERT INTO users (id, email, username, password, salt) VALUES (?1, ?2, ?3, ?4, ?5)",
        (id, email, username, password, salt),
    )
}

pub fn get_user_by_email(conn: &Connection, email: &str) -> Result<User> {
    init(conn);

    conn.query_one(
        "SELECT (id, email, username, password, salt) FROM users WHERE email=?1",
        (email,),
        |row| {
            Ok(User {
                id: row.get(0)?,
                email: row.get(1)?,
                username: row.get(2)?,
                password: row.get(3)?,
                salt: row.get(4)?,
            })
        },
    )
}

pub fn get_user_by_username(conn: &Connection, username: &str) -> Result<User> {
    init(conn);

    conn.query_one(
        "SELECT (id, email, username, password, salt) FROM users WHERE username=?1",
        (username,),
        |row| {
            Ok(User {
                id: row.get(0)?,
                email: row.get(1)?,
                username: row.get(2)?,
                password: row.get(3)?,
                salt: row.get(4)?,
            })
        },
    )
}

pub fn delete_user_by_id(conn: &Connection, id: Uuid) -> Result<usize> {
    init(conn);

    conn.execute("DELETE FROM users WHERE id=?1", (id,))
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
