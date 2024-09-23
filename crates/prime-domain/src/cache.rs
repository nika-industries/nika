use std::future::Future;

use miette::Result;
pub use models::Cache;
use models::CacheRecordId;
use repos::{CacheRepository, FetchModelByIndexError, FetchModelError};

/// The definition for the [`Cache`] domain model service.
pub trait CacheService: Clone + Send + Sync + 'static {
  /// Fetch a [`Cache`] by its ID.
  fn fetch(
    &self,
    id: CacheRecordId,
  ) -> impl Future<Output = Result<Option<Cache>, FetchModelError>> + Send;

  /// Find a [`Cache`] by its name.
  fn find_by_name(
    &self,
    name: &str,
  ) -> impl Future<Output = Result<Option<Cache>, FetchModelByIndexError>> + Send;
}

/// Canonical service for the [`Cache`] domain model.
pub struct CacheServiceCanonical<R: CacheRepository> {
  cache_repo: R,
}

impl<R: CacheRepository> Clone for CacheServiceCanonical<R> {
  fn clone(&self) -> Self {
    Self {
      cache_repo: self.cache_repo.clone(),
    }
  }
}

impl<R: CacheRepository> CacheServiceCanonical<R> {
  /// Create a new instance of the canonical [`Cache`] service.
  pub fn new(cache_repo: R) -> Self { Self { cache_repo } }
}

impl<R: CacheRepository> CacheService for CacheServiceCanonical<R> {
  async fn fetch(
    &self,
    id: CacheRecordId,
  ) -> Result<Option<Cache>, FetchModelError> {
    self.cache_repo.fetch_model_by_id(id).await
  }

  async fn find_by_name(
    &self,
    name: &str,
  ) -> Result<Option<Cache>, FetchModelByIndexError> {
    self.cache_repo.find_by_name(name).await
  }
}
