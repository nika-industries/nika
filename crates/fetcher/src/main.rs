use axum::{routing::get, Router};

async fn hello_world_handler() -> &'static str { "Hello, World!" }

#[tokio::main]
async fn main() {
  let app = Router::new().route("/", get(hello_world_handler));

  let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
  axum::serve(listener, app).await.unwrap();
}
