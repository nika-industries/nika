//! API server that handles platform actions for the frontend and CLI.

mod temp_storage_payload;

use axum::{
  extract::{FromRef, Path, State},
  response::IntoResponse,
  routing::{get, post},
  Json, Router,
};
use tasks::Task;

async fn prepare_fetch_payload(
  State(db): State<db::TikvDb>,
  Json((store_name, token_secret)): Json<(String, Option<String>)>,
) -> Result<Json<models::StorageCredentials>, mollusk::InternalApiError> {
  Ok(
    tasks::PrepareFetchPayloadTask {
      store_name,
      token_secret,
    }
    .run(db)
    .await
    .map(Json)?,
  )
}

// async fn get_store_creds_handler(
//   State(db): State<db::DbConnection>,
//   Path(store_name): Path<String>,
// ) -> Result<Json<models::StorageCredentials>, mollusk::InternalApiError>
// {   Ok(
//     tasks::FetchStoreCredsTask { store_name }
//       .run(db)
//       .await
//       .map(Json)?,
//   )
// }

#[tracing::instrument(skip(db, payload))]
async fn naive_upload(
  Path((store_name, path)): Path<(String, String)>,
  State(db): State<db::TikvDb>,
  payload: temp_storage_payload::TempStoragePayload,
) -> impl IntoResponse {
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
  db: db::TikvDb,
}

#[tokio::main]
async fn main() -> miette::Result<()> {
  tracing_subscriber::fmt::init();

  println!(art::ascii_art!("../../media/ascii_logo.png"));

  let state = AppState {
    db: db::DbConnection::new().await?,
  };

  let app = Router::new()
    .route("/naive-upload/:name/*path", post(naive_upload))
    // .route("/creds/:name", get(get_store_creds_handler))
    .route("/fetch_payload", get(prepare_fetch_payload))
    .with_state(state);

  let bind_address = "0.0.0.0:3000";
  let listener = tokio::net::TcpListener::bind(bind_address).await.unwrap();

  tracing::info!("listening on `{bind_address}`");
  axum::serve(listener, app).await.unwrap();

  Ok(())
}
