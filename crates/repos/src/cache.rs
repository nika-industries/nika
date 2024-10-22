//! Provides a repository for the [`Cache`] domain model.

use db::{FetchModelByIndexError, FetchModelError};
use hex::health::{self, HealthAware};
use models::StrictSlug;
pub use models::{Cache, CacheCreateRequest};
use tracing::instrument;

use super::*;
pub use crate::base::CreateModelError;
use crate::base::{BaseRepository, DatabaseAdapter};

/// Descriptor trait for repositories that handle [`Cache`] domain model.
#[async_trait::async_trait]
pub trait CacheRepository:
  ModelRepository<
  Model = Cache,
  ModelCreateRequest = CacheCreateRequest,
  CreateError = CreateModelError,
>
{
  /// Find a [`Cache`] by its name.
  #[instrument(skip(self))]
  async fn find_by_name(
    &self,
    name: StrictSlug,
  ) -> Result<Option<Cache>, FetchModelByIndexError> {
    self
      .fetch_model_by_index("name".to_string(), EitherSlug::Strict(name))
      .await
  }
}

#[async_trait::async_trait]
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

impl<DB: DatabaseAdapter + Clone> Clone for CacheRepositoryCanonical<DB> {
  fn clone(&self) -> Self {
    Self {
      base_repo: self.base_repo.clone(),
    }
  }
}

impl<DB: DatabaseAdapter> CacheRepositoryCanonical<DB> {
  /// Create a new instance of the [`Cache`] repository.
  pub fn new(db_adapter: DB) -> Self {
    tracing::info!("creating new `CacheRepositoryCanonical` instance");
    Self {
      base_repo: BaseRepository::new(db_adapter),
    }
  }
}

crate::impl_repository_on_base!(
  CacheRepositoryCanonical,
  Cache,
  CacheCreateRequest,
  CreateModelError
);
