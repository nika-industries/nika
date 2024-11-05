//! Various stream tools.

mod counted_async_reader;
pub mod read_jump;

use std::{
  io::Read,
  pin::Pin,
  task::{Context, Poll},
};

pub use tokio::io::AsyncRead;

pub use self::{
  counted_async_reader::CountedAsyncReader, read_jump::wrap_with_read_jump,
};

/// A stream that is aware of the compression algorithm used.
pub struct CompAwareStream {
  stream:    Box<dyn AsyncRead + Send + Unpin + 'static>,
  algorithm: Option<dvf::CompressionAlgorithm>,
}

impl CompAwareStream {
  /// Create a new `CompAwareStream`.
  pub fn new(
    stream: Box<dyn AsyncRead + Send + Unpin + 'static>,
    algorithm: Option<dvf::CompressionAlgorithm>,
  ) -> Self {
    Self { stream, algorithm }
  }

  /// Get the compression algorithm used.
  pub fn algorithm(&self) -> Option<dvf::CompressionAlgorithm> {
    self.algorithm
  }

  /// Get the inner stream and the compression algorithm used.
  pub fn into_inner(
    self,
  ) -> (
    Box<dyn AsyncRead + Send + Unpin + 'static>,
    Option<dvf::CompressionAlgorithm>,
  ) {
    (self.stream, self.algorithm)
  }

  /// Apply an adapter function to the inner stream.
  pub fn map_stream<F>(self, f: F) -> Self
  where
    F: FnOnce(
      Box<dyn AsyncRead + Send + Unpin + 'static>,
    ) -> Box<dyn AsyncRead + Send + Unpin + 'static>,
  {
    Self {
      stream:    f(self.stream),
      algorithm: self.algorithm,
    }
  }

  /// Apply a [`Read`] adapter function to the inner stream.
  pub fn map_stream_read_adapted<F>(self, f: F) -> Self
  where
    F: FnOnce(
      Box<dyn Read + Send + Unpin + 'static>,
    ) -> Box<dyn Read + Send + Unpin + 'static>,
  {
    self.map_stream(|s| Box::new(wrap_with_read_jump(s, f)))
  }
}

impl AsyncRead for CompAwareStream {
  fn poll_read(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &mut tokio::io::ReadBuf<'_>,
  ) -> Poll<tokio::io::Result<()>> {
    let this = Pin::into_inner(self);
    let pinned = std::pin::pin!(&mut this.stream);
    pinned.poll_read(cx, buf)
  }
}
