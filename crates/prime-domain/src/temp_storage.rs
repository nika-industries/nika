use repos::TempStorageRepository;
use storage::temp::TempStoragePath;

/// The definition for the temp storage service.
#[async_trait::async_trait]
pub trait TempStorageService: Send + Sync + 'static {
  /// Read data from the storage.
  async fn read(
    &self,
    path: TempStoragePath,
  ) -> Result<storage::DynAsyncReader, storage::ReadError>;
  /// Store data in the storage.
  async fn store(
    &self,
    data: storage::DynAsyncReader,
  ) -> Result<TempStoragePath, storage::WriteError>;
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
  ) -> Result<storage::DynAsyncReader, storage::ReadError> {
    self.storage_repo.read(path).await
  }

  #[tracing::instrument(skip(self, data))]
  async fn store(
    &self,
    data: storage::DynAsyncReader,
  ) -> Result<TempStoragePath, storage::WriteError> {
    self.storage_repo.store(data).await
  }
}
