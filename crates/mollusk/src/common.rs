use axum::http::StatusCode;
use miette::Diagnostic;
use serde::{Deserialize, Serialize};

use crate::MolluskError;

/// An unrecoverable internal error.
#[derive(thiserror::Error, Diagnostic, Debug, Serialize, Deserialize)]
#[error("internal error: {0:?}")]
pub struct InternalError(pub String);

impl MolluskError for InternalError {
  fn status_code(&self) -> StatusCode { StatusCode::INTERNAL_SERVER_ERROR }
  fn slug(&self) -> &'static str { "internal-error" }
  fn description(&self) -> String { "An internal error occurred".to_string() }
  fn tracing(&self) {
    tracing::error!("internal error: {:?}", self);
  }
}

/// An error that occurs when the cache does not exist.
#[derive(thiserror::Error, Diagnostic, Debug, Serialize, Deserialize)]
#[error("The cache does not exist: {0:?}")]
pub struct NoMatchingCacheError(pub String);

impl MolluskError for NoMatchingCacheError {
  fn status_code(&self) -> StatusCode { StatusCode::NOT_FOUND }
  fn slug(&self) -> &'static str { "missing-cache" }
  fn description(&self) -> String {
    format!("The cache {:?} does not exist.", self.0)
  }
  fn tracing(&self) {
    tracing::warn!("requested cache does not exist: {:?}", self.0);
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
  /// The name of the store.
  pub store_name: String,
  /// The required permission.
  pub permission: models::CachePermissionType,
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

/// An error that occurs when the token is malformed.
#[derive(thiserror::Error, Diagnostic, Debug, Serialize, Deserialize)]
#[error("The token is malformed: {token:?}")]
pub struct MalformedTokenSecretError {
  /// The malformed token.
  pub token: String,
}

impl MolluskError for MalformedTokenSecretError {
  fn status_code(&self) -> StatusCode { StatusCode::BAD_REQUEST }
  fn slug(&self) -> &'static str { "malformed-token" }
  fn description(&self) -> String {
    format!("The token {:?} is malformed.", self.token)
  }
  fn tracing(&self) {
    tracing::warn!("malformed token: {:?}", self.token);
  }
}

/// An error that occurs when the token doesn't exist.
#[derive(thiserror::Error, Diagnostic, Debug, Serialize, Deserialize)]
#[error("The supplied token does not exist: {token:?}")]
pub struct NonExistentTokenError {
  /// The non-existent token.
  pub token: String,
}

impl MolluskError for NonExistentTokenError {
  fn status_code(&self) -> StatusCode { StatusCode::FORBIDDEN }
  fn slug(&self) -> &'static str { "non-existent-token" }
  fn description(&self) -> String {
    format!("The supplied token {:?} does not exist.", self.token)
  }
  fn tracing(&self) {
    tracing::warn!("supplied token does not exist: {:?}", self.token);
  }
}

/// An error that occurs when the path is missing.
#[derive(thiserror::Error, Diagnostic, Debug, Serialize, Deserialize)]
#[error("The path is missing: {path:?}")]
pub struct MissingPathError {
  /// The missing path.
  pub path: String,
}

impl MolluskError for MissingPathError {
  fn status_code(&self) -> StatusCode { StatusCode::BAD_REQUEST }
  fn slug(&self) -> &'static str { "missing-path" }
  fn description(&self) -> String {
    format!("The path {:?} is missing.", self.path)
  }
  fn tracing(&self) {
    tracing::warn!("missing path: {:?}", self.path);
  }
}
