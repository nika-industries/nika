//! Provides standardized API schemas.

use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
};
use miette::Diagnostic;

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
  fn into_response(self) -> Response {
    self.tracing();
    (
      self.status_code(),
      serde_json::json!({
        "error": {
          "id": self.slug(),
          "description": ApiError::description(&self),
        },
      })
      .to_string(),
    )
      .into_response()
  }
}

/// Extension trait that adds the `render_api_error` to `Result<T, E: ApiError>`
pub trait RenderApiError<T> {
  /// Converts `Result<T, E: ApiError>` into `Result<T, Response>`
  fn render_api_error(self) -> Result<T, Response>;
}

impl<T, E> RenderApiError<T> for Result<T, E>
where
  E: ApiError,
{
  fn render_api_error(self) -> Result<T, Response> {
    self.map_err(|e| e.into_response())
  }
}
