use std::{path::Path, sync::Arc};

use bytes_stream::BytesStream;
use models::R2StorageCredentials;
use futures::{StreamExt, TryFutureExt, TryStreamExt};
use miette::{Context, IntoDiagnostic};
use object_store::{
  aws::{AmazonS3, AmazonS3Builder},
  Error as ObjectStoreError, ObjectStore, PutPayload,
};
use tokio::sync::Mutex;
use tokio_util::compat::FuturesAsyncReadCompatExt;

use super::{DynAsyncReader, ReadError, StorageClient};
use crate::WriteError;

pub struct S3CompatStorageClient {
  store: AmazonS3,
}

impl S3CompatStorageClient {
  pub async fn new_r2(creds: R2StorageCredentials) -> miette::Result<Self> {
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
        Ok(S3CompatStorageClient { store: r2 })
      }
    }
  }
}

#[async_trait::async_trait]
impl StorageClient for S3CompatStorageClient {
  #[tracing::instrument(skip(self))]
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
      .map_err(|e| {
        tracing::error!("error while streaming from R2 store: {e:?}");
        futures::io::Error::from(e)
      })
      .into_async_read();

    Ok(Box::new(futures::io::BufReader::new(stream).compat()))
  }

  #[tracing::instrument(skip(self, reader))]
  async fn write(
    &self,
    input_path: &Path,
    mut reader: DynAsyncReader,
  ) -> Result<(), WriteError> {
    let input_path_string = input_path.to_str().unwrap().to_string();
    let path = object_store::path::Path::parse(input_path_string.clone())
      .map_err(|_| WriteError::InvalidPath(input_path_string))?;

    let chunk_size = 10 * 1024 * 1024;

    let bytes_chunks = tokio_util::io::ReaderStream::new(
      tokio::io::BufReader::with_capacity(chunk_size, &mut reader),
    )
    .bytes_chunks(chunk_size);

    tracing::info!("starting multipart");
    let multipart = Arc::new(Mutex::new(
      self
        .store
        .put_multipart(&path)
        .await
        .into_diagnostic()
        .wrap_err("failed to start multipart")
        .map_err(WriteError::MultipartError)?,
    ));

    let part_stream = bytes_chunks
      .map_err(|e| {
        Err::<(), _>(e)
          .into_diagnostic()
          .wrap_err("failed to get bytes chunk from reader")
          .unwrap_err()
      })
      .map(|r| {
        let value = multipart.clone();
        async move {
          let chunk = r?;
          tracing::info!("got bytes chunk with {} bytes", chunk.len());

          let mut multipart = value.lock().await;
          let future = multipart
            .put_part(PutPayload::from_bytes(chunk))
            .map_err(|e| {
              Err::<(), _>(e)
                .into_diagnostic()
                .wrap_err("failed to upload part")
                .unwrap_err()
            });
          drop(multipart);
          future.await?;

          Ok::<_, miette::Report>(())
        }
      })
      .buffered(3);

    let _ = part_stream
      .try_collect::<Vec<_>>()
      .await
      .map_err(WriteError::MultipartError)?;

    multipart
      .lock()
      .await
      .complete()
      .await
      .into_diagnostic()
      .wrap_err("failed to complete multipart")
      .map_err(WriteError::MultipartError)?;

    tracing::info!("finishing multipart");

    Ok(())
  }
}
