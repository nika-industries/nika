use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tokio::io::AsyncRead;

pub type DynStorageClient = Box<dyn StorageClient + Send + Sync + 'static>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum StorageCredentials {
  Local(PathBuf),
}

impl StorageCredentials {
  pub async fn client(&self) -> DynStorageClient {
    match self {
      Self::Local(path) => Box::new(LocalStorageClient(path.clone())),
    }
  }
}

pub struct LocalStorageClient(PathBuf);

#[derive(thiserror::Error, Debug)]
pub enum ReadError {
  #[error("the file was not available in storage: {0}")]
  NotFound(PathBuf),
  #[error("a local filesystem error occurred: {0}")]
  IoError(#[from] std::io::Error),
}

#[async_trait::async_trait]
pub trait StorageClient {
  async fn read(&self, path: &Path) -> Result<Box<dyn AsyncRead>, ReadError>;
}

#[async_trait::async_trait]
impl StorageClient for LocalStorageClient {
  async fn read(&self, path: &Path) -> Result<Box<dyn AsyncRead>, ReadError> {
    let path = self.0.as_path().join(path);

    if !std::fs::exists(&path)? {
      return Err(ReadError::NotFound(path));
    }

    let file = tokio::fs::File::open(&path).await?;

    Ok(Box::new(file))
  }
}
