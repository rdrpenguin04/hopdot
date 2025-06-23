use uuid::Uuid;

pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password: Vec<u8>,
    pub salt: Vec<u8>,
}
