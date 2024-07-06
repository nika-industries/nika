use std::path::{Path, PathBuf};

use super::{DynAsyncReader, ReadError, StorageClient};

pub struct LocalStorageClient(PathBuf);

impl LocalStorageClient {
  pub fn new(path: PathBuf) -> Self {
    Self(
      path
        .canonicalize()
        .expect("Failed to canonicalize path for `LocalStorageClient`")
        .to_path_buf(),
    )
  }
}

#[async_trait::async_trait]
impl StorageClient for LocalStorageClient {
  async fn read(&self, input_path: &Path) -> Result<DynAsyncReader, ReadError> {
    let path = self.0.as_path().join(input_path);

    // make sure it exists
    if !std::fs::exists(&path)? {
      return Err(ReadError::NotFound(input_path.to_path_buf()));
    }

    // canonicalize to remove relative segments and symlinks
    let path = path.canonicalize().map_err(|_| {
      ReadError::InvalidPath(input_path.to_string_lossy().to_string())
    })?;

    // make sure that it doesn't escape the store path
    //   we assume it has no relative segments because of the `canonicalize()`
    if !path.starts_with(&self.0) {
      return Err(ReadError::InvalidPath(
        input_path.to_string_lossy().to_string(),
      ));
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
  async fn read_works() {
    let temp = TempDir::new().unwrap();

    let f = temp.child("file1");
    std::fs::write(&f, "abc").unwrap();

    let client = LocalStorageClient::new(temp.path().to_path_buf());
    let mut reader = client
      .read(&PathBuf::from_str("file1").unwrap())
      .await
      .unwrap();

    let mut result = String::new();
    reader.read_to_string(&mut result).await.unwrap();

    assert_eq!(&result, "abc");
  }
}
