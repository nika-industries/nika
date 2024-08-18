use axum::http::StatusCode;
use miette::Diagnostic;
use serde::{Deserialize, Serialize};

use crate::{
  common::{
    NoMatchingStoreError, UnauthenticatedStoreAccessError,
    UnauthorizedStoreAccessError,
  },
  InternalError, MalformedTokenSecretError, MolluskError,
  NonExistentTokenError,
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
  /// The supplied token does not exist.
  #[error(transparent)]
  NonExistentToken(#[from] NonExistentTokenError),
  /// The token secret was malformed.
  #[error(transparent)]
  MalformedTokenSecret(#[from] MalformedTokenSecretError),
  /// Internal error
  #[error(transparent)]
  InternalError(#[from] InternalError),
}

crate::delegate_mollusk_error!(
  PrepareFetchPayloadError,
  NoMatchingStore,
  UnauthenticatedStoreAccess,
  UnauthorizedStoreAccess,
  NonExistentToken,
  MalformedTokenSecret,
  InternalError,
);
