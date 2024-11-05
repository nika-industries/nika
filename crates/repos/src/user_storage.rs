use std::path::Path;

use hex::{health, Hexagonal};
use storage::{
  CompAwareAReader, ReadError, StorageClientGenerator, WriteError,
};

/// The definition for the user storage service.
#[async_trait::async_trait]
pub trait UserStorageRepository: Hexagonal {
  /// The client type returned by the [`connect`](Self::connect_to_user_storage)
  /// method.
  type Client: UserStorageClient;
  /// Connects to user storage and returns a client.
  async fn connect_to_user_storage(
    &self,
    creds: models::StorageCredentials,
  ) -> miette::Result<Self::Client>;
}

/// The definition for the user storage client, produced by the
/// [`UserStorageRepository`].
#[async_trait::async_trait]
pub trait UserStorageClient: Hexagonal {
  /// Reads a file. Returns a [`DynAsyncReader`].
  async fn read(&self, path: &Path) -> Result<CompAwareAReader, ReadError>;
  /// Writes a file. Consumes a [`DynAsyncReader`].
  async fn write(
    &self,
    path: &Path,
    reader: CompAwareAReader,
  ) -> Result<models::FileSize, WriteError>;
}

#[async_trait::async_trait]
impl<T, I> UserStorageClient for T
where
  T: std::ops::Deref<Target = I> + Send + Sync + 'static,
  I: UserStorageClient + ?Sized,
{
  async fn read(&self, path: &Path) -> Result<CompAwareAReader, ReadError> {
    self.deref().read(path).await
  }
  async fn write(
    &self,
    path: &Path,
    reader: CompAwareAReader,
  ) -> Result<models::FileSize, WriteError> {
    self.deref().write(path, reader).await
  }
}

/// The canonical user storage client.
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
  async fn read(&self, path: &Path) -> Result<CompAwareAReader, ReadError> {
    self.0.read(path).await
  }
  async fn write(
    &self,
    path: &Path,
    reader: CompAwareAReader,
  ) -> Result<models::FileSize, WriteError> {
    self.0.write(path, reader).await
  }
}

/// The canonical user storage service.
pub struct UserStorageRepositoryCanonical {}

impl UserStorageRepositoryCanonical {
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
impl health::HealthReporter for UserStorageRepositoryCanonical {
  fn name(&self) -> &'static str { stringify!(UserStorageServiceCanonical) }
  async fn health_check(&self) -> health::ComponentHealth {
    health::IntrensicallyUp.into()
  }
}

#[async_trait::async_trait]
impl UserStorageRepository for UserStorageRepositoryCanonical {
  type Client = UserStorageClientCanonical;

  async fn connect_to_user_storage(
    &self,
    creds: models::StorageCredentials,
  ) -> miette::Result<Self::Client> {
    Ok(UserStorageClientCanonical::new(Box::new(
      creds.client().await?,
    )))
  }
}
