use models::{CacheRecordId, Entry, EntryRecordId, LaxSlug};
use repos::{
  CreateModelError, EntryCreateRequest, EntryRepository,
  FetchModelByIndexError, FetchModelError, ModelRepositoryCreator,
  ModelRepositoryFetcher,
};
use tracing::instrument;

/// The definition for the [`Entry`] domain model service.
#[async_trait::async_trait]
pub trait EntryService:
  ModelRepositoryFetcher<Model = Entry>
  + ModelRepositoryCreator<
    Model = Entry,
    ModelCreateRequest = EntryCreateRequest,
    CreateError = CreateModelError,
  > + Send
  + Sync
  + 'static
{
  /// Find an [`Entry`] by its cache ID and path.
  async fn find_by_entry_id_and_path(
    &self,
    cache_id: CacheRecordId,
    path: LaxSlug,
  ) -> Result<Option<Entry>, FetchModelByIndexError>;
}

/// Canonical service for the [`Entry`] domain model.
pub struct EntryServiceCanonical<R: EntryRepository> {
  entry_repo: R,
}

impl<R: EntryRepository + Clone> Clone for EntryServiceCanonical<R> {
  fn clone(&self) -> Self {
    Self {
      entry_repo: self.entry_repo.clone(),
    }
  }
}

impl<R: EntryRepository> EntryServiceCanonical<R> {
  /// Create a new instance of the canonical [`Entry`] service.
  pub fn new(entry_repo: R) -> Self {
    tracing::info!("creating new `EntryServiceCanonical` instance");
    Self { entry_repo }
  }
}

#[async_trait::async_trait]
impl<R: EntryRepository> ModelRepositoryFetcher for EntryServiceCanonical<R> {
  type Model = Entry;

  #[instrument(skip(self))]
  async fn fetch(
    &self,
    id: EntryRecordId,
  ) -> Result<Option<Entry>, FetchModelError> {
    self.entry_repo.fetch_model_by_id(id).await
  }
}

#[async_trait::async_trait]
impl<R: EntryRepository> ModelRepositoryCreator for EntryServiceCanonical<R> {
  type Model = Entry;
  type ModelCreateRequest = EntryCreateRequest;
  type CreateError = CreateModelError;

  #[instrument(skip(self))]
  async fn create_model(
    &self,
    input: EntryCreateRequest,
  ) -> Result<(), Self::CreateError> {
    self.entry_repo.create_model(input).await
  }
}

#[async_trait::async_trait]
impl<R: EntryRepository> EntryService for EntryServiceCanonical<R> {
  /// Find an [`Entry`] by its cache ID and path.
  #[instrument(skip(self))]
  async fn find_by_entry_id_and_path(
    &self,
    cache_id: CacheRecordId,
    path: LaxSlug,
  ) -> Result<Option<Entry>, FetchModelByIndexError> {
    self
      .entry_repo
      .find_by_entry_id_and_path(cache_id, path)
      .await
  }
}
