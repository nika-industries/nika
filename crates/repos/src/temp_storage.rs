use std::sync::Arc;

use models::dvf::TempStoragePath;
use storage::temp::TempStorageCreds;
pub use storage::{
  DynAsyncReader, ReadError as StorageReadError, StorageClientGenerator,
  WriteError as StorageWriteError,
};

/// Descriptor trait for repositories that handle temp storage.
#[async_trait::async_trait]
pub trait TempStorageRepository: Send + Sync + 'static {
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

/// The repository for temp storage.
#[derive(Clone)]
pub struct TempStorageRepositoryCanonical {
  client: Arc<storage::DynStorageClient>,
}

impl TempStorageRepositoryCanonical {
  /// Create a new instance of the temp storage repository.
  pub async fn new(creds: TempStorageCreds) -> miette::Result<Self> {
    tracing::info!("creating new `TempStorageRepositoryCanonical` instance");
    Ok(Self {
      client: Arc::new(creds.as_creds().client().await?),
    })
  }
}

#[async_trait::async_trait]
impl TempStorageRepository for TempStorageRepositoryCanonical {
  #[tracing::instrument(skip(self))]
  async fn read(
    &self,
    path: TempStoragePath,
  ) -> Result<DynAsyncReader, StorageReadError> {
    self.client.read(path.as_ref()).await
  }

  #[tracing::instrument(skip(self, data))]
  async fn store(
    &self,
    data: DynAsyncReader,
  ) -> Result<TempStoragePath, StorageWriteError> {
    let path = TempStoragePath::new_random();
    self.client.write(path.as_ref(), data).await?;
    Ok(path)
  }
}
