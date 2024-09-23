use std::future::Future;

pub use models::Entry;
use models::EntryRecordId;
use repos::{EntryRepository, FetchModelError, ModelRepositoryFetcher};

/// The definition for the [`Entry`] domain model service.
pub trait EntryService:
  ModelRepositoryFetcher<Model = Entry> + Clone + Send + Sync + 'static
{
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

impl<R: EntryRepository> EntryService for EntryServiceCanonical<R> {}
