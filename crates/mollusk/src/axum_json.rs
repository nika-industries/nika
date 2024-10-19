//! Taken straight from the axum implementation of `Json` to avoid a
//! dependency on axum.

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
