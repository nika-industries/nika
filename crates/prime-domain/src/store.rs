use std::future::Future;

pub use models::Store;
use models::StoreRecordId;
use repos::{FetchModelError, StoreRepository};

/// The definition for the [`Store`] domain model service.
pub trait StoreService: Clone + Send + Sync + 'static {
  /// Fetch a [`Store`] by its ID.
  fn fetch(
    &self,
    id: StoreRecordId,
  ) -> impl Future<Output = Result<Option<Store>, FetchModelError>> + Send;
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

impl<R: StoreRepository> StoreService for StoreServiceCanonical<R> {
  async fn fetch(
    &self,
    id: StoreRecordId,
  ) -> Result<Option<Store>, FetchModelError> {
    self.store_repo.fetch_model_by_id(id).await
  }
}
