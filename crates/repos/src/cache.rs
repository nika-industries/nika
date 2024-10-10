//! Provides a repository for the [`Cache`] domain model.

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

#[async_trait::async_trait]
impl<DB: DatabaseAdapter> health::HealthReporter
  for CacheRepositoryCanonical<DB>
{
  fn name(&self) -> &'static str { stringify!(CacheRepositoryCanonical<DB>) }
  type HealthReport = health::AdditiveComponentHealth;

  async fn health_check(&self) -> Self::HealthReport {
    health::AdditiveComponentHealth::start(self.base_repo.health_report().await)
  }
}

#[async_trait::async_trait]
impl<DB: DatabaseAdapter> ModelRepository for CacheRepositoryCanonical<DB> {
  type Model = Cache;
  type ModelCreateRequest = CacheCreateRequest;
  type CreateError = CreateModelError;

  #[instrument(skip(self))]
  async fn create_model(
    &self,
    input: Self::ModelCreateRequest,
  ) -> Result<(), Self::CreateError> {
    self.base_repo.create_model(input.into()).await
  }

  #[instrument(skip(self))]
  async fn fetch_model_by_id(
    &self,
    id: models::RecordId<Self::Model>,
  ) -> Result<Option<Self::Model>, FetchModelError> {
    self.base_repo.fetch_model_by_id(id).await
  }

  #[instrument(skip(self))]
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
