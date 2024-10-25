use std::path::Path;

use hex::{health, Hexagonal};
use repos::StorageClientGenerator;
use storage::{DynAsyncReader, ReadError, WriteError};

#[async_trait::async_trait]
pub trait UserStorageService: Hexagonal {
  type Client: UserStorageClient;
  async fn connect(
    &self,
    creds: models::StorageCredentials,
  ) -> miette::Result<Self::Client>;
}

#[async_trait::async_trait]
pub trait UserStorageClient: Hexagonal {
  /// Reads a file. Returns a [`DynAsyncReader`].
  async fn read(&self, path: &Path) -> Result<DynAsyncReader, ReadError>;
  /// Writes a file. Consumes a [`DynAsyncReader`].
  async fn write(
    &self,
    path: &Path,
    reader: DynAsyncReader,
  ) -> Result<models::FileSize, WriteError>;
}

#[async_trait::async_trait]
impl<T, I> UserStorageClient for T
where
  T: std::ops::Deref<Target = I> + Send + Sync + 'static,
  I: UserStorageClient + ?Sized,
{
  async fn read(&self, path: &Path) -> Result<DynAsyncReader, ReadError> {
    self.deref().read(path).await
  }
  async fn write(
    &self,
    path: &Path,
    reader: DynAsyncReader,
  ) -> Result<models::FileSize, WriteError> {
    self.deref().write(path, reader).await
  }
}

pub struct UserStorageClientCanonical(Box<dyn storage::StorageClient>);

impl UserStorageClientCanonical {
  fn new(client: Box<dyn storage::StorageClient>) -> Self { Self(client) }
}

#[async_trait::async_trait]
impl health::HealthReporter for UserStorageClientCanonical {
  fn name(&self) -> &'static str { self.0.name() }
  async fn health_check(&self) -> health::ComponentHealth {
    self.0.health_check().await
  }
}

#[async_trait::async_trait]
impl UserStorageClient for UserStorageClientCanonical {
  async fn read(&self, path: &Path) -> Result<DynAsyncReader, ReadError> {
    self.read(path).await
  }
  async fn write(
    &self,
    path: &Path,
    reader: DynAsyncReader,
  ) -> Result<models::FileSize, WriteError> {
    self.write(path, reader).await
  }
}

/// The canonical user storage service.
pub struct UserStorageServiceCanonical {}

impl UserStorageServiceCanonical {
  pub fn new() -> Self {
    tracing::info!("creating new `UserStorageServiceCanonical` instance");
    Self {}
  }
}

#[async_trait::async_trait]
impl health::HealthReporter for UserStorageServiceCanonical {
  fn name(&self) -> &'static str { stringify!(UserStorageServiceCanonical) }
  async fn health_check(&self) -> health::ComponentHealth {
    health::IntrensicallyUp.into()
  }
}

#[async_trait::async_trait]
impl UserStorageService for UserStorageServiceCanonical {
  type Client = UserStorageClientCanonical;

  async fn connect(
    &self,
    creds: models::StorageCredentials,
  ) -> miette::Result<Self::Client> {
    Ok(UserStorageClientCanonical::new(Box::new(
      creds.client().await?,
    )))
  }
}
