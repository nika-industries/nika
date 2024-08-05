use std::{
  io::{self},
  path::PathBuf,
  str::FromStr,
};

use axum::{
  async_trait,
  body::{Body, Bytes},
  extract::{FromRequest, Request},
  response::Response,
};
use storage::{temp::TempStoragePath, StorageClientGenerator};
use tokio_stream::StreamExt;

/// An extractor that reads the request body into temp storage.
pub struct TempStoragePayload(Body);

#[async_trait]
impl<S> FromRequest<S> for TempStoragePayload
where
  Bytes: FromRequest<S>,
  S: Send + Sync,
{
  type Rejection = Response;

  async fn from_request(
    req: Request,
    _state: &S,
  ) -> Result<Self, Self::Rejection> {
    Ok(Self(req.into_body()))
  }
}

#[allow(clippy::enum_variant_names)]
/// Error for [`TempStoragePayload::upload`].
#[derive(Debug, miette::Diagnostic, thiserror::Error)]
pub enum TempStoragePayloadError {
  /// Error writing to temp storage.
  #[error("Error writing to temp storage: {0}")]
  WriteError(storage::WriteError),
  /// Error getting temp storage credentials
  #[error("Error getting temp storage credentials: {0}")]
  CredsError(storage::temp::TempStorageCredsError),
  /// Error creating a storage client
  #[error("Error creating a storage client: {0}")]
  ClientError(miette::Report),
}

impl TempStoragePayload {
  pub async fn upload(
    self,
  ) -> Result<TempStoragePath, TempStoragePayloadError> {
    let client = storage::temp::get_temp_storage_creds()
      .map_err(TempStoragePayloadError::CredsError)?
      .client()
      .await
      .map_err(TempStoragePayloadError::ClientError)?;

    let body_stream = Box::new(tokio_util::io::StreamReader::new(
      self.0.into_data_stream().map(|result| {
        result.map_err(|err| io::Error::new(io::ErrorKind::Other, err))
      }),
    ));

    let path = PathBuf::from_str(&core_types::Ulid::new().to_string()).unwrap();
    client
      .write(&path, body_stream)
      .await
      .map_err(TempStoragePayloadError::WriteError)?;

    Ok(TempStoragePath(path))
  }
}
