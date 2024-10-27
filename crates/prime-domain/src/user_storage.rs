use std::{path::Path, sync::Arc};

use hex::{health, Hexagonal};
use repos::StorageClientGenerator;
use storage::{DynAsyncReader, ReadError, WriteError};

/// A dynamic [`UserStorageService`] trait object.
pub type DynUserStorageService =
  Arc<Box<dyn UserStorageService<Client = UserStorageClientCanonical>>>;

/// The definition for the user storage service.
#[async_trait::async_trait]
pub trait UserStorageService: Hexagonal {
  /// The client type returned by the [`connect`] method.
  type Client: UserStorageClient;
  /// Connects to user storage and returns a client.
  async fn connect(
    &self,
    creds: models::StorageCredentials,
  ) -> miette::Result<Self::Client>;
}

/// The definition for the user storage client, produced by the
/// [`UserStorageService`].
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
    self.0.read(path).await
  }
  async fn write(
    &self,
    path: &Path,
    reader: DynAsyncReader,
  ) -> Result<models::FileSize, WriteError> {
    self.0.write(path, reader).await
  }
}

/// The canonical user storage service.
pub struct UserStorageServiceCanonical {}

impl UserStorageServiceCanonical {
  /// Create a new instance of the canonical user storage service.
  #[allow(
    clippy::new_without_default,
    reason = "Service construction still should not be flippant, despite this \
              being stateless."
  )]
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
