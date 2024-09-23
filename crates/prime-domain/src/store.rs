use std::future::Future;

pub use models::Store;
use models::StoreRecordId;
use repos::{FetchModelError, ModelRepositoryFetcher, StoreRepository};

/// The definition for the [`Store`] domain model service.
pub trait StoreService:
  ModelRepositoryFetcher<Model = Store> + Clone + Send + Sync + 'static
{
}

/// Canonical service for the [`Store`] domain model.
pub struct StoreServiceCanonical<R: StoreRepository> {
  store_repo: R,
}

impl<R: StoreRepository> Clone for StoreServiceCanonical<R> {
  fn clone(&self) -> Self {
    Self {
      store_repo: self.store_repo.clone(),
    }
  }
}

impl<R: StoreRepository> StoreServiceCanonical<R> {
  /// Create a new instance of the canonical [`Store`] service.
  pub fn new(store_repo: R) -> Self { Self { store_repo } }
}

impl<R: StoreRepository> ModelRepositoryFetcher for StoreServiceCanonical<R> {
  type Model = Store;

  fn fetch(
    &self,
    id: StoreRecordId,
  ) -> impl Future<Output = Result<Option<Store>, FetchModelError>> + Send {
    self.store_repo.fetch_model_by_id(id)
  }
}

impl<R: StoreRepository> StoreService for StoreServiceCanonical<R> {}
