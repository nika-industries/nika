use std::path::Path;

use core_types::R2StorageCredentials;
use futures::{io::BufReader, TryStreamExt};
use miette::{Context, IntoDiagnostic};
use object_store::{
  aws::{AmazonS3, AmazonS3Builder},
  Error as ObjectStoreError, ObjectStore,
};
use tokio_util::compat::FuturesAsyncReadCompatExt;

use super::{DynAsyncReader, ReadError, StorageClient};

pub struct R2StorageClient {
  store: AmazonS3,
}

impl R2StorageClient {
  pub async fn new(creds: R2StorageCredentials) -> miette::Result<Self> {
    match creds {
      R2StorageCredentials::Default {
        access_key,
        secret_access_key,
        endpoint,
        bucket,
      } => {
        let r2 = AmazonS3Builder::new()
          .with_endpoint(endpoint)
          .with_access_key_id(access_key)
          .with_secret_access_key(secret_access_key)
          .with_bucket_name(bucket)
          .with_client_options(
            object_store::ClientOptions::new()
              .with_allow_http2()
              .with_timeout_disabled(),
          )
          .build()
          .into_diagnostic()
          .wrap_err("failed to build R2 client instance")?;
        Ok(R2StorageClient { store: r2 })
      }
    }
  }
}

#[async_trait::async_trait]
impl StorageClient for R2StorageClient {
  async fn read(&self, input_path: &Path) -> Result<DynAsyncReader, ReadError> {
    let input_path_string = input_path.to_str().unwrap().to_string();
    let path = object_store::path::Path::parse(input_path_string.clone())
      .map_err(|_| ReadError::InvalidPath(input_path_string))?;

    let get_result = self.store.get(&path).await.map_err(|e| {
      if let ObjectStoreError::NotFound { .. } = e {
        ReadError::NotFound(input_path.to_path_buf())
      } else {
        Err(e).into_diagnostic().unwrap()
      }
    })?;

    let stream = get_result
      .into_stream()
      .map_err(futures_util::io::Error::from)
      .into_async_read();

    Ok(Box::new(stream.compat()))
  }
}
