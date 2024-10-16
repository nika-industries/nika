//! Provides standardized API schemas and errors for inter-service use.

mod common;
mod confirm_token_by_secret_has_permission_error;
mod creds_fetching_error;
mod prepare_fetch_payload_error;

use axum_core::response::{IntoResponse, Response};
use http::StatusCode;
use miette::Diagnostic;
use serde::Serialize;

use self::axum_json::Json;
pub use self::{
  common::*,
  confirm_token_by_secret_has_permission_error::ConfirmTokenBySecretHasPermissionError,
  creds_fetching_error::CredsFetchingError,
  prepare_fetch_payload_error::PrepareFetchPayloadError,
};

mod axum_json {
  use axum_core::response::{IntoResponse, Response};
  use bytes::{BufMut, BytesMut};
  use http::{header, HeaderValue, StatusCode};
  use serde::Serialize;

  #[derive(Debug, Clone, Copy, Default)]
  pub struct Json<T>(pub T);

  impl<T> IntoResponse for Json<T>
  where
    T: Serialize,
  {
    fn into_response(self) -> Response {
      // Use a small initial capacity of 128 bytes like serde_json::to_vec
      // https://docs.rs/serde_json/1.0.82/src/serde_json/ser.rs.html#2189
      let mut buf = BytesMut::with_capacity(128).writer();
      match serde_json::to_writer(&mut buf, &self.0) {
        Ok(()) => (
          [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
          )],
          buf.into_inner().freeze(),
        )
          .into_response(),
        Err(err) => (
          StatusCode::INTERNAL_SERVER_ERROR,
          [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
          )],
          err.to_string(),
        )
          .into_response(),
      }
    }
  }
}

/// An error that can be directly returned to a user from an API route.
pub trait MolluskError: Diagnostic + Sized {
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
          "description": MolluskError::description(&self),
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
  E: MolluskError + Serialize,
{
  fn render_internal_error(self) -> Result<T, InternalApiError> {
    self.map_err(|e| InternalApiError::from(e))
  }
}

impl<T: MolluskError + Serialize> From<T> for InternalApiError {
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
  E: MolluskError,
{
  fn render_external_error(self) -> Result<T, ExternalApiError> {
    self.map_err(|e| ExternalApiError::from(e))
  }
}

impl<T: MolluskError> From<T> for ExternalApiError {
  fn from(e: T) -> Self { ExternalApiError(e.into_external_response()) }
}

/// A macro to delegate the `MolluskError` trait to an enum of errors.
#[macro_export]
macro_rules! delegate_mollusk_error {
  ($enum_name:ident, $($variant:ident),+ $(,)?) => {
    impl MolluskError for $enum_name {
      fn status_code(&self) -> StatusCode {
        match self {
          $(Self::$variant(e) => e.status_code()),+
        }
      }

      fn slug(&self) -> &'static str {
        match self {
          $(Self::$variant(e) => e.slug()),+
        }
      }

      fn description(&self) -> String {
        match self {
          $(Self::$variant(e) => e.description()),+
        }
      }

      fn tracing(&self) {
        match self {
          $(Self::$variant(e) => e.tracing()),+
        }
      }
    }
  };
}
