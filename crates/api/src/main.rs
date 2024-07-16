use axum::{
  extract::{FromRef, State},
  response::IntoResponse,
  routing::get,
  Router,
};
use rope::Backend;

async fn health_check_handler(
  State(health_check_tasks): State<rope::RedisBackend<tasks::HealthCheckTask>>,
) -> impl IntoResponse {
  let id = health_check_tasks
    .submit_task(tasks::HealthCheckTask)
    .await
    .unwrap();

  let status = health_check_tasks
    .await_task(id, tokio::time::Duration::from_secs_f32(0.01))
    .await
    .unwrap()
    .unwrap();

  format!("status: {status:?}")
}

#[derive(Clone, FromRef)]
struct AppState {
  health_check_task_backend: rope::RedisBackend<tasks::HealthCheckTask>,
}

#[tokio::main]
async fn main() -> miette::Result<()> {
  tracing_subscriber::fmt::init();

  println!(art::ascii_art!("../../media/ascii_logo.png"));

  let state = AppState {
    health_check_task_backend:
      rope::RedisBackend::<tasks::HealthCheckTask>::new(()).await,
  };

  let app = Router::new()
    .route("/health", get(health_check_handler))
    .with_state(state);

  let bind_address = "0.0.0.0:3000";
  let listener = tokio::net::TcpListener::bind(bind_address).await.unwrap();

  tracing::info!("listening on `{bind_address}`");
  axum::serve(listener, app).await.unwrap();

  Ok(())
}
