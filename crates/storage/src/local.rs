use std::path::{Path, PathBuf};

use tokio::io::AsyncRead;

use super::{ReadError, StorageClient};

pub struct LocalStorageClient(PathBuf);

impl LocalStorageClient {
  pub fn new(path: PathBuf) -> Self { Self(path) }
}

#[async_trait::async_trait]
impl StorageClient for LocalStorageClient {
  async fn read(
    &self,
    path: &Path,
  ) -> Result<Box<dyn AsyncRead + Unpin>, ReadError> {
    let path = self.0.as_path().join(path);

    // make sure it exists
    if !std::fs::exists(&path)? {
      return Err(ReadError::NotFound(path));
    }

    let file = tokio::fs::File::open(&path).await?;

    Ok(Box::new(file))
  }
}
