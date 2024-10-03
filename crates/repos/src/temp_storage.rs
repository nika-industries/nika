use std::sync::Arc;

use storage::{
  temp::{TempStorageCreds, TempStoragePath},
  StorageClientGenerator,
};

/// Descriptor trait for repositories that handle temp storage.
#[async_trait::async_trait]
pub trait TempStorageRepository: Send + Sync + 'static {
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
  ) -> Result<storage::DynAsyncReader, storage::ReadError> {
    self.client.read(&path.0).await
  }

  #[tracing::instrument(skip(self, data))]
  async fn store(
    &self,
    data: storage::DynAsyncReader,
  ) -> Result<TempStoragePath, storage::WriteError> {
    let path = TempStoragePath::new_random();
    self.client.write(&path.0, data).await?;
    Ok(path)
  }
}
