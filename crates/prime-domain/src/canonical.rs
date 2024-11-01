pub use hex;
use hex::health;
use miette::Result;
pub use models;
use models::{
  Cache, CacheRecordId, Entry, EntryCreateRequest, EntryRecordId, Store,
  StoreRecordId, StrictSlug, Token, TokenRecordId,
};
pub use repos::{self, StorageReadError, StorageWriteError};
use repos::{
  db::{FetchModelByIndexError, FetchModelError},
  CacheRepository, DynAsyncReader, EntryRepository, StoreRepository,
  TempStorageRepository, TokenRepository, UserStorageClient,
};
use tracing::instrument;

use crate::{PrimeDomainService, TokenVerifyError};

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
