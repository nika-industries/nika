use std::sync::Arc;

use hex::{health, Hexagonal};
use miette::Result;
use models::{Cache, EitherSlug, StrictSlug};
use repos::{
  db::{FetchModelByIndexError, FetchModelError},
  CacheRepository, ModelRepositoryFetcher,
};
use tracing::instrument;

/// A dynamic [`CacheService`] trait object.
pub type DynCacheService = Arc<Box<dyn CacheService>>;

/// The definition for the [`Cache`] domain model service.
#[async_trait::async_trait]
pub trait CacheService:
  ModelRepositoryFetcher<Model = Cache> + Hexagonal
{
  /// Find a [`Cache`] by its name.
  async fn find_by_name(
    &self,
    name: StrictSlug,
  ) -> Result<Option<Cache>, FetchModelByIndexError>;
}

/// Canonical service for the [`Cache`] domain model.
pub struct CacheServiceCanonical<R: CacheRepository> {
  cache_repo: R,
}

impl<R: CacheRepository + Clone> Clone for CacheServiceCanonical<R> {
  fn clone(&self) -> Self {
    Self {
      cache_repo: self.cache_repo.clone(),
    }
  }
}

impl<R: CacheRepository> CacheServiceCanonical<R> {
  /// Create a new instance of the canonical [`Cache`] service.
  pub fn new(cache_repo: R) -> Self {
    tracing::info!("creating new `CacheServiceCanonical` instance");
    Self { cache_repo }
  }
}

#[async_trait::async_trait]
impl<R: CacheRepository> health::HealthReporter for CacheServiceCanonical<R> {
  fn name(&self) -> &'static str { stringify!(CacheServiceCanonical<R>) }
  async fn health_check(&self) -> health::ComponentHealth {
    health::AdditiveComponentHealth::from_futures(Some(
      self.cache_repo.health_report(),
    ))
    .await
    .into()
  }
}

crate::impl_model_repository_fetcher_for_service!(
  CacheServiceCanonical,
  Cache,
  CacheRepository,
  cache_repo
);

#[async_trait::async_trait]
impl<R: CacheRepository> CacheService for CacheServiceCanonical<R> {
  #[instrument(skip(self))]
  async fn find_by_name(
    &self,
    name: StrictSlug,
  ) -> Result<Option<Cache>, FetchModelByIndexError> {
    self.cache_repo.find_by_name(name).await
  }
}
