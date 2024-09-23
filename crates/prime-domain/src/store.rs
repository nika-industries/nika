pub use models::Store;
use repos::StoreRepository;

/// The definition for the [`Store`] domain model service.
pub trait StoreService: Clone + Send + Sync + 'static {}

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

impl<R: StoreRepository> StoreService for StoreServiceCanonical<R> {}
