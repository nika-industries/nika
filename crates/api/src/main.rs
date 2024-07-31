//! API server that handles platform actions for the frontend and CLI.

use std::{path::PathBuf, str::FromStr};

use axum::{
  extract::{FromRef, Path, State},
  response::IntoResponse,
  routing::{get, post},
  Json, Router,
};
use rope::{Backend, RedisBackend};
use storage::StorageClientGenerator;
use tasks::{HealthCheckTask, Task};
use tokio_stream::StreamExt;

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

#[tracing::instrument(skip(db, body))]
async fn test_upload(
  State(db): State<db::DbConnection>,
  Path((store_name, path)): Path<(String, String)>,
  body: axum::body::Body,
) -> impl IntoResponse {
  let client = tasks::FetchStoreCredsTask { store_name }
    .run(db)
    .await
    .unwrap()
    .client()
    .await
    .unwrap();

  client
    .write(
      PathBuf::from_str(&path).unwrap().as_path(),
      Box::new(tokio_util::io::StreamReader::new(
        body.into_data_stream().map(|result| {
          result
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))
        }),
      )),
    )
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
    .route("/test-upload/:name/*path", post(test_upload))
    .route("/creds/:name", get(get_store_creds_handler))
    .route("/health", get(health_check_handler))
    .with_state(state);

  let bind_address = "0.0.0.0:3000";
  let listener = tokio::net::TcpListener::bind(bind_address).await.unwrap();

  tracing::info!("listening on `{bind_address}`");
  axum::serve(listener, app).await.unwrap();

  Ok(())
}
