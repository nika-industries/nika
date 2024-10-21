use std::sync::Arc;

use hex::{health, Hexagonal};
use miette::Result;
use models::{EitherSlug, Store, StoreRecordId};
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

#[async_trait::async_trait]
impl<R: StoreRepository> ModelRepositoryFetcher for StoreServiceCanonical<R> {
  type Model = Store;

  #[instrument(skip(self))]
  async fn fetch(
    &self,
    id: StoreRecordId,
  ) -> Result<Option<Store>, FetchModelError> {
    self.store_repo.fetch_model_by_id(id).await
  }
  #[instrument(skip(self))]
  async fn fetch_model_by_index(
    &self,
    index_name: String,
    index_value: EitherSlug,
  ) -> Result<Option<Store>, FetchModelByIndexError> {
    self
      .store_repo
      .fetch_model_by_index(index_name, index_value)
      .await
  }
  #[instrument(skip(self))]
  async fn enumerate_models(&self) -> Result<Vec<Store>> {
    self.store_repo.enumerate_models().await
  }
}

#[async_trait::async_trait]
impl<R: StoreRepository> StoreService for StoreServiceCanonical<R> {}
