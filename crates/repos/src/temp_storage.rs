use std::sync::Arc;

use hex::{
  health::{self},
  Hexagonal,
};
use models::TempStoragePath;
use storage::temp::TempStorageCreds;
pub use storage::{
  DynAsyncReader, ReadError as StorageReadError, StorageClientGenerator,
  WriteError as StorageWriteError,
};

#[cfg(feature = "mock-temp-storage")]
pub use self::mock::TempStorageRepositoryMock;

/// Descriptor trait for repositories that handle temp storage.
#[async_trait::async_trait]
pub trait TempStorageRepository: Hexagonal {
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

// impl for smart pointer to dyn TempStorageRepository
#[async_trait::async_trait]
impl<I> TempStorageRepository for Box<I>
where
  I: TempStorageRepository + ?Sized,
  Box<I>: Hexagonal,
{
  async fn read(
    &self,
    path: TempStoragePath,
  ) -> Result<DynAsyncReader, StorageReadError> {
    (**self).read(path).await
  }
  async fn store(
    &self,
    data: DynAsyncReader,
  ) -> Result<TempStoragePath, StorageWriteError> {
    (**self).store(data).await
  }
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
impl health::HealthReporter for TempStorageRepositoryCanonical {
  fn name(&self) -> &'static str { stringify!(TempStorageRepositoryCanonical) }
  async fn health_check(&self) -> health::ComponentHealth {
    health::AdditiveComponentHealth::start(self.client.health_report().await)
      .into()
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

#[cfg(feature = "mock-temp-storage")]
mod mock {
  use super::*;

  /// A mock repository for temp storage.
  #[derive(Clone)]
  pub struct TempStorageRepositoryMock {
    fs_root: std::path::PathBuf,
  }

  impl TempStorageRepositoryMock {
    /// Create a new instance of the temp storage repository.
    pub fn new(fs_root: std::path::PathBuf) -> Self {
      tracing::info!("creating new `TempStorageRepositoryMock` instance");
      Self { fs_root }
    }
  }

  #[async_trait::async_trait]
  impl health::HealthReporter for TempStorageRepositoryMock {
    fn name(&self) -> &'static str { stringify!(TempStorageRepositoryMock) }

    async fn health_check(&self) -> health::ComponentHealth {
      health::IntrensicallyUp.into()
    }
  }

  #[async_trait::async_trait]
  impl TempStorageRepository for TempStorageRepositoryMock {
    #[tracing::instrument(skip(self))]
    async fn read(
      &self,
      path: TempStoragePath,
    ) -> Result<DynAsyncReader, StorageReadError> {
      let path = self.fs_root.join(path.as_ref());
      let file = tokio::fs::File::open(path).await?;
      Ok(Box::new(file))
    }

    #[tracing::instrument(skip(self, data))]
    async fn store(
      &self,
      mut data: DynAsyncReader,
    ) -> Result<TempStoragePath, StorageWriteError> {
      // create fs_root if it doesn't exist
      tokio::fs::create_dir_all(&self.fs_root).await?;

      let path = TempStoragePath::new_random();
      let real_path = self.fs_root.join(path.as_ref());
      let mut file = tokio::fs::File::create(real_path).await?;
      tokio::io::copy(&mut data, &mut file).await?;
      Ok(path)
    }
  }
}
