use models::{Store, StoreRecordId};
use repos::{FetchModelError, ModelRepositoryFetcher, StoreRepository};

/// The definition for the [`Store`] domain model service.
#[async_trait::async_trait]
pub trait StoreService:
  ModelRepositoryFetcher<Model = Store> + Send + Sync + 'static
{
}

/// Canonical service for the [`Store`] domain model.
pub struct StoreServiceCanonical<R: StoreRepository> {
  store_repo: R,
}

impl<R: StoreRepository + Clone> Clone for StoreServiceCanonical<R> {
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

#[async_trait::async_trait]
impl<R: StoreRepository> ModelRepositoryFetcher for StoreServiceCanonical<R> {
  type Model = Store;

  async fn fetch(
    &self,
    id: StoreRecordId,
  ) -> Result<Option<Store>, FetchModelError> {
    self.store_repo.fetch_model_by_id(id).await
  }
}

#[async_trait::async_trait]
impl<R: StoreRepository> StoreService for StoreServiceCanonical<R> {}
