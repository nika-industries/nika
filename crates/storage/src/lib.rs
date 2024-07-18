mod local;
mod s3_compat;

use std::path::{Path, PathBuf};

use tokio::io::AsyncRead;

use self::{local::LocalStorageClient, s3_compat::S3CompatStorageClient};

pub type DynStorageClient = Box<dyn StorageClient + Send + Sync + 'static>;
pub type DynAsyncReader = Box<dyn AsyncRead + Send + Unpin + 'static>;

pub trait StorageClientGenerator {
  fn client(
    &self,
  ) -> impl std::future::Future<Output = miette::Result<DynStorageClient>> + Send;
}

impl StorageClientGenerator for core_types::StorageCredentials {
  async fn client(&self) -> miette::Result<DynStorageClient> {
    match self {
      Self::Local(local_storage_creds) => Ok(Box::new(
        LocalStorageClient::new(local_storage_creds.clone()).await?,
      )),
      Self::R2(r2_storage_creds) => Ok(Box::new(
        S3CompatStorageClient::new_r2(r2_storage_creds.clone()).await?,
      )),
    }
  }
}

#[derive(thiserror::Error, Debug, miette::Diagnostic)]
pub enum ReadError {
  #[error("the file was not available in storage: {0}")]
  NotFound(PathBuf),
  #[error("the supplied path was invalid: {0}")]
  InvalidPath(String),
  #[error("a local filesystem error occurred: {0}")]
  IoError(#[from] std::io::Error),
}

#[derive(thiserror::Error, Debug, miette::Diagnostic)]
pub enum WriteError {
  #[error("the supplied path was invalid: {0}")]
  InvalidPath(String),
  #[error("a local filesystem error occurred: {0}")]
  IoError(#[from] std::io::Error),
  #[error("an error occurred while performing a multipart upload: {0}")]
  MultipartError(miette::Report),
}

#[async_trait::async_trait]
pub trait StorageClient {
  async fn read(&self, path: &Path) -> Result<DynAsyncReader, ReadError>;
  async fn write(
    &self,
    path: &Path,
    reader: DynAsyncReader,
  ) -> Result<(), WriteError>;
}
