//! Provides a repository for the [`Cache`] domain model.

pub use models::{Store, StoreCreateRequest};

use super::*;
pub use crate::base::CreateModelError;
use crate::base::{BaseRepository, DatabaseAdapter};

/// Descriptor trait for repositories that handle [`Store`] domain model.
pub trait StoreRepository:
  ModelRepository<Model = Store, ModelCreateRequest = StoreCreateRequest>
{
}

impl<T> StoreRepository for T where
  T: ModelRepository<Model = Store, ModelCreateRequest = StoreCreateRequest>
{
}

/// The repository for the [`Store`] domain model.
pub struct StoreRepositoryCanonical<DB: DatabaseAdapter> {
  base_repo: BaseRepository<Store, DB>,
}

impl<DB: DatabaseAdapter> Clone for StoreRepositoryCanonical<DB> {
  fn clone(&self) -> Self {
    Self {
      base_repo: self.base_repo.clone(),
    }
  }
}

impl<DB: DatabaseAdapter> StoreRepositoryCanonical<DB> {
  /// Create a new instance of the [`Store`] repository.
  pub fn new(db_adapter: DB) -> Self {
    Self {
      base_repo: BaseRepository::new(db_adapter),
    }
  }
}

impl<DB: DatabaseAdapter> ModelRepository for StoreRepositoryCanonical<DB> {
  type Model = Store;
  type ModelCreateRequest = StoreCreateRequest;
  type CreateError = CreateModelError;

  fn create_model(
    &self,
    input: Self::ModelCreateRequest,
  ) -> impl Future<Output = Result<(), Self::CreateError>> + Send {
    self.base_repo.create_model(input.into())
  }

  fn fetch_model_by_id(
    &self,
    id: models::RecordId<Self::Model>,
  ) -> impl Future<Output = Result<Option<Self::Model>, FetchModelError>> + Send
  {
    self.base_repo.fetch_model_by_id(id)
  }

  fn fetch_model_by_index(
    &self,
    index_name: String,
    index_value: EitherSlug,
  ) -> impl Future<Output = Result<Option<Self::Model>, FetchModelByIndexError>> + Send
  {
    self.base_repo.fetch_model_by_index(index_name, index_value)
  }
}
