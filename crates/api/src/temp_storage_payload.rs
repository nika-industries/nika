use std::{
  io::{self},
  sync::Arc,
};

use axum::{
  async_trait,
  body::{Body, Bytes},
  extract::{FromRef, FromRequest, Request},
  response::Response,
};
use prime_domain::{models::TempStoragePath, TempStorageService};
use tokio_stream::StreamExt;

use crate::AppState;

/// An extractor that reads the request body into temp storage.
pub struct TempStoragePayload(Body, Arc<Box<dyn TempStorageService>>);

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
      app_state.temp_storage_service.clone(),
    ))
  }
}

#[allow(clippy::enum_variant_names)]
/// Error for [`TempStoragePayload::upload`].
#[derive(Debug, miette::Diagnostic, thiserror::Error)]
pub enum TempStoragePayloadError {
  /// Error writing to temp storage.
  #[error("Error writing to temp storage: {0}")]
  WriteError(storage::WriteError),
}

impl TempStoragePayload {
  pub async fn upload(
    self,
  ) -> Result<TempStoragePath, TempStoragePayloadError> {
    let TempStoragePayload(body, temp_storage_service) = self;

    let body_stream = Box::new(tokio_util::io::StreamReader::new(
      body.into_data_stream().map(|result| {
        result.map_err(|err| io::Error::new(io::ErrorKind::Other, err))
      }),
    ));

    let path = temp_storage_service
      .store(body_stream)
      .await
      .map_err(TempStoragePayloadError::WriteError)?;

    Ok(path)
  }
}
