use hex::Hexagonal;
use models::dvf::TempStoragePath;
use repos::{
  DynAsyncReader, StorageReadError, StorageWriteError, TempStorageRepository,
};

/// The definition for the temp storage service.
#[async_trait::async_trait]
pub trait TempStorageService: Hexagonal {
  /// Read data from the storage.
  async fn read(
    &self,
    path: TempStoragePath,
  ) -> Result<DynAsyncReader, StorageReadError>;
  /// Store data in the storage.
  async fn store(
    &self,
    data: DynAsyncReader,
  ) -> Result<TempStoragePath, StorageWriteError>;
}

/// Canonical service for the temp storage service.
pub struct TempStorageServiceCanonical<S: TempStorageRepository> {
  storage_repo: S,
}

impl<S: TempStorageRepository + Clone> Clone
  for TempStorageServiceCanonical<S>
{
  fn clone(&self) -> Self {
    Self {
      storage_repo: self.storage_repo.clone(),
    }
  }
}

impl<S: TempStorageRepository> TempStorageServiceCanonical<S> {
  /// Create a new instance of the canonical temp storage service.
  pub fn new(storage_repo: S) -> Self {
    tracing::info!("creating new `TempStorageServiceCanonical` instance");
    Self { storage_repo }
  }
}

#[async_trait::async_trait]
impl<S: TempStorageRepository> TempStorageService
  for TempStorageServiceCanonical<S>
{
  #[tracing::instrument(skip(self))]
  async fn read(
    &self,
    path: TempStoragePath,
  ) -> Result<DynAsyncReader, StorageReadError> {
    self.storage_repo.read(path).await
  }

  #[tracing::instrument(skip(self, data))]
  async fn store(
    &self,
    data: DynAsyncReader,
  ) -> Result<TempStoragePath, StorageWriteError> {
    self.storage_repo.store(data).await
  }
}
