use axum::http::StatusCode;
use miette::Diagnostic;
use mollusk::CredsFetchingError;
use storage::ReadError;

#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error, Diagnostic, Debug)]
pub enum FetcherError {
  #[error("Error fetching credentials: {0}")]
  CredsFetchingError(#[from] CredsFetchingError),
  #[error("An error occured while fetching: {0}")]
  ReadError(#[from] storage::ReadError),
  #[error("Failed to build the store")]
  StoreInitError(#[diagnostic_source] miette::Report),
}

impl mollusk::ApiError for FetcherError {
  fn status_code(&self) -> StatusCode {
    match self {
      FetcherError::CredsFetchingError(
        CredsFetchingError::NoMatchingStore(_),
      )
      | FetcherError::ReadError(ReadError::NotFound(_)) => {
        StatusCode::NOT_FOUND
      }
      FetcherError::CredsFetchingError(
        CredsFetchingError::SurrealDbStoreRetrievalError(_),
      )
      | FetcherError::StoreInitError(_)
      | FetcherError::ReadError(ReadError::IoError(_))
      | FetcherError::CredsFetchingError(CredsFetchingError::StoreInitError(
        _,
      ))
      | FetcherError::CredsFetchingError(
        CredsFetchingError::TempStorageCredsError(_),
      ) => StatusCode::INTERNAL_SERVER_ERROR,
      FetcherError::ReadError(ReadError::InvalidPath(_)) => {
        StatusCode::BAD_REQUEST
      }
    }
  }

  fn slug(&self) -> &'static str {
    match self {
      FetcherError::CredsFetchingError(
        CredsFetchingError::NoMatchingStore(_),
      ) => "missing-store",
      FetcherError::ReadError(ReadError::NotFound(_)) => "missing-path",
      FetcherError::ReadError(ReadError::InvalidPath(_)) => "invalid-path",
      FetcherError::CredsFetchingError(
        CredsFetchingError::SurrealDbStoreRetrievalError(_),
      )
      | FetcherError::ReadError(ReadError::IoError(_))
      | FetcherError::CredsFetchingError(CredsFetchingError::StoreInitError(
        _,
      ))
      | FetcherError::CredsFetchingError(
        CredsFetchingError::TempStorageCredsError(_),
      )
      | FetcherError::StoreInitError(_) => "internal-error",
    }
  }

  fn description(&self) -> String {
    match self {
      FetcherError::CredsFetchingError(
        CredsFetchingError::NoMatchingStore(store_name),
      ) => format!("The store \"{store_name}\" does not exist."),
      FetcherError::CredsFetchingError(
        CredsFetchingError::SurrealDbStoreRetrievalError(e),
      ) => format!("An internal error occurred: {e}"),
      FetcherError::CredsFetchingError(
        CredsFetchingError::TempStorageCredsError(e),
      ) => format!("An internal error occurred: {e}"),
      FetcherError::CredsFetchingError(CredsFetchingError::StoreInitError(
        _,
      ))
      | FetcherError::StoreInitError(_) => "Failed to use store".to_string(),
      FetcherError::ReadError(ReadError::NotFound(path)) => {
        format!("The resource at {path:?} does not exist.")
      }
      FetcherError::ReadError(ReadError::InvalidPath(path)) => {
        format!("The requested path \"{path}\" is invalid.")
      }
      FetcherError::ReadError(ReadError::IoError(e)) => {
        format!("An internal error occurred: {e}")
      }
    }
  }

  fn tracing(&self) {
    match self {
      FetcherError::CredsFetchingError(
        CredsFetchingError::NoMatchingStore(_),
      ) => {
        tracing::warn!("asked to fetch from non-existent store");
      }
      FetcherError::CredsFetchingError(
        CredsFetchingError::SurrealDbStoreRetrievalError(e),
      ) => {
        tracing::error!("failed to retrieve store from surrealdb: {e}");
      }
      FetcherError::CredsFetchingError(
        CredsFetchingError::TempStorageCredsError(e),
      ) => {
        tracing::error!("failed to fetch temp storage creds: {e}");
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
      FetcherError::CredsFetchingError(CredsFetchingError::StoreInitError(
        e,
      )) => {
        tracing::error!("failed to init store: {e}");
      }
      FetcherError::StoreInitError(e) => {
        tracing::error!("failed to init store: {e}");
      }
    }
  }
}
