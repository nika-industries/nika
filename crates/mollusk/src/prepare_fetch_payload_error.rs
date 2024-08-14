use axum::http::StatusCode;
use miette::Diagnostic;
use serde::{Deserialize, Serialize};

use crate::{
  common::{
    NoMatchingStoreError, UnauthenticatedStoreAccessError,
    UnauthorizedStoreAccessError,
  },
  InternalError, MolluskError,
};

/// An error that occurs when preparing to fetch a payload.
#[derive(thiserror::Error, Diagnostic, Debug, Serialize, Deserialize)]
pub enum PrepareFetchPayloadError {
  /// No matching store was found.
  #[error(transparent)]
  NoMatchingStore(#[from] NoMatchingStoreError),
  /// The store access was unauthenticated (no token supplied).
  #[error(transparent)]
  UnauthenticatedStoreAccess(#[from] UnauthenticatedStoreAccessError),
  /// The store access was unauthorized (token supplied but insufficient).
  #[error(transparent)]
  UnauthorizedStoreAccess(#[from] UnauthorizedStoreAccessError),
  /// Internal error
  #[error(transparent)]
  InternalError(#[from] InternalError),
}

impl MolluskError for PrepareFetchPayloadError {
  fn status_code(&self) -> StatusCode {
    match self {
      Self::NoMatchingStore(e) => e.status_code(),
      Self::UnauthenticatedStoreAccess(e) => e.status_code(),
      Self::UnauthorizedStoreAccess(e) => e.status_code(),
      Self::InternalError(e) => e.status_code(),
    }
  }
  fn slug(&self) -> &'static str {
    match self {
      Self::NoMatchingStore(e) => e.slug(),
      Self::UnauthenticatedStoreAccess(e) => e.slug(),
      Self::UnauthorizedStoreAccess(e) => e.slug(),
      Self::InternalError(e) => e.slug(),
    }
  }
  fn description(&self) -> String {
    match self {
      Self::NoMatchingStore(e) => e.description(),
      Self::UnauthenticatedStoreAccess(e) => e.description(),
      Self::UnauthorizedStoreAccess(e) => e.description(),
      Self::InternalError(e) => e.description(),
    }
  }
  fn tracing(&self) {
    match self {
      Self::NoMatchingStore(e) => e.tracing(),
      Self::UnauthenticatedStoreAccess(e) => e.tracing(),
      Self::UnauthorizedStoreAccess(e) => e.tracing(),
      Self::InternalError(e) => e.tracing(),
    }
  }
}
