use std::sync::Arc;

use apalis::{
  prelude::{Storage, TaskId},
  redis::RedisStorage,
};
use axum::{
  extract::{FromRef, State},
  response::IntoResponse,
  routing::get,
  Router,
};
use miette::Diagnostic;
use mollusk::{ApiError, RenderApiError};
use tokio::sync::Mutex;

#[derive(thiserror::Error, Diagnostic, Debug)]
pub enum HealthCheckError {
  #[error("failed to access the job storage: {0}")]
  StorageError(#[from] jobs::StorageError),
  #[error("job not found: {0}")]
  JobNotFound(TaskId),
}

impl ApiError for HealthCheckError {
  fn status_code(&self) -> axum::http::StatusCode { todo!() }
  fn slug(&self) -> &'static str { todo!() }
  fn description(&self) -> String { todo!() }
  fn tracing(&self) { todo!() }
}

#[tracing::instrument]
async fn health_check_job(
  State(storage): State<Arc<Mutex<RedisStorage<jobs::HealthCheckJob>>>>,
) -> impl IntoResponse {
  async move {
    let mut storage_lock = storage.lock().await;
    let job_id = storage_lock.push(jobs::HealthCheckJob).await?;
    drop(storage_lock);

    // loop {
    let mut storage_lock = storage.lock().await;
    let job = storage_lock
      .fetch_by_id(&job_id)
      .await?
      .ok_or(HealthCheckError::JobNotFound(job_id))?;
    // let job = job.get::<Context>();

    // }

    Ok::<_, HealthCheckError>(format!("job: {job:?}"))
  }
  .await
  .map_err(|e| format!("{e:?}"))
}

#[derive(FromRef, Clone)]
struct AppState {
  health_check_storage: Arc<Mutex<RedisStorage<jobs::HealthCheckJob>>>,
}

#[tokio::main]
async fn main() -> miette::Result<()> {
  tracing_subscriber::fmt::init();

  println!(art::ascii_art!("../../media/ascii_logo.png"));

  let state = AppState {
    health_check_storage: Arc::new(Mutex::new(jobs::get_storage().await?)),
  };

  let app = Router::new()
    .route("/health", get(health_check_job))
    .with_state(state);

  let bind_address = "0.0.0.0:3000";
  let listener = tokio::net::TcpListener::bind(bind_address).await.unwrap();

  tracing::info!("listening on `{bind_address}`");
  axum::serve(listener, app).await.unwrap();

  Ok(())
}
