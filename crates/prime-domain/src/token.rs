use std::{future::Future, str::FromStr};

pub use models::Token;
use repos::{FetchModelError, TokenRepository};

/// The error type for token verification.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum TokenVerifyError {
  /// The token ID is invalid.
  #[error("invalid token ID")]
  InvalidId,
  /// The token ID was not found.
  #[error("token ID not found")]
  IdNotFound,
  /// The token secret is invalid.
  #[error("invalid token secret")]
  InvalidSecret,
  /// The token secret does not match the expected secret.
  #[error("token secret mismatch")]
  SecretMismatch,
  /// An error occurred while fetching the token.
  #[error("error fetching token")]
  #[diagnostic_source]
  FetchError(FetchModelError),
}

/// The definition for the [`Token`] domain model service.
pub trait TokenService: Clone + Send + Sync + 'static {
  /// Verifies that the supplied token ID and secret are valid and exist.
  fn verify_token_id_and_secret(
    &self,
    token_id: String,
    token_secret: String,
  ) -> impl Future<Output = Result<Token, TokenVerifyError>>;
}

/// Canonical service for the [`Token`] domain model.
pub struct TokenServiceCanonical<R: TokenRepository> {
  token_repo: R,
}

impl<R: TokenRepository> Clone for TokenServiceCanonical<R> {
  fn clone(&self) -> Self {
    Self {
      token_repo: self.token_repo.clone(),
    }
  }
}

impl<R: TokenRepository> TokenServiceCanonical<R> {
  /// Create a new instance of the canonical [`Token`] service.
  pub fn new(token_repo: R) -> Self { Self { token_repo } }
}

impl<R: TokenRepository> TokenService for TokenServiceCanonical<R> {
  async fn verify_token_id_and_secret(
    &self,
    token_id: String,
    token_secret: String,
  ) -> Result<Token, TokenVerifyError> {
    let token_id = models::TokenRecordId::from_str(&token_id)
      .map_err(|_| TokenVerifyError::InvalidId)?;

    let token = self
      .token_repo
      .fetch_model_by_id(token_id)
      .await
      .map_err(TokenVerifyError::FetchError)?;

    let token = token.ok_or(TokenVerifyError::IdNotFound)?;

    let token_secret_slug = models::StrictSlug::new(token_secret.clone());
    if token_secret_slug.clone().into_inner() != token_secret {
      return Err(TokenVerifyError::InvalidSecret);
    }

    if token.secret != token_secret_slug {
      return Err(TokenVerifyError::SecretMismatch);
    }

    Ok(token)
  }
}
