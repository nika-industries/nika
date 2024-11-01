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

mod canonical;
mod pointer;

use std::sync::Arc;

pub use hex;
use hex::Hexagonal;
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
  DynAsyncReader,
};

pub use self::canonical::*;

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

  /// Write to store contents.
  async fn write_to_store(
    &self,
    store_id: StoreRecordId,
    path: models::LaxSlug,
    data: DynAsyncReader,
  ) -> Result<models::FileSize, WriteToStoreError>;
  /// Read from store contents.
  async fn read_from_store(
    &self,
    store_id: StoreRecordId,
    path: models::LaxSlug,
  ) -> Result<DynAsyncReader, ReadFromStoreError>;

  /// Read data from the temp storage.
  async fn read_from_temp_storage(
    &self,
    path: models::TempStoragePath,
  ) -> Result<DynAsyncReader, StorageReadError>;
  /// Store data in the temp storage.
  async fn write_to_temp_storage(
    &self,
    data: DynAsyncReader,
  ) -> Result<models::TempStoragePath, StorageWriteError>;
}

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

/// The error type for writing to a store.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum WriteToStoreError {
  /// The store was not found.
  #[error("store not found")]
  StoreNotFound(StoreRecordId),
  /// An error occurred while fetching the store.
  #[error("failed to fetch store")]
  FetchError(FetchModelError),
  /// An error occurred while connecting to user storage.
  #[error("failed to connect to user storage")]
  StorageConnectionError(miette::Report),
  /// An error occurred while writing to the store.
  #[error("failed to write to store")]
  StorageWriteError(StorageWriteError),
}

/// The error type for reading from a store.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum ReadFromStoreError {
  /// The store was not found.
  #[error("store not found")]
  StoreNotFound(StoreRecordId),
  /// An error occurred while fetching the store.
  #[error("failed to fetch store")]
  FetchError(FetchModelError),
  /// An error occurred while connecting to user storage.
  #[error("failed to connect to user storage")]
  StorageConnectionError(miette::Report),
  /// An error occurred while reading from the store.
  #[error("failed to read from store")]
  StorageReadError(StorageReadError),
}
