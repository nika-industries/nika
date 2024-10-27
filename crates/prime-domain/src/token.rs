use std::sync::Arc;

use hex::{health, Hexagonal};
use miette::Result;
use models::{EitherSlug, Token, TokenRecordId};
use repos::{
  db::{FetchModelByIndexError, FetchModelError},
  ModelRepositoryFetcher, TokenRepository,
};
use tracing::instrument;

/// A dynamic [`TokenService`] trait object.
pub type DynTokenService = Arc<Box<dyn TokenService>>;

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
#[async_trait::async_trait]
pub trait TokenService:
  ModelRepositoryFetcher<Model = Token> + Hexagonal
{
  /// Verifies that the supplied token ID and secret are valid and exist.
  async fn verify_token_id_and_secret(
    &self,
    id: TokenRecordId,
    secret: models::TokenSecret,
  ) -> Result<Token, TokenVerifyError>;
}

/// Canonical service for the [`Token`] domain model.
pub struct TokenServiceCanonical<R: TokenRepository> {
  token_repo: R,
}

impl<R: TokenRepository + Clone> Clone for TokenServiceCanonical<R> {
  fn clone(&self) -> Self {
    Self {
      token_repo: self.token_repo.clone(),
    }
  }
}

impl<R: TokenRepository> TokenServiceCanonical<R> {
  /// Create a new instance of the canonical [`Token`] service.
  pub fn new(token_repo: R) -> Self {
    tracing::info!("creating new `TokenServiceCanonical` instance");
    Self { token_repo }
  }
}

#[async_trait::async_trait]
impl<R: TokenRepository> health::HealthReporter for TokenServiceCanonical<R> {
  fn name(&self) -> &'static str { stringify!(TokenServiceCanonical<R>) }
  async fn health_check(&self) -> health::ComponentHealth {
    health::AdditiveComponentHealth::from_futures(Some(
      self.token_repo.health_report(),
    ))
    .await
    .into()
  }
}

crate::impl_model_repository_fetcher_for_service!(
  TokenServiceCanonical,
  Token,
  TokenRepository,
  token_repo
);

#[async_trait::async_trait]
impl<R: TokenRepository> TokenService for TokenServiceCanonical<R> {
  #[instrument(skip(self))]
  async fn verify_token_id_and_secret(
    &self,
    id: TokenRecordId,
    secret: models::TokenSecret,
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
