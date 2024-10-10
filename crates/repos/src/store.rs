//! Provides a repository for the [`Store`] domain model.

use hex::health::{self, HealthAware};
pub use models::{Store, StoreCreateRequest};
use tracing::instrument;

use super::*;
pub use crate::base::CreateModelError;
use crate::base::{BaseRepository, DatabaseAdapter};

/// Descriptor trait for repositories that handle [`Store`] domain model.
#[async_trait::async_trait]
pub trait StoreRepository:
  ModelRepository<
  Model = Store,
  ModelCreateRequest = StoreCreateRequest,
  CreateError = CreateModelError,
>
{
}

impl<T> StoreRepository for T where
  T: ModelRepository<
    Model = Store,
    ModelCreateRequest = StoreCreateRequest,
    CreateError = CreateModelError,
  >
{
}

/// The repository for the [`Store`] domain model.
pub struct StoreRepositoryCanonical<DB: DatabaseAdapter> {
  base_repo: BaseRepository<Store, DB>,
}

impl<DB: DatabaseAdapter + Clone> Clone for StoreRepositoryCanonical<DB> {
  fn clone(&self) -> Self {
    Self {
      base_repo: self.base_repo.clone(),
    }
  }
}

impl<DB: DatabaseAdapter> StoreRepositoryCanonical<DB> {
  /// Create a new instance of the [`Store`] repository.
  pub fn new(db_adapter: DB) -> Self {
    tracing::info!("creating new `StoreRepositoryCanonical` instance");
    Self {
      base_repo: BaseRepository::new(db_adapter),
    }
  }
}

#[async_trait::async_trait]
impl<DB: DatabaseAdapter> health::HealthReporter
  for StoreRepositoryCanonical<DB>
{
  fn name(&self) -> &'static str { stringify!(StoreRepositoryCanonical<DB>) }
  type HealthReport = health::AdditiveComponentHealth;

  async fn health_check(&self) -> Self::HealthReport {
    health::AdditiveComponentHealth::start(self.base_repo.health_report().await)
  }
}

#[async_trait::async_trait]
impl<DB: DatabaseAdapter> ModelRepository for StoreRepositoryCanonical<DB> {
  type Model = Store;
  type ModelCreateRequest = StoreCreateRequest;
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
