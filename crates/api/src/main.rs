//! API server that handles platform actions for the frontend and CLI.

mod temp_storage_payload;

use axum::{
  extract::{FromRef, Path, State},
  response::IntoResponse,
  routing::{get, post},
  Json, Router,
};
use rope::{Backend, RedisBackend};
use tasks::{HealthCheckTask, Task};

async fn health_check_handler(
  State(health_check_tasks): State<RedisBackend<HealthCheckTask>>,
) -> impl IntoResponse {
  let id = health_check_tasks
    .submit_task(HealthCheckTask)
    .await
    .unwrap();

  let status = health_check_tasks
    .await_task(id, tokio::time::Duration::from_secs_f32(0.01))
    .await
    .unwrap()
    .unwrap();

  format!("status: {status:?}")
}

async fn get_store_creds_handler(
  State(db): State<db::DbConnection>,
  Path(store_name): Path<String>,
) -> Result<Json<core_types::StorageCredentials>, mollusk::InternalApiError> {
  Ok(
    tasks::FetchStoreCredsTask { store_name }
      .run(db)
      .await
      .map(Json)?,
  )
}

#[tracing::instrument(skip(db, payload))]
async fn naive_upload(
  Path((store_name, path)): Path<(String, String)>,
  State(db): State<db::DbConnection>,
  payload: temp_storage_payload::TempStoragePayload,
) -> impl IntoResponse {
  let _creds = tasks::FetchStoreCredsTask {
    store_name: store_name.clone(),
  }
  .run(db.clone())
  .await
  .unwrap();

  let payload_path = payload.upload().await.unwrap();
  tasks::NaiveUploadTask {
    store_name,
    path: path.into(),
    temp_storage_path: payload_path,
  }
  .run(db)
  .await
  .unwrap();
}

#[derive(Clone, FromRef)]
struct AppState {
  db: db::DbConnection,
  health_check_task_backend: RedisBackend<HealthCheckTask>,
}

#[tokio::main]
async fn main() -> miette::Result<()> {
  tracing_subscriber::fmt::init();

  println!(art::ascii_art!("../../media/ascii_logo.png"));

  let state = AppState {
    db: db::DbConnection::new().await?,
    health_check_task_backend: RedisBackend::<HealthCheckTask>::new(()).await,
  };

  let app = Router::new()
    .route("/naive-upload/:name/*path", post(naive_upload))
    .route("/creds/:name", get(get_store_creds_handler))
    .route("/health", get(health_check_handler))
    .with_state(state);

  let bind_address = "0.0.0.0:3000";
  let listener = tokio::net::TcpListener::bind(bind_address).await.unwrap();

  tracing::info!("listening on `{bind_address}`");
  axum::serve(listener, app).await.unwrap();

  Ok(())
}
