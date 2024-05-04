use std::env;

use axum::{routing::get, Extension, Json, Router};
use serde_json::{json, Value};
use tokio_rusqlite::{params, Connection};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;
    tracing_subscriber::fmt::init();

    // let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("0.0.0.0:{port}");
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let conn = Connection::open(&db_url).await?;

    let app = Router::new()
        .nest("/api", Router::new().route("/ping", get(ping)))
        .layer(Extension(&conn));

    let listener = tokio::net::TcpListener::bind(&server_url).await.unwrap();
    axum::serve(listener, app).await?;
    tracing::debug!("Listening on {}", server_url);

    Ok(())
}

async fn ping(Extension(conn): Extension<&Connection>) -> anyhow::Result<Json<Value>> {
    conn.call(|conn| {
        Ok(conn.execute(
            "INSERT INTO feed (url, name) VALUES (?1, ?2)",
            params![&"https://example.com", &"Example"],
        )?)
    })
    .await?;

    Ok(Json(json!({"message": "Data inserted successfully"})))
}
