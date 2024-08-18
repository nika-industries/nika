use axum::http::StatusCode;
use miette::Diagnostic;
use serde::{Deserialize, Serialize};

use crate::MolluskError;

/// Error for retrieving storage credentials.
#[derive(thiserror::Error, Diagnostic, Debug, Serialize, Deserialize)]
pub enum CredsFetchingError {
  /// The store does not exist.
  #[error("The store does not exist: {0}")]
  NoMatchingStore(String),
  /// An error occurred while building the store credentials.
  #[error("Failed to build the store")]
  StoreInitError(String),
  /// Temp storage credentials could not be fetched.
  #[error("Failed to fetch temp storage credentials")]
  TempStorageCredsError(String),
  /// An error occurred while fetching the store from surreal.
  #[error("SurrealDB error: {0}")]
  SurrealDbStoreRetrievalError(String),
}

impl MolluskError for CredsFetchingError {
  fn status_code(&self) -> StatusCode {
    match self {
      CredsFetchingError::NoMatchingStore(_) => StatusCode::NOT_FOUND,
      CredsFetchingError::SurrealDbStoreRetrievalError(_)
      | CredsFetchingError::TempStorageCredsError(_)
      | CredsFetchingError::StoreInitError(_) => {
        StatusCode::INTERNAL_SERVER_ERROR
      }
    }
  }

  fn slug(&self) -> &'static str {
    match self {
      CredsFetchingError::NoMatchingStore(_) => "missing-store",
      CredsFetchingError::SurrealDbStoreRetrievalError(_)
      | CredsFetchingError::TempStorageCredsError(_)
      | CredsFetchingError::StoreInitError(_) => "internal-error",
    }
  }

  fn description(&self) -> String {
    match self {
      CredsFetchingError::NoMatchingStore(store_name) => {
        format!("The store \"{store_name}\" does not exist.")
      }
      CredsFetchingError::TempStorageCredsError(e) => {
        format!("An internal error occurred: {e}")
      }
      CredsFetchingError::SurrealDbStoreRetrievalError(e) => {
        format!("An internal error occurred: {e}")
      }
      CredsFetchingError::StoreInitError(e) => {
        format!("Failed to use store: {e}")
      }
    }
  }

  fn tracing(&self) {
    match self {
      CredsFetchingError::NoMatchingStore(store_name) => {
        tracing::warn!(
          "asked to fetch credentials for non-existent store: {store_name:?}"
        );
      }
      CredsFetchingError::TempStorageCredsError(e) => {
        tracing::error!("failed to fetch temp storage credentials: {e}");
      }
      CredsFetchingError::SurrealDbStoreRetrievalError(e) => {
        tracing::error!("failed to retrieve store from surrealdb: {e}");
      }
      CredsFetchingError::StoreInitError(e) => {
        tracing::error!("failed to init store: {e}");
      }
    }
  }
}
