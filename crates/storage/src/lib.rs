pub mod local;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tokio::io::AsyncRead;

use self::local::LocalStorageClient;

pub type DynStorageClient = Box<dyn StorageClient + Send + Sync + 'static>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum StorageCredentials {
  Local(PathBuf),
}

impl StorageCredentials {
  pub async fn client(&self) -> DynStorageClient {
    match self {
      Self::Local(path) => Box::new(LocalStorageClient::new(path.clone())),
    }
  }
}

#[derive(thiserror::Error, Debug)]
pub enum ReadError {
  #[error("the file was not available in storage: {0}")]
  NotFound(PathBuf),
  #[error("a local filesystem error occurred: {0}")]
  IoError(#[from] std::io::Error),
}

#[async_trait::async_trait]
pub trait StorageClient {
  async fn read(
    &self,
    path: &Path,
  ) -> Result<Box<dyn AsyncRead + Unpin>, ReadError>;
}
