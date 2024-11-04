use miette::Result;
use models::{
  CacheRecordId, EntryRecordId, StoreRecordId, StrictSlug, TokenRecordId,
};
use repos::{
  db::{FetchModelByIndexError, FetchModelError},
  Cache, DynAsyncReader, Entry, EntryCreateRequest, StorageReadError,
  StorageWriteError, Store, Token,
};

use crate::{
  PrimeDomainService, ReadFromStoreError, TokenVerifyError, WriteToStoreError,
};

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

  async fn write_to_store(
    &self,
    store_id: StoreRecordId,
    path: models::LaxSlug,
    data: DynAsyncReader,
  ) -> Result<models::CompressionStatus, WriteToStoreError> {
    self.deref().write_to_store(store_id, path, data).await
  }
  async fn read_from_store(
    &self,
    store_id: StoreRecordId,
    path: models::LaxSlug,
  ) -> Result<DynAsyncReader, ReadFromStoreError> {
    self.deref().read_from_store(store_id, path).await
  }

  async fn read_from_temp_storage(
    &self,
    path: models::TempStoragePath,
  ) -> Result<DynAsyncReader, StorageReadError> {
    self.deref().read_from_temp_storage(path).await
  }
  async fn write_to_temp_storage(
    &self,
    data: DynAsyncReader,
  ) -> Result<models::TempStoragePath, StorageWriteError> {
    self.deref().write_to_temp_storage(data).await
  }
}
