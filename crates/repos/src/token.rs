//! Provides a repository for the [`Token`] domain model.

use hex::health::{self, HealthAware};
pub use models::{Token, TokenCreateRequest};
use tracing::instrument;

use super::*;
pub use crate::base::CreateModelError;
use crate::base::{BaseRepository, DatabaseAdapter};

/// Descriptor trait for repositories that handle [`Token`] domain model.
#[async_trait::async_trait]
pub trait TokenRepository:
  ModelRepository<
  Model = Token,
  ModelCreateRequest = TokenCreateRequest,
  CreateError = CreateModelError,
>
{
}

impl<T> TokenRepository for T where
  T: ModelRepository<
    Model = Token,
    ModelCreateRequest = TokenCreateRequest,
    CreateError = CreateModelError,
  >
{
}

/// The repository for the [`Token`] domain model.
pub struct TokenRepositoryCanonical<DB: DatabaseAdapter> {
  base_repo: BaseRepository<Token, DB>,
}

impl<DB: DatabaseAdapter + Clone> Clone for TokenRepositoryCanonical<DB> {
  fn clone(&self) -> Self {
    Self {
      base_repo: self.base_repo.clone(),
    }
  }
}

impl<DB: DatabaseAdapter> TokenRepositoryCanonical<DB> {
  /// Create a new instance of the [`Token`] repository.
  pub fn new(db_adapter: DB) -> Self {
    tracing::info!("creating new `TokenRepositoryCanonical` instance");
    Self {
      base_repo: BaseRepository::new(db_adapter),
    }
  }
}

#[async_trait::async_trait]
impl<DB: DatabaseAdapter> health::HealthReporter
  for TokenRepositoryCanonical<DB>
{
  fn name(&self) -> &'static str { stringify!(TokenRepositoryCanonical<DB>) }
  async fn health_check(&self) -> health::ComponentHealth {
    health::AdditiveComponentHealth::from_futures(Some(
      self.base_repo.health_report(),
    ))
    .await
    .into()
  }
}

crate::impl_model_repository!(
  TokenRepositoryCanonical,
  Token,
  TokenCreateRequest,
  CreateModelError
);
