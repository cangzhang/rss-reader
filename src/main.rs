pub mod errors;
pub mod users;

use std::{env, net::SocketAddr};

use axum::{
    http::StatusCode,
    routing::{get, post},
    Extension, Json, Router,
};
use serde_json::{json, Value};
use sqlx::SqlitePool;

async fn json() -> Json<Value> {
    Json(json!({ "data": 42 }))
}

async fn create_user(
    Json(form): Json<users::CreateUserForm>,
    Extension(db_pool): Extension<SqlitePool>,
) -> Result<(StatusCode, Json<users::User>), errors::CustomError> {
    let user = sqlx::query_as!(
        users::User,
        r#"INSERT INTO users (name, active) VALUES(?, 1)"#,
        form.name
    )
    .execute(&db_pool)
    .await
    .map_err(|_| errors::CustomError::InternalServerError)?;

    let user_id = user.last_insert_rowid();
    let user = sqlx::query_as!(
        users::User,
        r#"SELECT id, name, active FROM users where id = ?"#,
        user_id
    )
    .fetch_one(&db_pool)
    .await
    .map_err(|_| errors::CustomError::InternalServerError)?;

    Ok((StatusCode::CREATED, Json(user)))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let db_pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    let app = Router::new()
        .route("/", get(json))
        .route("/api/user", post(create_user))
        .layer(Extension(db_pool));
    let addr = SocketAddr::from(([127, 0, 0, 1], 5050));
    tracing::debug!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
