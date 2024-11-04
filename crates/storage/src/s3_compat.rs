use std::{path::Path, sync::Arc};

use bytes_stream::BytesStream;
use futures::{StreamExt, TryFutureExt, TryStreamExt};
use hex::health;
use miette::{Context, IntoDiagnostic, Report};
use models::R2StorageCredentials;
use object_store::{
  aws::{AmazonS3, AmazonS3Builder},
  Error as ObjectStoreError, ObjectStore, PutPayload,
};
use stream_tools::CountedAsyncReader;
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
impl health::HealthReporter for S3CompatStorageClient {
  fn name(&self) -> &'static str { stringify!(S3CompatStorageClient) }
  async fn health_check(&self) -> health::ComponentHealth {
    let result = self
      .store
      .head(&object_store::path::Path::parse("a").unwrap())
      .await;
    health::SingularComponentHealth::new(match result {
      Ok(_) => health::HealthStatus::Ok,
      Err(e) => match e {
        ObjectStoreError::NotFound { .. } => health::HealthStatus::Ok,
        _ => health::HealthStatus::Down(vec![health::FailureMessage::new(
          &format!("failed to perform heartbeat request: {e:?}"),
        )]),
      },
    })
    .into()
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
  ) -> Result<models::FileSize, WriteError> {
    // sanitize the destination path
    let input_path_string = input_path.to_str().unwrap().to_string();
    let path = object_store::path::Path::parse(input_path_string.clone())
      .map_err(|_| WriteError::InvalidPath(input_path_string))?;

    // chunk size of 10MB
    let chunk_size = 10 * 1024 * 1024;

    let (mut reader, counter) = CountedAsyncReader::new(&mut reader);

    // create a stream of bytes chunks from the reader
    let bytes_chunks = tokio_util::io::ReaderStream::new(
      tokio::io::BufReader::with_capacity(chunk_size, &mut reader),
    )
    .bytes_chunks(chunk_size);

    // initiate the multipart with the destination storage
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

    // create a stream of multipart `put_part` futures, running 3 at a time
    let part_stream = bytes_chunks
      .map_err(|e| {
        Report::from_err(e).wrap_err("failed to get bytes chunk from reader")
      })
      .map(|r| {
        let value = multipart.clone();
        async move {
          let chunk = r?;
          tracing::info!("got bytes chunk with {} bytes", chunk.len());

          let mut multipart = value.lock().await;
          let future = multipart
            .put_part(PutPayload::from_bytes(chunk))
            .map_err(|e| Report::from_err(e).wrap_err("failed to upload part"));
          drop(multipart);
          future.await?;

          Ok::<_, Report>(())
        }
      })
      .buffered(3);

    // wait for all parts to be uploaded
    let _ = part_stream
      .try_collect::<Vec<_>>()
      .await
      .map_err(WriteError::MultipartError)?;

    // complete the multipart
    multipart
      .lock()
      .await
      .complete()
      .await
      .into_diagnostic()
      .wrap_err("failed to complete multipart")
      .map_err(WriteError::MultipartError)?;

    tracing::info!("finishing multipart");

    let file_size = models::FileSize::new(counter.current_size().await);

    Ok(file_size)
  }
}
