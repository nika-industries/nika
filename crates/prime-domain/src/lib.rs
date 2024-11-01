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

use std::sync::Arc;

pub use hex;
use hex::{health, Hexagonal};
use miette::Result;
pub use models;
use models::{
  Cache, CacheRecordId, Entry, EntryCreateRequest, EntryRecordId, Store,
  StoreRecordId, StrictSlug, Token, TokenRecordId,
};
pub use repos::{
  self, StorageReadError, StorageWriteError, TempStorageCreds,
  TempStorageCredsError,
};
use repos::{
  db::{FetchModelByIndexError, FetchModelError},
  CacheRepository, DynAsyncReader, EntryRepository, StoreRepository,
  TempStorageRepository, TokenRepository, UserStorageClient,
};
use tracing::instrument;

/// The error type for token verification.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum TokenVerifyError {
  /// The token ID was not found.
  #[error("token ID not found")]
  IdNotFound,
  /// The token secret does not match the expected secret.
  #[error("token secret mismatch")]
  SecretMismatch,
  /// An error occurred while fetching the token.
  #[error("error fetching token")]
  #[diagnostic_source]
  FetchError(FetchModelError),
}

/// A dynamic [`PrimeDomainService`] trait object.
pub type DynPrimeDomainService = Arc<Box<dyn PrimeDomainService>>;

/// The prime domain service trait.
#[async_trait::async_trait]
pub trait PrimeDomainService: Hexagonal {
  /// Fetch a [`Cache`] by its ID.
  async fn fetch_cache_by_id(
    &self,
    id: CacheRecordId,
  ) -> Result<Option<Cache>, FetchModelError>;
  /// Fetch an [`Entry`] by its ID.
  async fn fetch_entry_by_id(
    &self,
    id: EntryRecordId,
  ) -> Result<Option<Entry>, FetchModelError>;
  /// Fetch a [`Store`] by its ID.
  async fn fetch_store_by_id(
    &self,
    id: StoreRecordId,
  ) -> Result<Option<Store>, FetchModelError>;
  /// Fetch a [`Token`] by its ID.
  async fn fetch_token_by_id(
    &self,
    id: TokenRecordId,
  ) -> Result<Option<Token>, FetchModelError>;
  /// Produce a list of all [`Cache`]s.
  async fn enumerate_caches(&self) -> Result<Vec<Cache>>;
  /// Produce a list of all [`Entry`]s.
  async fn enumerate_entries(&self) -> Result<Vec<Entry>>;
  /// Produce a list of all [`Store`]s.
  async fn enumerate_stores(&self) -> Result<Vec<Store>>;
  /// Produce a list of all [`Token`]s.
  async fn enumerate_tokens(&self) -> Result<Vec<Token>>;

  /// Find a [`Cache`] by its name.
  async fn find_cache_by_name(
    &self,
    name: StrictSlug,
  ) -> Result<Option<Cache>, FetchModelByIndexError>;
  /// Find an [`Entry`] by its [`Cache`] ID and path.
  async fn find_entry_by_id_and_path(
    &self,
    cache_id: CacheRecordId,
    path: models::LaxSlug,
  ) -> Result<Option<Entry>, FetchModelByIndexError>;
  /// Creates an [`Entry`] from an [`EntryCreateRequest`].
  async fn create_entry(
    &self,
    entry_cr: EntryCreateRequest,
  ) -> Result<Entry, repos::CreateModelError>;
  /// Verify a [`Token`] by its ID and secret.
  async fn verify_token_id_and_secret(
    &self,
    id: TokenRecordId,
    secret: models::TokenSecret,
  ) -> Result<Token, TokenVerifyError>;

  /// Connect to a user storage.
  async fn connect_to_user_storage(
    &self,
    creds: models::StorageCredentials,
  ) -> Result<Box<dyn UserStorageClient>>;

  /// Read data from the temp storage.
  async fn read_from_temp_storage(
    &self,
    path: models::TempStoragePath,
  ) -> Result<DynAsyncReader, StorageReadError>;
  /// Store data in the temp storage.
  async fn store_in_temp_storage(
    &self,
    data: DynAsyncReader,
  ) -> Result<models::TempStoragePath, StorageWriteError>;
}

// impl for smart pointers
#[async_trait::async_trait]
impl<T, I> PrimeDomainService for T
where
  T: std::ops::Deref<Target = I> + Send + Sync + 'static,
  I: PrimeDomainService + ?Sized,
{
  async fn fetch_cache_by_id(
    &self,
    id: CacheRecordId,
  ) -> Result<Option<Cache>, FetchModelError> {
    self.deref().fetch_cache_by_id(id).await
  }
  async fn fetch_entry_by_id(
    &self,
    id: EntryRecordId,
  ) -> Result<Option<Entry>, FetchModelError> {
    self.deref().fetch_entry_by_id(id).await
  }
  async fn fetch_store_by_id(
    &self,
    id: StoreRecordId,
  ) -> Result<Option<Store>, FetchModelError> {
    self.deref().fetch_store_by_id(id).await
  }
  async fn fetch_token_by_id(
    &self,
    id: TokenRecordId,
  ) -> Result<Option<Token>, FetchModelError> {
    self.deref().fetch_token_by_id(id).await
  }
  async fn enumerate_caches(&self) -> Result<Vec<Cache>> {
    self.deref().enumerate_caches().await
  }
  async fn enumerate_entries(&self) -> Result<Vec<Entry>> {
    self.deref().enumerate_entries().await
  }
  async fn enumerate_stores(&self) -> Result<Vec<Store>> {
    self.deref().enumerate_stores().await
  }
  async fn enumerate_tokens(&self) -> Result<Vec<Token>> {
    self.deref().enumerate_tokens().await
  }

  async fn find_cache_by_name(
    &self,
    name: StrictSlug,
  ) -> Result<Option<Cache>, FetchModelByIndexError> {
    self.deref().find_cache_by_name(name).await
  }
  async fn find_entry_by_id_and_path(
    &self,
    cache_id: CacheRecordId,
    path: models::LaxSlug,
  ) -> Result<Option<Entry>, FetchModelByIndexError> {
    self.deref().find_entry_by_id_and_path(cache_id, path).await
  }
  async fn create_entry(
    &self,
    entry_cr: EntryCreateRequest,
  ) -> Result<Entry, repos::CreateModelError> {
    self.deref().create_entry(entry_cr).await
  }
  async fn verify_token_id_and_secret(
    &self,
    id: TokenRecordId,
    secret: models::TokenSecret,
  ) -> Result<Token, TokenVerifyError> {
    self.deref().verify_token_id_and_secret(id, secret).await
  }

  async fn connect_to_user_storage(
    &self,
    creds: models::StorageCredentials,
  ) -> Result<Box<dyn UserStorageClient>> {
    Ok(Box::new(self.deref().connect_to_user_storage(creds).await?))
  }

  async fn read_from_temp_storage(
    &self,
    path: models::TempStoragePath,
  ) -> Result<DynAsyncReader, StorageReadError> {
    self.deref().read_from_temp_storage(path).await
  }
  async fn store_in_temp_storage(
    &self,
    data: DynAsyncReader,
  ) -> Result<models::TempStoragePath, StorageWriteError> {
    self.deref().store_in_temp_storage(data).await
  }
}
/// The canonical implementation of [`PrimeDomainService`].
pub struct PrimeDomainServiceCanonical<
  CR: CacheRepository,
  ER: EntryRepository,
  SR: StoreRepository,
  TR: TokenRepository,
  TSR: TempStorageRepository,
  USR: repos::UserStorageRepository,
> {
  cache_repo:        CR,
  entry_repo:        ER,
  store_repo:        SR,
  token_repo:        TR,
  temp_storage_repo: TSR,
  user_storage_repo: USR,
}

impl<CR, ER, SR, TR, TSR, USR>
  PrimeDomainServiceCanonical<CR, ER, SR, TR, TSR, USR>
where
  CR: CacheRepository,
  ER: EntryRepository,
  SR: StoreRepository,
  TR: TokenRepository,
  TSR: TempStorageRepository,
  USR: repos::UserStorageRepository,
{
  /// Create a new instance of the canonical prime domain service.
  pub fn new(
    cache_repo: CR,
    entry_repo: ER,
    store_repo: SR,
    token_repo: TR,
    temp_storage_repo: TSR,
    user_storage_repo: USR,
  ) -> Self {
    tracing::info!("creating new `PrimeDomainServiceCanonical` instance");
    Self {
      cache_repo,
      entry_repo,
      store_repo,
      token_repo,
      temp_storage_repo,
      user_storage_repo,
    }
  }
}

#[async_trait::async_trait]
impl<CR, ER, SR, TR, TSR, USR> PrimeDomainService
  for PrimeDomainServiceCanonical<CR, ER, SR, TR, TSR, USR>
where
  CR: CacheRepository,
  ER: EntryRepository,
  SR: StoreRepository,
  TR: TokenRepository,
  TSR: TempStorageRepository,
  USR: repos::UserStorageRepository,
{
  async fn fetch_cache_by_id(
    &self,
    id: CacheRecordId,
  ) -> Result<Option<Cache>, FetchModelError> {
    self.cache_repo.fetch_model_by_id(id).await
  }
  async fn fetch_entry_by_id(
    &self,
    id: EntryRecordId,
  ) -> Result<Option<Entry>, FetchModelError> {
    self.entry_repo.fetch_model_by_id(id).await
  }
  async fn fetch_store_by_id(
    &self,
    id: StoreRecordId,
  ) -> Result<Option<Store>, FetchModelError> {
    self.store_repo.fetch_model_by_id(id).await
  }
  async fn fetch_token_by_id(
    &self,
    id: TokenRecordId,
  ) -> Result<Option<Token>, FetchModelError> {
    self.token_repo.fetch_model_by_id(id).await
  }
  async fn enumerate_caches(&self) -> Result<Vec<Cache>> {
    self.cache_repo.enumerate_models().await
  }
  async fn enumerate_entries(&self) -> Result<Vec<Entry>> {
    self.entry_repo.enumerate_models().await
  }
  async fn enumerate_stores(&self) -> Result<Vec<Store>> {
    self.store_repo.enumerate_models().await
  }
  async fn enumerate_tokens(&self) -> Result<Vec<Token>> {
    self.token_repo.enumerate_models().await
  }

  async fn find_cache_by_name(
    &self,
    name: StrictSlug,
  ) -> Result<Option<Cache>, FetchModelByIndexError> {
    self.cache_repo.find_by_name(name).await
  }
  async fn find_entry_by_id_and_path(
    &self,
    cache_id: CacheRecordId,
    path: models::LaxSlug,
  ) -> Result<Option<Entry>, FetchModelByIndexError> {
    self
      .entry_repo
      .find_entry_by_id_and_path(cache_id, path)
      .await
  }
  async fn create_entry(
    &self,
    entry_cr: EntryCreateRequest,
  ) -> Result<Entry, repos::CreateModelError> {
    self.entry_repo.create_model(entry_cr).await
  }
  async fn verify_token_id_and_secret(
    &self,
    id: TokenRecordId,
    secret: models::TokenSecret,
  ) -> Result<Token, TokenVerifyError> {
    let token = self
      .token_repo
      .fetch_model_by_id(id)
      .await
      .map_err(TokenVerifyError::FetchError)?
      .ok_or(TokenVerifyError::IdNotFound)?;

    if token.secret != secret {
      return Err(TokenVerifyError::SecretMismatch);
    }
    Ok(token)
  }

  async fn connect_to_user_storage(
    &self,
    creds: models::StorageCredentials,
  ) -> Result<Box<dyn UserStorageClient>> {
    Ok(Box::new(
      self
        .user_storage_repo
        .connect_to_user_storage(creds)
        .await?,
    ))
  }

  async fn read_from_temp_storage(
    &self,
    path: models::TempStoragePath,
  ) -> Result<DynAsyncReader, StorageReadError> {
    self.temp_storage_repo.read(path).await
  }
  async fn store_in_temp_storage(
    &self,
    data: DynAsyncReader,
  ) -> Result<models::TempStoragePath, StorageWriteError> {
    self.temp_storage_repo.store(data).await
  }
}

#[async_trait::async_trait]
impl<CR, ER, SR, TR, TSR, USR> health::HealthReporter
  for PrimeDomainServiceCanonical<CR, ER, SR, TR, TSR, USR>
where
  CR: CacheRepository,
  ER: EntryRepository,
  SR: StoreRepository,
  TR: TokenRepository,
  TSR: TempStorageRepository,
  USR: repos::UserStorageRepository,
{
  fn name(&self) -> &'static str { stringify!(PrimeDomainServiceCanonical) }
  #[instrument(skip(self))]
  async fn health_check(&self) -> health::ComponentHealth {
    health::AdditiveComponentHealth::from_futures(vec![
      self.cache_repo.health_report(),
      self.entry_repo.health_report(),
      self.store_repo.health_report(),
      self.token_repo.health_report(),
      self.temp_storage_repo.health_report(),
      self.user_storage_repo.health_report(),
    ])
    .await
    .into()
  }
}

/// Implement the [`ModelRepositoryFetcher`](repos::ModelRepositoryFetcher)
/// trait for a service which is generic on its repo trait.
#[macro_export]
macro_rules! impl_model_repository_fetcher_for_service {
  ($service:ident, $model:ty, $repo_trait:ident, $repo_field:ident) => {
    #[async_trait::async_trait]
    impl<R: $repo_trait> ModelRepositoryFetcher for $service<R> {
      type Model = $model;

      #[instrument(skip(self))]
      async fn fetch(
        &self,
        id: models::RecordId<Self::Model>,
      ) -> Result<Option<Self::Model>, FetchModelError> {
        self.$repo_field.fetch_model_by_id(id).await
      }
      #[instrument(skip(self))]
      async fn fetch_model_by_index(
        &self,
        index_name: String,
        index_value: EitherSlug,
      ) -> Result<Option<Self::Model>, FetchModelByIndexError> {
        self
          .$repo_field
          .fetch_model_by_index(index_name, index_value)
          .await
      }
      #[instrument(skip(self))]
      async fn enumerate_models(&self) -> Result<Vec<Self::Model>> {
        self.$repo_field.enumerate_models().await
      }
    }
  };
}
