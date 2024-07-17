use axum::http::StatusCode;
use miette::Diagnostic;
use storage::ReadError;

#[derive(thiserror::Error, Diagnostic, Debug)]
pub enum FetcherError {
  #[error("The store does not exist: {0}")]
  NoMatchingStore(String),
  #[error("SurrealDB error: {0}")]
  SurrealDbStoreRetrievalError(db::SurrealError),
  #[error("An error occured while fetching: {0}")]
  ReadError(#[from] storage::ReadError),
  #[error("Failed to build the store")]
  StoreInitError(#[diagnostic_source] miette::Report),
}

impl mollusk::ApiError for FetcherError {
  fn status_code(&self) -> StatusCode {
    match self {
      FetcherError::NoMatchingStore(_) => StatusCode::NOT_FOUND,
      FetcherError::SurrealDbStoreRetrievalError(_) => {
        StatusCode::INTERNAL_SERVER_ERROR
      }
      FetcherError::ReadError(ReadError::NotFound(_)) => StatusCode::NOT_FOUND,
      FetcherError::ReadError(ReadError::InvalidPath(_)) => {
        StatusCode::BAD_REQUEST
      }
      FetcherError::ReadError(ReadError::IoError(_)) => {
        StatusCode::INTERNAL_SERVER_ERROR
      }
      FetcherError::StoreInitError(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
  }

  fn slug(&self) -> &'static str {
    match self {
      FetcherError::NoMatchingStore(_) => "missing-store",
      FetcherError::ReadError(ReadError::NotFound(_)) => "missing-path",
      FetcherError::ReadError(ReadError::InvalidPath(_)) => "invalid-path",
      FetcherError::SurrealDbStoreRetrievalError(_)
      | FetcherError::ReadError(ReadError::IoError(_))
      | FetcherError::StoreInitError(_) => "internal-error",
    }
  }

  fn description(&self) -> String {
    match self {
      FetcherError::NoMatchingStore(store_name) => {
        format!("The store \"{store_name}\" does not exist.")
      }
      FetcherError::SurrealDbStoreRetrievalError(e) => {
        format!("An internal error occurred: {e}")
      }
      FetcherError::ReadError(ReadError::NotFound(path)) => {
        format!("The resource at {path:?} does not exist.")
      }
      FetcherError::ReadError(ReadError::InvalidPath(path)) => {
        format!("The requested path \"{path}\" is invalid.")
      }
      FetcherError::ReadError(ReadError::IoError(e)) => {
        format!("An internal error occurred: {e}")
      }
      FetcherError::StoreInitError(e) => {
        format!("Failed to use store: {e}")
      }
    }
  }

  fn tracing(&self) {
    match self {
      FetcherError::NoMatchingStore(_) => {
        tracing::warn!("asked to fetch from non-existent store");
      }
      FetcherError::SurrealDbStoreRetrievalError(e) => {
        tracing::error!("failed to retrieve store from surrealdb: {e}");
      }
      FetcherError::ReadError(ReadError::NotFound(_)) => {
        tracing::warn!("asked to fetch missing path");
      }
      FetcherError::ReadError(ReadError::InvalidPath(_)) => {
        tracing::warn!("asked to fetch invalid path");
      }
      FetcherError::ReadError(ReadError::IoError(e)) => {
        tracing::warn!("failed to fetch path due to `IoError`: {e}");
      }
      FetcherError::StoreInitError(e) => {
        tracing::error!("failed to init store: {e}");
      }
    }
  }
}
