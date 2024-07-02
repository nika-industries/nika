use std::path::PathBuf;

pub type DynStorageClient = Box<dyn StorageClient + Send + Sync + 'static>;

pub enum StorageCredentials {
  LocalTmp(PathBuf),
}

impl StorageCredentials {
  pub async fn client(&self) -> DynStorageClient {
    match self {
      Self::LocalTmp(path) => Box::new(LocalTmpStorageClient(path.clone())),
    }
  }
}

pub struct LocalTmpStorageClient(PathBuf);

pub trait StorageClient {}

impl StorageClient for LocalTmpStorageClient {}
