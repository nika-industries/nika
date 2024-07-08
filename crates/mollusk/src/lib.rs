use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
};
use miette::Diagnostic;

pub trait ApiError: Diagnostic {
  /// The [`StatusCode`] that the error should return.
  fn status_code(&self) -> StatusCode;
  /// The unique slug for the error, to enable client-side handling.
  fn slug(&self) -> &'static str;
  /// The human-readable description for the error.
  fn description(&self) -> String;
  /// This method should run any logging or tracing calls attached to the error.
  fn tracing(&self);

  fn into_response(&self) -> axum::response::Response {
    self.tracing();
    return (
      self.status_code(),
      serde_json::json!({
        "error": {
          "id": self.slug(),
          "description": ApiError::description(self),
        },
      })
      .to_string(),
    )
      .into_response();
  }
}

pub trait RenderApiError<T> {
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
