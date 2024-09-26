use std::future::Future;

use models::{CacheRecordId, Entry, EntryRecordId, LaxSlug};
use repos::{
  CreateModelError, EntryCreateRequest, EntryRepository,
  FetchModelByIndexError, FetchModelError, ModelRepositoryCreator,
  ModelRepositoryFetcher,
};

/// The definition for the [`Entry`] domain model service.
pub trait EntryService:
  ModelRepositoryFetcher<Model = Entry>
  + ModelRepositoryCreator<
    Model = Entry,
    ModelCreateRequest = EntryCreateRequest,
    CreateError = CreateModelError,
  > + Clone
  + Send
  + Sync
  + 'static
{
  /// Find an [`Entry`] by its cache ID and path.
  fn find_by_entry_id_and_path(
    &self,
    cache_id: CacheRecordId,
    path: LaxSlug,
  ) -> impl Future<Output = Result<Option<Entry>, FetchModelByIndexError>> + Send;
}

/// Canonical service for the [`Entry`] domain model.
pub struct EntryServiceCanonical<R: EntryRepository> {
  entry_repo: R,
}

impl<R: EntryRepository> Clone for EntryServiceCanonical<R> {
  fn clone(&self) -> Self {
    Self {
      entry_repo: self.entry_repo.clone(),
    }
  }
}

impl<R: EntryRepository> EntryServiceCanonical<R> {
  /// Create a new instance of the canonical [`Entry`] service.
  pub fn new(entry_repo: R) -> Self { Self { entry_repo } }
}

impl<R: EntryRepository> ModelRepositoryFetcher for EntryServiceCanonical<R> {
  type Model = Entry;

  fn fetch(
    &self,
    id: EntryRecordId,
  ) -> impl Future<Output = Result<Option<Entry>, FetchModelError>> + Send {
    self.entry_repo.fetch_model_by_id(id)
  }
}

impl<R: EntryRepository> ModelRepositoryCreator for EntryServiceCanonical<R> {
  type Model = Entry;
  type ModelCreateRequest = EntryCreateRequest;
  type CreateError = CreateModelError;

  async fn create_model(
    &self,
    input: EntryCreateRequest,
  ) -> Result<(), Self::CreateError> {
    self.entry_repo.create_model(input).await
  }
}

impl<R: EntryRepository> EntryService for EntryServiceCanonical<R> {
  /// Find an [`Entry`] by its cache ID and path.
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
