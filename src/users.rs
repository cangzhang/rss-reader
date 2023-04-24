use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub active: bool,
}

#[derive(Deserialize, Serialize, sqlx::FromRow)]
pub struct CreateUserForm {
    pub name: String,
}