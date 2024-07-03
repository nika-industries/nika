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
  ) -> Result<Box<dyn AsyncRead + Send + Sync + Unpin + 'static>, ReadError> {
    let path = self.0.as_path().join(path);

    // make sure it exists
    if !std::fs::exists(&path)? {
      return Err(ReadError::NotFound(path));
    }

    let file = tokio::fs::File::open(&path).await?;

    Ok(Box::new(file))
  }
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;

  use temp_dir::TempDir;
  use tokio::io::AsyncReadExt;

  use super::*;

  #[tokio::test]
  async fn it_works() {
    let temp = TempDir::new().unwrap();

    let f = temp.child("file1");
    std::fs::write(&f, "abc").unwrap();

    let client = LocalStorageClient::new(temp.path().to_path_buf());
    let mut reader = client
      .read(&PathBuf::from_str("file1").unwrap())
      .await
      .unwrap();

    let mut result = String::new();
    (&mut reader).read_to_string(&mut result).await.unwrap();

    assert_eq!(&result, "abc");
  }
}
