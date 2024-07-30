//! Provides standardized API schemas and errors for inter-service use.

mod creds_fetching_error;

use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use miette::Diagnostic;
use serde::Serialize;

pub use self::creds_fetching_error::CredsFetchingError;

/// An error that can be directly returned to a user from an API route.
pub trait ApiError: Diagnostic + Sized {
  /// The [`StatusCode`] that the error should return.
  fn status_code(&self) -> StatusCode;
  /// The unique slug for the error, to enable client-side handling.
  fn slug(&self) -> &'static str;
  /// The human-readable description for the error.
  fn description(&self) -> String;
  /// This method should run any logging or tracing calls attached to the error.
  fn tracing(&self);

  /// Converts the API error into an [`axum`] [`Response`].
  fn into_external_response(self) -> Response {
    self.tracing();
    (
      self.status_code(),
      Json(serde_json::json!({
        "error": {
          "id": self.slug(),
          "description": ApiError::description(&self),
        },
      })),
    )
      .into_response()
  }
}

/// Wrapper for internally published API errors.
pub struct InternalApiError(Response);

impl IntoResponse for InternalApiError {
  fn into_response(self) -> Response { self.0 }
}

/// Wrapper for externally published API errors.
pub struct ExternalApiError(Response);

impl IntoResponse for ExternalApiError {
  fn into_response(self) -> Response { self.0 }
}

/// Extension trait that adds the `render_internal_error` methods to `Result<T,
/// E: ApiError>`. Requires the error to be serializable.
pub trait RenderInternalError<T> {
  /// Converts `Result<T, E: ApiError>` into `Result<T, InternalApiError>`
  fn render_internal_error(self) -> Result<T, InternalApiError>;
}

impl<T, E> RenderInternalError<T> for Result<T, E>
where
  E: ApiError + Serialize,
{
  fn render_internal_error(self) -> Result<T, InternalApiError> {
    self.map_err(|e| InternalApiError::from(e))
  }
}

impl<T: ApiError + Serialize> From<T> for InternalApiError {
  fn from(e: T) -> Self {
    e.tracing();
    InternalApiError((e.status_code(), Json(e)).into_response())
  }
}

/// Extension trait that adds the `render_external_error` methods to `Result<T,
/// E: ApiError>`.
pub trait RenderExternalError<T> {
  /// Converts `Result<T, E: ApiError>` into `Result<T, ExternalApiError>`
  fn render_external_error(self) -> Result<T, ExternalApiError>;
}

impl<T, E> RenderExternalError<T> for Result<T, E>
where
  E: ApiError,
{
  fn render_external_error(self) -> Result<T, ExternalApiError> {
    self.map_err(|e| ExternalApiError::from(e))
  }
}

impl<T: ApiError> From<T> for ExternalApiError {
  fn from(e: T) -> Self { ExternalApiError(e.into_external_response()) }
}
