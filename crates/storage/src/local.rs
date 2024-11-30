use std::path::{Path, PathBuf};

use belt::Belt;
use dvf::LocalStorageCredentials;
use hex::health;
use miette::{Context, IntoDiagnostic};
use tokio::io::{AsyncWriteExt, BufReader, BufWriter};

use super::{ReadError, StorageClient};
use crate::WriteError;

pub struct LocalStorageClient(PathBuf);

impl LocalStorageClient {
  pub async fn new(creds: LocalStorageCredentials) -> miette::Result<Self> {
    Ok(Self(
      creds
        .0
        .canonicalize()
        .into_diagnostic()
        .wrap_err("failed to canonicalize path for `LocalStorageClient`")?
        .to_path_buf(),
    ))
  }
}

#[async_trait::async_trait]
impl health::HealthReporter for LocalStorageClient {
  fn name(&self) -> &'static str { stringify!(LocalStorageClient) }
  async fn health_check(&self) -> health::ComponentHealth {
    health::IntrensicallyUp.into()
  }
}

#[async_trait::async_trait]
impl StorageClient for LocalStorageClient {
  #[tracing::instrument(skip(self))]
  async fn read(&self, input_path: &Path) -> Result<Belt, ReadError> {
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

    Ok(Belt::from_async_buf_read(
      BufReader::new(file),
      Some(belt::DEFAULT_CHUNK_SIZE),
    ))
  }

  #[tracing::instrument(skip(self))]
  async fn write(
    &self,
    path: &Path,
    data: Belt,
  ) -> Result<dvf::FileSize, WriteError> {
    let target_path = self.0.as_path().join(path);

    // Ensure the directory structure exists
    if let Some(parent) = target_path.parent() {
      tokio::fs::create_dir_all(parent).await?;
    }

    // Create and open the target file
    let file = tokio::fs::File::create(&target_path).await?;
    let mut writer = BufWriter::new(file);

    let counter = data.counter();
    let mut data = data.to_async_buf_read();

    // Copy data from the input to the writer
    tokio::io::copy(&mut data, &mut writer).await?;

    // Ensure all data is flushed to the file
    writer.flush().await?;

    let file_size = dvf::FileSize::new(counter.current());

    Ok(file_size)
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

    let client = LocalStorageClient::new(LocalStorageCredentials(
      temp.path().to_path_buf(),
    ))
    .await
    .unwrap();
    let data = client
      .read(&PathBuf::from_str("file1").unwrap())
      .await
      .unwrap();

    let mut result = String::new();
    data
      .to_async_buf_read()
      .read_to_string(&mut result)
      .await
      .unwrap();

    assert_eq!(&result, "abc");
  }
}
