use axum::http::StatusCode;
use miette::Diagnostic;
use serde::{Deserialize, Serialize};

use crate::MolluskError;

/// An unrecoverable internal error.
#[derive(thiserror::Error, Diagnostic, Debug, Serialize, Deserialize)]
pub enum InternalError {
  /// Temp storage credentials could not be fetched.
  #[error("Failed to fetch temp storage credentials")]
  TempStorageCredsError(String),
  /// An error occurred while connecting to surreal.
  #[error("SurrealDB connection error: {0}")]
  SurrealDbConnectionError(String),
  /// An error occurred while querying surreal.
  #[error("SurrealDB query error: {0}")]
  SurrealDbQueryError(String),
}

impl MolluskError for InternalError {
  fn status_code(&self) -> StatusCode { StatusCode::INTERNAL_SERVER_ERROR }
  fn slug(&self) -> &'static str { "internal-error" }
  fn description(&self) -> String { "An internal error occurred".to_string() }
  fn tracing(&self) {
    tracing::error!("internal error: {:?}", self);
  }
}

/// An error that occurs when the store does not exist.
#[derive(thiserror::Error, Diagnostic, Debug, Serialize, Deserialize)]
#[error("The store does not exist: {0:?}")]
pub struct NoMatchingStoreError(pub String);

impl MolluskError for NoMatchingStoreError {
  fn status_code(&self) -> StatusCode { StatusCode::NOT_FOUND }
  fn slug(&self) -> &'static str { "missing-store" }
  fn description(&self) -> String {
    format!("The store {:?} does not exist.", self.0)
  }
  fn tracing(&self) {
    tracing::warn!("requested store does not exist: {:?}", self.0);
  }
}

/// An error that occurs when the store requires authentication but no token was
/// provided.
#[derive(thiserror::Error, Diagnostic, Debug, Serialize, Deserialize)]
#[error("The store requires authentication: {0:?}")]
pub struct UnauthenticatedStoreAccessError(pub String);

impl MolluskError for UnauthenticatedStoreAccessError {
  fn status_code(&self) -> StatusCode { StatusCode::UNAUTHORIZED }
  fn slug(&self) -> &'static str { "unauthenticated-store-access" }
  fn description(&self) -> String {
    format!("The store {:?} requires authentication.", self.0)
  }
  fn tracing(&self) {
    tracing::warn!("requested store requires authentication: {:?}", self.0);
  }
}

/// An error that occurs when the token does not have the requested access to
/// the store.
#[derive(thiserror::Error, Diagnostic, Debug, Serialize, Deserialize)]
#[error(
  "The given token does not have access to the store {store_name:?}; required \
   permission: \"{permission}\""
)]
pub struct UnauthorizedStoreAccessError {
  store_name: String,
  permission: core_types::StorePermission,
}

impl MolluskError for UnauthorizedStoreAccessError {
  fn status_code(&self) -> StatusCode { StatusCode::FORBIDDEN }
  fn slug(&self) -> &'static str { "unauthorized-store-access" }
  fn description(&self) -> String {
    format!(
      "The given token does not have access to the store {:?}; required \
       permission: {:?}",
      self.store_name, self.permission
    )
  }
  fn tracing(&self) {
    tracing::warn!(
      "access to requested store {:?} is unauthorized: requires {:?}",
      self.store_name,
      self.permission
    );
  }
}
