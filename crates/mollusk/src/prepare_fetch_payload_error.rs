use axum::http::StatusCode;
use miette::Diagnostic;
use serde::{Deserialize, Serialize};

use crate::{
  common::{
    NoMatchingStoreError, UnauthenticatedStoreAccessError,
    UnauthorizedStoreAccessError,
  },
  ApiError,
};

#[derive(thiserror::Error, Diagnostic, Debug, Serialize, Deserialize)]
pub enum PrepareFetchPayloadError {
  #[error(transparent)]
  NoMatchingStore(#[from] NoMatchingStoreError),
  #[error(transparent)]
  UnauthenticatedStoreAccess(#[from] UnauthenticatedStoreAccessError),
  #[error(transparent)]
  UnauthorizedStoreAccess(#[from] UnauthorizedStoreAccessError),
}

impl ApiError for PrepareFetchPayloadError {
  fn status_code(&self) -> StatusCode {
    match self {
      Self::NoMatchingStore(e) => e.status_code(),
      Self::UnauthenticatedStoreAccess(e) => e.status_code(),
      Self::UnauthorizedStoreAccess(e) => e.status_code(),
    }
  }
  fn slug(&self) -> &'static str {
    match self {
      Self::NoMatchingStore(e) => e.slug(),
      Self::UnauthenticatedStoreAccess(e) => e.slug(),
      Self::UnauthorizedStoreAccess(e) => e.slug(),
    }
  }
  fn description(&self) -> String {
    match self {
      Self::NoMatchingStore(e) => e.description(),
      Self::UnauthenticatedStoreAccess(e) => e.description(),
      Self::UnauthorizedStoreAccess(e) => e.description(),
    }
  }
  fn tracing(&self) {
    match self {
      Self::NoMatchingStore(e) => e.tracing(),
      Self::UnauthenticatedStoreAccess(e) => e.tracing(),
      Self::UnauthorizedStoreAccess(e) => e.tracing(),
    }
  }
}
