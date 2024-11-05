//! Various stream tools.

mod counted_async_reader;
pub mod read_jump;

use std::{
  io::Read,
  pin::Pin,
  task::{Context, Poll},
};

pub use tokio::io::AsyncRead;

use self::counted_async_reader::Counter;
pub use self::{
  counted_async_reader::CountedAsyncReader, read_jump::wrap_with_read_jump,
};

/// A stream that is aware of the compression algorithm used.
pub struct CompAwareAReader {
  stream:    Box<dyn AsyncRead + Send + Unpin + 'static>,
  algorithm: Option<dvf::CompressionAlgorithm>,
}

impl CompAwareAReader {
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

  /// Set the compression algorithm used.
  pub fn set_algorithm(
    self,
    algorithm: Option<dvf::CompressionAlgorithm>,
  ) -> Self {
    Self {
      stream: self.stream,
      algorithm,
    }
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

  /// Start counting the bytes read from the stream.
  pub fn counter(self) -> (Self, Counter) {
    let (stream, counter) = CountedAsyncReader::new(self.stream);
    (
      Self {
        stream:    Box::new(stream),
        algorithm: self.algorithm,
      },
      counter,
    )
  }

  /// Forgets the compression algorithm used.
  pub fn forget_algorithm(self) -> CompUnawareAReader {
    CompUnawareAReader::new(self.stream)
  }
}

impl AsyncRead for CompAwareAReader {
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

/// A stream that is unaware of the compression algorithm used.
pub struct CompUnawareAReader(Box<dyn AsyncRead + Send + Unpin + 'static>);

impl CompUnawareAReader {
  /// Create a new `CompUnawareAReader`.
  pub fn new(stream: Box<dyn AsyncRead + Send + Unpin + 'static>) -> Self {
    Self(stream)
  }

  /// Assigns a compression algorithm to the reader.
  pub fn assign_algorithm(
    self,
    algorithm: Option<dvf::CompressionAlgorithm>,
  ) -> CompAwareAReader {
    CompAwareAReader::new(self.0, algorithm)
  }
}

impl AsyncRead for CompUnawareAReader {
  fn poll_read(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &mut tokio::io::ReadBuf<'_>,
  ) -> Poll<tokio::io::Result<()>> {
    let this = Pin::into_inner(self);
    let pinned = std::pin::pin!(&mut this.0);
    pinned.poll_read(cx, buf)
  }
}
