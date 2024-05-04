use std::{env, net::SocketAddr};

use axum::{
    routing::{get, post},
    Extension, Json, Router,
};
use serde_json::{json, Value};

async fn json() -> Json<Value> {
    Json(json!({ "data": 42 }))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;
    tracing_subscriber::fmt::init();

    // let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("0.0.0.0:{port}");

    let app = Router::new().nest("/api", Router::new().route("/ping", get(json)));

    let listener = tokio::net::TcpListener::bind(&server_url).await.unwrap();
    axum::serve(listener, app).await?;
    tracing::debug!("Listening on {}", server_url);

    Ok(())
}
