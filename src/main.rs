use std::{env, net::SocketAddr};

use axum::{
    routing::{get, post},
    Extension, Json, Router,
};
use serde_json::{json, Value};
use sqlx::SqlitePool;
use tower_cookies::CookieManagerLayer;

mod controllers;
mod errors;
mod models;
mod middlewares;

async fn json() -> Json<Value> {
    Json(json!({ "data": 42 }))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;
    tracing_subscriber::fmt::init();

    let db_pool: SqlitePool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    let app = Router::new()
        .nest(
            "/api",
            Router::new()
                .route("/ping", get(json))
                .route("/list", get(controllers::users::list))
                .route("/user", post(controllers::users::create_user))
                .route("/login", post(controllers::users::login)),
        )
        // .layer(middlewares::auth::SessionLayer)
        .layer(Extension(db_pool))
        .layer(CookieManagerLayer::new());

    let addr = SocketAddr::from(([127, 0, 0, 1], 5050));
    tracing::debug!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
