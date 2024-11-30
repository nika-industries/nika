use std::sync::Arc;

use hex::{
  health::{self},
  Hexagonal,
};
use models::TempStoragePath;
use storage::{belt::Belt, temp::TempStorageCreds};
pub use storage::{
  ReadError as StorageReadError, StorageClientGenerator,
  WriteError as StorageWriteError,
};

pub use self::mock::TempStorageRepositoryMock;

/// Descriptor trait for repositories that handle temp storage.
#[async_trait::async_trait]
pub trait TempStorageRepository: Hexagonal {
  /// Read data from the storage.
  async fn read(&self, path: TempStoragePath)
    -> Result<Belt, StorageReadError>;
  /// Store data in the storage.
  async fn store(
    &self,
    data: Belt,
  ) -> Result<TempStoragePath, StorageWriteError>;
}

#[async_trait::async_trait]
impl<T, I> TempStorageRepository for T
where
  T: std::ops::Deref<Target = I> + Send + Sync + 'static,
  I: TempStorageRepository + ?Sized,
{
  async fn read(
    &self,
    path: TempStoragePath,
  ) -> Result<Belt, StorageReadError> {
    self.deref().read(path).await
  }
  async fn store(
    &self,
    data: Belt,
  ) -> Result<TempStoragePath, StorageWriteError> {
    self.deref().store(data).await
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
    health::AdditiveComponentHealth::from_futures(Some(
      self.client.health_report(),
    ))
    .await
    .into()
  }
}

#[async_trait::async_trait]
impl TempStorageRepository for TempStorageRepositoryCanonical {
  #[tracing::instrument(skip(self))]
  async fn read(
    &self,
    path: TempStoragePath,
  ) -> Result<Belt, StorageReadError> {
    self.client.read(path.path()).await
  }

  #[tracing::instrument(skip(self, data))]
  async fn store(
    &self,
    data: Belt,
  ) -> Result<TempStoragePath, StorageWriteError> {
    let mut path = TempStoragePath::new_random(models::FileSize::new(0));
    let counter = data.counter();
    self.client.write(path.path(), data).await?;
    path.set_size(models::FileSize::new(counter.current()));
    Ok(path)
  }
}

mod mock {
  use storage::belt;

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
    ) -> Result<Belt, StorageReadError> {
      let path = self.fs_root.join(path.path());
      let file = tokio::fs::File::open(path).await?;
      Ok(Belt::from_async_read(file, Some(belt::DEFAULT_CHUNK_SIZE)))
    }

    #[tracing::instrument(skip(self, data))]
    async fn store(
      &self,
      data: Belt,
    ) -> Result<TempStoragePath, StorageWriteError> {
      // create fs_root if it doesn't exist
      tokio::fs::create_dir_all(&self.fs_root).await?;

      let mut path = TempStoragePath::new_random(models::FileSize::new(0));
      let real_path = self.fs_root.join(path.path());

      let counter = data.counter();
      let mut file = tokio::fs::File::create(real_path).await?;
      tokio::io::copy(&mut data.to_async_buf_read(), &mut file).await?;
      path.set_size(models::FileSize::new(counter.current()));
      Ok(path)
    }
  }
}
