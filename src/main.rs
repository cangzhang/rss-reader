use std::env;

use axum::{routing::get, Extension, Json, Router};
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
        .nest(
            "/api",
            Router::new().route("/ping", get(w_data).post(w_data)),
        )
        .layer(Extension(conn.clone()));

    let listener = tokio::net::TcpListener::bind(&server_url).await.unwrap();
    axum::serve(listener, app).await?;
    tracing::debug!("Listening on {}", server_url);

    Ok(())
}

async fn w_data(Extension(conn): Extension<Connection>) -> anyhow::Result<Json<Vec<String>>, ()> {
    let _ = conn
        .call(|conn| {
            Ok(conn.execute(
                "INSERT INTO feed (url, name, created_at) VALUES (?1, ?2, datetime('now'))",
                params![&"https://example.com", &"Example"],
            )?)
        })
        .await;

    Ok(Json(vec![]))
}
