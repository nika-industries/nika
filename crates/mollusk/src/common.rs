use axum::http::StatusCode;
use miette::Diagnostic;
use serde::{Deserialize, Serialize};

use crate::ApiError;

#[derive(thiserror::Error, Diagnostic, Debug, Serialize, Deserialize)]
#[error("The store does not exist: {0:?}")]
pub struct NoMatchingStoreError(String);

impl ApiError for NoMatchingStoreError {
  fn status_code(&self) -> StatusCode { StatusCode::NOT_FOUND }
  fn slug(&self) -> &'static str { "missing-store" }
  fn description(&self) -> String {
    format!("The store {:?} does not exist.", self.0)
  }
  fn tracing(&self) {
    tracing::warn!("requested store does not exist: {:?}", self.0);
  }
}

#[derive(thiserror::Error, Diagnostic, Debug, Serialize, Deserialize)]
#[error("The store requires authentication: {0:?}")]
pub struct UnauthenticatedStoreAccessError(String);

impl ApiError for UnauthenticatedStoreAccessError {
  fn status_code(&self) -> StatusCode { StatusCode::UNAUTHORIZED }
  fn slug(&self) -> &'static str { "unauthenticated-store-access" }
  fn description(&self) -> String {
    format!("The store {:?} requires authentication.", self.0)
  }
  fn tracing(&self) {
    tracing::warn!("requested store requires authentication: {:?}", self.0);
  }
}

#[derive(thiserror::Error, Diagnostic, Debug, Serialize, Deserialize)]
#[error(
  "The given token does not have access to the store {store_name:?}; required \
   permission: \"{permission}\""
)]
pub struct UnauthorizedStoreAccessError {
  store_name: String,
  permission: core_types::StorePermission,
}

impl ApiError for UnauthorizedStoreAccessError {
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
