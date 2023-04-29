use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, sqlx::FromRow, Debug, Clone)]
pub struct Session {
    pub id: i64,
    pub cookie_id: String,
    pub user_id: i64,
    pub last_active: String,
}