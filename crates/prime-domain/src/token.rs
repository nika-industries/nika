use std::future::Future;

pub use models::Token;
use models::{StrictSlug, TokenRecordId};
use repos::{FetchModelError, TokenRepository};

/// The error type for token verification.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum TokenVerifyError {
  /// The token ID was not found.
  #[error("token ID not found")]
  IdNotFound,
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
  /// Fetch a [`Token`] by its ID.
  fn fetch(
    &self,
    id: TokenRecordId,
  ) -> impl Future<Output = Result<Option<Token>, FetchModelError>> + Send;

  /// Verifies that the supplied token ID and secret are valid and exist.
  fn verify_token_id_and_secret(
    &self,
    id: TokenRecordId,
    secret: StrictSlug,
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
  async fn fetch(
    &self,
    id: TokenRecordId,
  ) -> Result<Option<Token>, FetchModelError> {
    self.token_repo.fetch_model_by_id(id).await
  }

  async fn verify_token_id_and_secret(
    &self,
    id: TokenRecordId,
    secret: StrictSlug,
  ) -> Result<Token, TokenVerifyError> {
    let token = self
      .fetch(id)
      .await
      .map_err(TokenVerifyError::FetchError)?
      .ok_or(TokenVerifyError::IdNotFound)?;

    if token.secret != secret {
      return Err(TokenVerifyError::SecretMismatch);
    }

    Ok(token)
  }
}
