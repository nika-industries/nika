//! Provides prime-domain Services, the entry points for domain-specific
//! business logic.

mod cache {
  use repos::CacheRepository;

  /// The definition for the [`Cache`](models::Cache) domain model service.
  pub trait CacheService: Clone + Send + Sync + 'static {}

  /// Canonical service for the [`Cache`](models::Cache) domain model.
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
    /// Create a new instance of the canonical [`Cache`](models::Cache) service.
    pub fn new(cache_repo: R) -> Self { Self { cache_repo } }
  }

  impl<R: CacheRepository> CacheService for CacheServiceCanonical<R> {}
}

pub use models::{Cache, CacheCreateRequest};
pub use repos::ModelRepository;

pub use self::cache::*;
