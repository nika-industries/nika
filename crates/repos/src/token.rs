//! Provides a repository for the [`Token`] domain model.

pub use models::{Token, TokenCreateRequest};

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

impl<DB: DatabaseAdapter> Clone for TokenRepositoryCanonical<DB> {
  fn clone(&self) -> Self {
    Self {
      base_repo: self.base_repo.clone(),
    }
  }
}

impl<DB: DatabaseAdapter> TokenRepositoryCanonical<DB> {
  /// Create a new instance of the [`Token`] repository.
  pub fn new(db_adapter: DB) -> Self {
    Self {
      base_repo: BaseRepository::new(db_adapter),
    }
  }
}

#[async_trait::async_trait]
impl<DB: DatabaseAdapter> ModelRepository for TokenRepositoryCanonical<DB> {
  type Model = Token;
  type ModelCreateRequest = TokenCreateRequest;
  type CreateError = CreateModelError;

  async fn create_model(
    &self,
    input: Self::ModelCreateRequest,
  ) -> Result<(), Self::CreateError> {
    self.base_repo.create_model(input.into()).await
  }

  async fn fetch_model_by_id(
    &self,
    id: models::RecordId<Self::Model>,
  ) -> Result<Option<Self::Model>, FetchModelError> {
    self.base_repo.fetch_model_by_id(id).await
  }

  async fn fetch_model_by_index(
    &self,
    index_name: String,
    index_value: EitherSlug,
  ) -> Result<Option<Self::Model>, FetchModelByIndexError> {
    self
      .base_repo
      .fetch_model_by_index(index_name, index_value)
      .await
  }
}
