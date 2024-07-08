use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
};
use storage::ReadError;

#[derive(thiserror::Error, Debug)]
pub enum FetcherError {
  #[error("The store does not exist: {0}")]
  NoMatchingStore(String),
  #[error("SurrealDB error: {0}")]
  SurrealDbStoreRetrievalError(db::SurrealError),
  #[error("An error occured while fetching: {0}")]
  ReadError(#[from] storage::ReadError),
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
