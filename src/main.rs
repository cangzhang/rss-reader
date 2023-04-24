use std::{env, net::SocketAddr};

use axum::{
    routing::{get, post},
    Extension, Json, Router,
};
use serde_json::{json, Value};
use sqlx::SqlitePool;

mod controllers;
mod errors;
mod models;

async fn json() -> Json<Value> {
    Json(json!({ "data": 42 }))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let db_pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    let app = Router::new()
        .route("/api/ping", get(json))
        .route("/api/user", post(controllers::users::create_user))
        .layer(Extension(db_pool));
    let addr = SocketAddr::from(([127, 0, 0, 1], 5050));
    tracing::debug!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
