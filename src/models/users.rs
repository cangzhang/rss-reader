use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, sqlx::FromRow, Debug, Clone)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub active: bool,
    pub password_hash: String,
}

#[derive(Deserialize, Serialize, sqlx::FromRow)]
pub struct CreateUserForm {
    pub name: String,
    pub password: String,
}