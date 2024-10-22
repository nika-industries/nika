//! Provides a repository for the [`Entry`] domain model.

use db::{FetchModelByIndexError, FetchModelError};
use hex::health::{self, HealthAware};
use models::{CacheRecordId, LaxSlug};
pub use models::{Entry, EntryCreateRequest};
use tracing::instrument;

use super::*;
pub use crate::base::CreateModelError;
use crate::base::{BaseRepository, DatabaseAdapter};

/// Descriptor trait for repositories that handle [`Entry`] domain model.
#[async_trait::async_trait]
pub trait EntryRepository:
  ModelRepository<
  Model = Entry,
  ModelCreateRequest = EntryCreateRequest,
  CreateError = CreateModelError,
>
{
  /// Find an [`Entry`] by its cache ID and path.
  #[instrument(skip(self))]
  async fn find_by_entry_id_and_path(
    &self,
    cache_id: CacheRecordId,
    path: LaxSlug,
  ) -> Result<Option<Entry>, FetchModelByIndexError> {
    let index_value = LaxSlug::new(format!("{cache_id}-{path}"));
    self
      .fetch_model_by_index("cache-id-path".into(), index_value.into())
      .await
  }
}

impl<T> EntryRepository for T where
  T: ModelRepository<
    Model = Entry,
    ModelCreateRequest = EntryCreateRequest,
    CreateError = CreateModelError,
  >
{
}

/// The repository for the [`Entry`] domain model.
pub struct EntryRepositoryCanonical<DB: DatabaseAdapter> {
  base_repo: BaseRepository<Entry, DB>,
}

impl<DB: DatabaseAdapter + Clone> Clone for EntryRepositoryCanonical<DB> {
  fn clone(&self) -> Self {
    Self {
      base_repo: self.base_repo.clone(),
    }
  }
}

impl<DB: DatabaseAdapter> EntryRepositoryCanonical<DB> {
  /// Create a new instance of the [`Entry`] repository.
  pub fn new(db_adapter: DB) -> Self {
    tracing::info!("creating new `EntryRepositoryCanonical` instance");
    Self {
      base_repo: BaseRepository::new(db_adapter),
    }
  }
}

#[async_trait::async_trait]
impl<DB: DatabaseAdapter> health::HealthReporter
  for EntryRepositoryCanonical<DB>
{
  fn name(&self) -> &'static str { stringify!(EntryRepositoryCanonical<DB>) }
  async fn health_check(&self) -> health::ComponentHealth {
    health::AdditiveComponentHealth::from_futures(Some(
      self.base_repo.health_report(),
    ))
    .await
    .into()
  }
}

crate::impl_model_repository!(
  EntryRepositoryCanonical,
  Entry,
  EntryCreateRequest,
  CreateModelError
);
