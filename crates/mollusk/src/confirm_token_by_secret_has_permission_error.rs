use http::StatusCode;
use miette::Diagnostic;
use serde::{Deserialize, Serialize};

use crate::{
  InternalError, MalformedTokenSecretError, MolluskError, NonExistentTokenError,
};

/// An error that occurs while confirming a token (by secret) has a permission.
#[derive(thiserror::Error, Diagnostic, Debug, Serialize, Deserialize)]
pub enum ConfirmTokenBySecretHasPermissionError {
  /// The token does not exist.
  #[error(transparent)]
  NonExistentToken(#[from] NonExistentTokenError),
  /// The token is malformed.
  #[error(transparent)]
  MalformedTokenSecret(#[from] MalformedTokenSecretError),
  /// Internal error
  #[error(transparent)]
  InternalError(#[from] InternalError),
}

crate::delegate_mollusk_error!(
  ConfirmTokenBySecretHasPermissionError,
  NonExistentToken,
  MalformedTokenSecret,
  InternalError
);
