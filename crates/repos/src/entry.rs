//! Provides a repository for the [`Entry`] domain model.

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
impl<DB: DatabaseAdapter> ModelRepository for EntryRepositoryCanonical<DB> {
  type Model = Entry;
  type ModelCreateRequest = EntryCreateRequest;
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
