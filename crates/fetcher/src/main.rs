use std::{ops::Deref, path::PathBuf, str::FromStr};

use axum::{
  body::Body,
  extract::{FromRef, Path, State},
  http::StatusCode,
  response::{IntoResponse, Response},
  routing::get,
  Router,
};
use storage::{ReadError, StorageClientGenerator};

#[tracing::instrument(skip(db))]
async fn fetch_handler(
  State(db): State<db::DbConnection>,
  Path((store_name, path)): Path<(String, String)>,
) -> Result<Response, FetcherError> {
  let store = db
    .fetch_store_by_name(&store_name)
    .await
    .map_err(FetcherError::SurrealDbStoreRetrievalError)?
    .ok_or_else(|| FetcherError::NoMatchingStore(store_name))?;

  let client = store.config.client().await;

  let response = fetch_path_from_client(&client, path).await?;
  Ok(response)
}

#[derive(thiserror::Error, Debug)]
enum FetcherError {
  #[error("The store does not exist: {0}")]
  NoMatchingStore(String),
  #[error("SurrealDB error: {0}")]
  SurrealDbStoreRetrievalError(db::SurrealError),
  #[error("An error occured while fetching: {0}")]
  ReadError(#[from] storage::ReadError),
}

#[tracing::instrument(skip(client))]
async fn fetch_path_from_client(
  client: impl Deref<Target = storage::DynStorageClient>,
  path: String,
) -> Result<Response, FetcherError> {
  let Ok(path) = PathBuf::from_str(&path) else {
    tracing::warn!("asked to fetch invalid path");
    Err(storage::ReadError::InvalidPath(path))?
  };

  let reader = client.read(&path).await?;
  let stream = tokio_util::io::ReaderStream::new(reader);

  tracing::info!("fetching path");
  Ok(Body::from_stream(stream).into_response())
}

impl IntoResponse for FetcherError {
  fn into_response(self) -> Response {
    match self {
      FetcherError::NoMatchingStore(store_name) => {
        tracing::warn!("asked to fetch from non-existent store");
        (
          StatusCode::NOT_FOUND,
          format!("The store \"{store_name}\" does not exist."),
        )
          .into_response()
      }
      FetcherError::SurrealDbStoreRetrievalError(e) => {
        tracing::error!("failed to retrieve store from surrealdb: {e}");
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          format!("An internal error occurred: {e}"),
        )
          .into_response()
      }
      FetcherError::ReadError(ReadError::NotFound(path)) => {
        tracing::warn!("asked to fetch missing path");
        (
          StatusCode::NOT_FOUND,
          format!("The resource at {path:?} does not exist."),
        )
          .into_response()
      }
      FetcherError::ReadError(ReadError::InvalidPath(path)) => {
        tracing::warn!("asked to fetch invalid path");
        (
          StatusCode::BAD_REQUEST,
          format!("Your requested path \"{path}\" is invalid"),
        )
          .into_response()
      }
      FetcherError::ReadError(ReadError::IoError(e)) => {
        tracing::warn!("failed to fetch path: {e}");
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          format!("An internal error occurred: {e}"),
        )
          .into_response()
      }
    }
  }
}

#[derive(Clone, FromRef)]
struct AppState {
  db: db::DbConnection,
}

#[tokio::main]
async fn main() -> miette::Result<()> {
  tracing_subscriber::fmt::init();

  println!();
  for line in art::ascii_art!("../../media/ascii_logo.png").lines() {
    println!("{}", line);
  }
  println!();

  let app_state = AppState {
    db: db::DbConnection::new().await?,
  };
  let app = Router::new()
    .route("/:name/*path", get(fetch_handler))
    .with_state(app_state);

  let bind_address = "0.0.0.0:3000";
  let listener = tokio::net::TcpListener::bind(bind_address).await.unwrap();

  tracing::info!("listening on `{bind_address}`");
  axum::serve(listener, app).await.unwrap();

  Ok(())
}
