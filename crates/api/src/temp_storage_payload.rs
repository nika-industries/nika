use std::io::{self};

use axum::{
  async_trait,
  body::{Body, Bytes},
  extract::{FromRef, FromRequest, Request},
  response::Response,
};
use prime_domain::{
  models::TempStoragePath,
  repos::belt::{self, Belt},
  DynPrimeDomainService, PrimeDomainService,
};
use tokio_stream::StreamExt;

use crate::AppState;

/// An extractor that reads the request body into temp storage.
pub struct TempStoragePayload(Body, DynPrimeDomainService);

#[async_trait]
impl<S> FromRequest<S> for TempStoragePayload
where
  Bytes: FromRequest<S>,
  S: Send + Sync,
  AppState: FromRef<S>,
{
  type Rejection = Response;

  async fn from_request(
    req: Request,
    state: &S,
  ) -> Result<Self, Self::Rejection> {
    let app_state = AppState::from_ref(state);

    Ok(Self(
      req.into_body(),
      app_state.prime_domain_service.clone(),
    ))
  }
}

#[allow(clippy::enum_variant_names)]
/// Error for [`TempStoragePayload::upload`].
#[derive(Debug, miette::Diagnostic, thiserror::Error)]
pub enum TempStoragePayloadError {
  /// Error writing to temp storage.
  #[error("Error writing to temp storage: {0}")]
  WriteError(prime_domain::StorageWriteError),
}

impl TempStoragePayload {
  pub async fn upload(
    self,
  ) -> Result<TempStoragePath, TempStoragePayloadError> {
    let TempStoragePayload(body, temp_storage_service) = self;

    let data = Belt::from_stream(
      body.into_data_stream().map(|res| {
        res.map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
      }),
      Some(belt::DEFAULT_CHUNK_SIZE),
    );
    let path = temp_storage_service
      .write_to_temp_storage(data)
      .await
      .map_err(TempStoragePayloadError::WriteError)?;

    Ok(path)
  }
}
