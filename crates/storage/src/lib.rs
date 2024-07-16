pub mod local;

use std::path::{Path, PathBuf};

use tokio::io::AsyncRead;

use self::local::LocalStorageClient;

pub type DynStorageClient = Box<dyn StorageClient + Send + Sync + 'static>;
pub type DynAsyncReader = Box<dyn AsyncRead + Send + Sync + Unpin + 'static>;

pub trait StorageClientGenerator {
  fn client(
    &self,
  ) -> impl std::future::Future<Output = DynStorageClient> + Send;
}

impl StorageClientGenerator for core_types::StorageCredentials {
  async fn client(&self) -> DynStorageClient {
    match self {
      Self::Local(local_storage_creds) => {
        Box::new(LocalStorageClient::new(local_storage_creds.0.clone()))
      }
    }
  }
}

#[derive(thiserror::Error, Debug)]
pub enum ReadError {
  #[error("the file was not available in storage: {0}")]
  NotFound(PathBuf),
  #[error("the supplied path was invalid: {0}")]
  InvalidPath(String),
  #[error("a local filesystem error occurred: {0}")]
  IoError(#[from] std::io::Error),
}

#[async_trait::async_trait]
pub trait StorageClient {
  async fn read(&self, path: &Path) -> Result<DynAsyncReader, ReadError>;
}
