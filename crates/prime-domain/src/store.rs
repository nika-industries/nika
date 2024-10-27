use std::sync::Arc;

use hex::{health, Hexagonal};
use miette::Result;
use models::{EitherSlug, Store};
use repos::{
  db::{FetchModelByIndexError, FetchModelError},
  ModelRepositoryFetcher, StoreRepository,
};
use tracing::instrument;

/// A dynamic [`StoreService`] trait object.
pub type DynStoreService = Arc<Box<dyn StoreService>>;

/// The definition for the [`Store`] domain model service.
#[async_trait::async_trait]
pub trait StoreService:
  ModelRepositoryFetcher<Model = Store> + Hexagonal
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
  pub fn new(store_repo: R) -> Self {
    tracing::info!("creating new `StoreServiceCanonical` instance");
    Self { store_repo }
  }
}

#[async_trait::async_trait]
impl<R: StoreRepository> health::HealthReporter for StoreServiceCanonical<R> {
  fn name(&self) -> &'static str { stringify!(StoreServiceCanonical<R>) }
  async fn health_check(&self) -> health::ComponentHealth {
    health::AdditiveComponentHealth::from_futures(Some(
      self.store_repo.health_report(),
    ))
    .await
    .into()
  }
}

crate::impl_model_repository_fetcher_for_service!(
  StoreServiceCanonical,
  Store,
  StoreRepository,
  store_repo
);

#[async_trait::async_trait]
impl<R: StoreRepository> StoreService for StoreServiceCanonical<R> {}
