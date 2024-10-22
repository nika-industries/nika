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
  async fn health_check(&self) -> health::ComponentHealth {
    health::AdditiveComponentHealth::from_futures(Some(
      self.base_repo.health_report(),
    ))
    .await
    .into()
  }
}

crate::impl_model_repository!(
  StoreRepositoryCanonical,
  Store,
  StoreCreateRequest,
  CreateModelError
);
