//! Provides traits and implementations for storage clients.

mod local;
mod s3_compat;
pub mod temp;

use std::path::{Path, PathBuf};

pub use belt;
use belt::Belt;
use hex::Hexagonal;

use self::{local::LocalStorageClient, s3_compat::S3CompatStorageClient};

/// Trait alias for `Box<dyn StorageClient + ...>`
pub type DynStorageClient = Box<dyn StorageClient + Send + Sync + 'static>;

/// Extension trait that allows generating a dynamic client from
/// `StorageCredentials`.
pub trait StorageClientGenerator {
  /// Generates a dynamic client from `StorageCredentials`.
  fn client(
    &self,
  ) -> impl std::future::Future<Output = miette::Result<DynStorageClient>> + Send;
}

impl StorageClientGenerator for dvf::StorageCredentials {
  async fn client(&self) -> miette::Result<DynStorageClient> {
    match self {
      Self::Local(local_storage_creds) => Ok(Box::new(
        LocalStorageClient::new(local_storage_creds.clone()).await?,
      ) as DynStorageClient),
      Self::R2(r2_storage_creds) => Ok(Box::new(
        S3CompatStorageClient::new_r2(r2_storage_creds.clone()).await?,
      ) as DynStorageClient),
    }
  }
}

/// An error type used when reading from a `StorageClient`.
#[derive(thiserror::Error, Debug, miette::Diagnostic)]
pub enum ReadError {
  /// The path was not found in the storage.
  #[error("the file was not available in storage: {0}")]
  NotFound(PathBuf),
  /// The path was invalid.
  #[error("the supplied path was invalid: {0}")]
  InvalidPath(String),
  /// An IO error occurred.
  #[error("a local filesystem error occurred: {0}")]
  IoError(#[from] std::io::Error),
}

/// An error type used when writing to a `StorageClient`.
#[derive(thiserror::Error, Debug, miette::Diagnostic)]
pub enum WriteError {
  /// The path was invalid.
  #[error("the supplied path was invalid: {0}")]
  InvalidPath(String),
  /// An IO error occurred.
  #[error("a local filesystem error occurred: {0}")]
  IoError(#[from] std::io::Error),
  /// An error occurred while uploading a multipart.
  #[error("an error occurred while performing a multipart upload: {0}")]
  MultipartError(miette::Report),
}

/// The main storage trait. Allows reading to or writing from a stream of bytes.
#[async_trait::async_trait]
pub trait StorageClient: Hexagonal {
  /// Reads a file. Returns a [`Belt`].
  async fn read(&self, path: &Path) -> Result<Belt, ReadError>;
  /// Writes a file. Consumes a [`CompUnawareAReader`].
  async fn write(
    &self,
    path: &Path,
    data: Belt,
  ) -> Result<dvf::FileSize, WriteError>;
}

#[async_trait::async_trait]
impl<T, I> StorageClient for T
where
  T: std::ops::Deref<Target = I> + Send + Sync + 'static,
  I: StorageClient + ?Sized,
{
  async fn read(&self, path: &Path) -> Result<Belt, ReadError> {
    self.deref().read(path).await
  }
  async fn write(
    &self,
    path: &Path,
    data: Belt,
  ) -> Result<dvf::FileSize, WriteError> {
    self.deref().write(path, data).await
  }
}
