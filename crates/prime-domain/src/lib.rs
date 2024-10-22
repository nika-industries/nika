//! Provides prime-domain Services, the entry points for domain-specific
//! business logic.
//!
//! This is where our services are defined. A service is a domain-specific
//! business logic entrypoint. It has a single responsibility and has everything
//! necessary to manipulate a sub-category of the domain.
//!
//! All of the business logic for a given domain operation should be inside a
//! service method. Data should be validated and encapsulated before it gets to
//! the service.

mod cache;
mod entry;
mod store;
mod temp_storage;
mod token;

pub use hex;
pub use models;
pub use repos;

pub use self::{
  cache::{CacheService, CacheServiceCanonical, DynCacheService},
  entry::{DynEntryService, EntryService, EntryServiceCanonical},
  store::{DynStoreService, StoreService, StoreServiceCanonical},
  temp_storage::{
    DynTempStorageService, TempStorageService, TempStorageServiceCanonical,
  },
  token::{DynTokenService, TokenService, TokenServiceCanonical},
};

/// Implement the [`ModelRepositoryFetcher`](repos::ModelRepositoryFetcher)
/// trait for a service which is generic on its repo trait.
#[macro_export]
macro_rules! impl_model_repository_fetcher_for_service {
  ($service:ident, $model:ty, $repo_trait:ident) => {
    #[async_trait::async_trait]
    impl<R: $repo_trait> ModelRepositoryFetcher for $service<R> {
      type Model = $model;

      #[instrument(skip(self))]
      async fn fetch(
        &self,
        id: models::RecordId<Self::Model>,
      ) -> Result<Option<Self::Model>, FetchModelError> {
        self.fetch(id).await
      }
      #[instrument(skip(self))]
      async fn fetch_model_by_index(
        &self,
        index_name: String,
        index_value: EitherSlug,
      ) -> Result<Option<Self::Model>, FetchModelByIndexError> {
        self.fetch_model_by_index(index_name, index_value).await
      }
      #[instrument(skip(self))]
      async fn enumerate_models(&self) -> Result<Vec<Self::Model>> {
        self.enumerate_models().await
      }
    }
  };
}
