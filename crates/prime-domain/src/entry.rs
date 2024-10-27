use std::sync::Arc;

use hex::{health, Hexagonal};
use miette::Result;
use models::{CacheRecordId, EitherSlug, Entry, LaxSlug};
use repos::{
  db::{FetchModelByIndexError, FetchModelError},
  CreateModelError, EntryCreateRequest, EntryRepository,
  ModelRepositoryCreator, ModelRepositoryFetcher,
};
use tracing::instrument;

/// A dynamic [`EntryService`] trait object.
pub type DynEntryService = Arc<Box<dyn EntryService>>;

/// The definition for the [`Entry`] domain model service.
#[async_trait::async_trait]
pub trait EntryService:
  ModelRepositoryFetcher<Model = Entry>
  + ModelRepositoryCreator<
    Model = Entry,
    ModelCreateRequest = EntryCreateRequest,
    CreateError = CreateModelError,
  > + Hexagonal
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
impl<R: EntryRepository> health::HealthReporter for EntryServiceCanonical<R> {
  fn name(&self) -> &'static str { stringify!(EntryServiceCanonical<R>) }
  async fn health_check(&self) -> health::ComponentHealth {
    health::AdditiveComponentHealth::from_futures(Some(
      self.entry_repo.health_report(),
    ))
    .await
    .into()
  }
}

crate::impl_model_repository_fetcher_for_service!(
  EntryServiceCanonical,
  Entry,
  EntryRepository,
  entry_repo
);

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
