use axum::{routing::get, Json, Router};
use serde_json::{json, Value};

async fn json() -> Json<Value> {
    Json(json!({ "data": 42 }))
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(json));

    axum::Server::bind(&"0.0.0.0:5050".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
