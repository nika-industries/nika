use std::{path::PathBuf, str::FromStr, sync::Arc};

use axum::{
  body::Body,
  extract::{FromRef, Path, State},
  http::StatusCode,
  response::{IntoResponse, Response},
  routing::get,
  Router,
};
use storage::ReadError;

#[tracing::instrument(skip(client))]
async fn fetch_handler(
  State(client): State<Arc<storage::DynStorageClient>>,
  Path(path): Path<String>,
) -> Response {
  let Ok(path) = PathBuf::from_str(&path) else {
    tracing::warn!("asked to fetch invalid path");
    return (
      StatusCode::BAD_REQUEST,
      "Your requested path could not be parsed as a path.",
    )
      .into_response();
  };

  let reader = match client.read(&path).await {
    Ok(r) => r,
    Err(ReadError::NotFound(_)) => {
      tracing::warn!("asked to fetch missing path");
      return (
        StatusCode::NOT_FOUND,
        format!("The resource at the path {path:?} doesn't exist"),
      )
        .into_response();
    }
    Err(ReadError::InvalidPath(_)) => {
      tracing::warn!("asked to fetch invalid path");
      return (StatusCode::BAD_REQUEST, "Your requested path is invalid")
        .into_response();
    }
    Err(ReadError::IoError(e)) => {
      tracing::error!("failed to fetch path: {e}");
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("An internal error occurred: {e}"),
      )
        .into_response();
    }
  };

  let stream = tokio_util::io::ReaderStream::new(reader);
  tracing::info!("fetching path");
  Body::from_stream(stream).into_response()
}

#[derive(Clone, FromRef)]
struct AppState {
  storage_client: Arc<storage::DynStorageClient>,
}

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();

  let client =
    storage::StorageCredentials::Local(PathBuf::from_str("/tmp/nika").unwrap())
      .client()
      .await;
  let app_state = AppState {
    storage_client: Arc::new(client),
  };
  let app = Router::new()
    .route("/*path", get(fetch_handler))
    .with_state(app_state);

  let bind_address = "0.0.0.0:3000";
  let listener = tokio::net::TcpListener::bind(bind_address).await.unwrap();

  tracing::info!("listening on `{bind_address}`");
  axum::serve(listener, app).await.unwrap();
}
