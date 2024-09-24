//! Provides a repository for the [`Cache`] domain model.

use models::StrictSlug;
pub use models::{Cache, CacheCreateRequest};

use super::*;
pub use crate::base::CreateModelError;
use crate::base::{BaseRepository, DatabaseAdapter};

/// Descriptor trait for repositories that handle [`Cache`] domain model.
pub trait CacheRepository:
  ModelRepository<
  Model = Cache,
  ModelCreateRequest = CacheCreateRequest,
  CreateError = CreateModelError,
>
{
  /// Find a [`Cache`] by its name.
  fn find_by_name(
    &self,
    name: StrictSlug,
  ) -> impl Future<Output = Result<Option<Cache>, FetchModelByIndexError>> + Send
  {
    self.fetch_model_by_index("name".to_string(), EitherSlug::Strict(name))
  }
}

impl<T> CacheRepository for T where
  T: ModelRepository<
    Model = Cache,
    ModelCreateRequest = CacheCreateRequest,
    CreateError = CreateModelError,
  >
{
}

/// The repository for the [`Cache`] domain model.
pub struct CacheRepositoryCanonical<DB: DatabaseAdapter> {
  base_repo: BaseRepository<Cache, DB>,
}

impl<DB: DatabaseAdapter> Clone for CacheRepositoryCanonical<DB> {
  fn clone(&self) -> Self {
    Self {
      base_repo: self.base_repo.clone(),
    }
  }
}

impl<DB: DatabaseAdapter> CacheRepositoryCanonical<DB> {
  /// Create a new instance of the [`Cache`] repository.
  pub fn new(db_adapter: DB) -> Self {
    Self {
      base_repo: BaseRepository::new(db_adapter),
    }
  }
}

impl<DB: DatabaseAdapter> ModelRepository for CacheRepositoryCanonical<DB> {
  type Model = Cache;
  type ModelCreateRequest = CacheCreateRequest;
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
