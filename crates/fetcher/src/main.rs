use std::{ops::Deref, path::PathBuf, str::FromStr, sync::Arc};

use axum::{
  body::Body,
  extract::{FromRef, Path, State},
  http::StatusCode,
  response::{IntoResponse, Response},
  routing::get,
  Router,
};
use miette::IntoDiagnostic;
use storage::ReadError;

#[tracing::instrument(skip(client))]
async fn fetch_handler(
  State(client): State<Arc<storage::DynStorageClient>>,
  Path(path): Path<String>,
) -> Response {
  fetch_path_from_client(client, path).await.into_response()
}

#[derive(thiserror::Error, Debug)]
#[error("An error occured while fetching: {0}")]
struct FetcherError(#[from] storage::ReadError);

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
    match self.0 {
      ReadError::NotFound(path) => {
        tracing::warn!("asked to fetch missing path");
        (
          StatusCode::NOT_FOUND,
          format!("The resource at {path:?} does not exist."),
        )
          .into_response()
      }
      ReadError::InvalidPath(path) => {
        tracing::warn!("asked to fetch invalid path");
        (
          StatusCode::BAD_REQUEST,
          format!("Your requested path \"{path}\" is invalid"),
        )
          .into_response()
      }
      ReadError::IoError(e) => {
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
  db:             db::DbConnection,
  storage_client: Arc<storage::DynStorageClient>,
}

#[tokio::main]
async fn main() -> miette::Result<()> {
  tracing_subscriber::fmt::init();

  println!("");
  for line in art::ascii_art!("../../media/ascii_logo.png").lines() {
    println!("{}", line);
  }
  println!("");

  let client = storage::StorageCredentials::Local(
    PathBuf::from_str("/tmp/nika").into_diagnostic()?,
  )
  .client()
  .await;
  let app_state = AppState {
    storage_client: Arc::new(client),
    db:             db::DbConnection::new().await?,
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
