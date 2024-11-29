use std::{
  fmt, io,
  num::NonZeroUsize,
  pin::Pin,
  task::{Context, Poll},
};

use bytes::Bytes;
use futures::{Stream, StreamExt};
use pin_project::pin_project;

/// A stream wrapper that limits the size of chunks passing through.
#[pin_project]
pub struct Limiter<S> {
  chunk_size: NonZeroUsize,
  #[pin]
  stream:     S,
  buffer:     Option<Bytes>,
}

impl<S> Limiter<S>
where
  S: Stream<Item = io::Result<Bytes>>,
{
  /// Create a new `Limiter` wrapping the given stream with a maximum chunk
  /// size.
  pub fn new(chunk_size: NonZeroUsize, stream: S) -> Self {
    Self {
      chunk_size,
      stream,
      buffer: None,
    }
  }
}

impl<S> fmt::Debug for Limiter<S> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Limiter")
      .field("chunk_size", &self.chunk_size)
      .field("stream", &"...")
      .field("buffer", &self.buffer)
      .finish()
  }
}

impl<S> Stream for Limiter<S>
where
  S: Stream<Item = io::Result<Bytes>>,
{
  type Item = io::Result<Bytes>;

  fn poll_next(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    let mut this = self.project();

    // If there's data in the buffer, yield a chunk from it first.
    if let Some(buffer) = this.buffer.as_mut() {
      if buffer.len() > this.chunk_size.get() {
        let chunk = buffer.split_to(this.chunk_size.get());
        return Poll::Ready(Some(Ok(chunk)));
      } else {
        let remaining = this.buffer.take();
        return Poll::Ready(remaining.map(Ok));
      }
    }

    // Otherwise, poll the wrapped stream for the next chunk.
    match this.stream.as_mut().poll_next(cx) {
      Poll::Ready(Some(Ok(mut bytes))) => {
        if bytes.len() > this.chunk_size.get() {
          let chunk = bytes.split_to(this.chunk_size.get());
          *this.buffer = Some(bytes);
          Poll::Ready(Some(Ok(chunk)))
        } else {
          Poll::Ready(Some(Ok(bytes)))
        }
      }
      Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
      Poll::Ready(None) => Poll::Ready(None),
      Poll::Pending => Poll::Pending,
    }
  }
}

#[cfg(test)]
mod tests {
  use std::{io, num::NonZeroUsize};

  use bytes::Bytes;
  use futures::{stream, StreamExt};

  use super::*;

  #[tokio::test]
  async fn test_limiter_basic() {
    // Input stream with arbitrary byte chunks
    let input_chunks = vec![
      Ok(Bytes::from_static(b"hello")),
      Ok(Bytes::from_static(b"world")),
      Ok(Bytes::from_static(b"!")),
    ];
    let stream = stream::iter(input_chunks);

    // Create a Limiter with a chunk size of 5
    let chunk_size = NonZeroUsize::new(5).unwrap();
    let mut limiter = Limiter::new(chunk_size, stream);

    // Validate output
    assert_eq!(limiter.next().await.unwrap().unwrap(), Bytes::from("hello"));
    assert_eq!(limiter.next().await.unwrap().unwrap(), Bytes::from("world"));
    assert_eq!(limiter.next().await.unwrap().unwrap(), Bytes::from("!"));
    assert!(limiter.next().await.is_none());
  }

  #[tokio::test]
  async fn test_limiter_split_large_chunks() {
    // Input stream with large chunks
    let input_chunks =
      vec![Ok(Bytes::from_static(b"abcdefghijklmnopqrstuvwxyz"))];
    let stream = stream::iter(input_chunks);

    // Create a Limiter with a chunk size of 5
    let chunk_size = NonZeroUsize::new(5).unwrap();
    let mut limiter = Limiter::new(chunk_size, stream);

    // Validate that large chunks are split correctly
    let expected_chunks = vec![
      Bytes::from_static(b"abcde"),
      Bytes::from_static(b"fghij"),
      Bytes::from_static(b"klmno"),
      Bytes::from_static(b"pqrst"),
      Bytes::from_static(b"uvwxy"),
      Bytes::from_static(b"z"),
    ];

    for expected in expected_chunks {
      assert_eq!(limiter.next().await.unwrap().unwrap(), expected);
    }

    assert!(limiter.next().await.is_none());
  }

  #[tokio::test]
  async fn test_limiter_empty_input() {
    // Input stream with no chunks
    let input_chunks: Vec<io::Result<Bytes>> = vec![];
    let stream = stream::iter(input_chunks);

    // Create a Limiter with any chunk size
    let chunk_size = NonZeroUsize::new(5).unwrap();
    let mut limiter = Limiter::new(chunk_size, stream);

    // Validate that no chunks are emitted
    assert!(limiter.next().await.is_none());
  }

  #[tokio::test]
  async fn test_limiter_error_propagation() {
    // Input stream with an error
    let input_chunks = vec![
      Ok(Bytes::from_static(b"hello")),
      Err(io::Error::new(io::ErrorKind::Other, "test error")),
      Ok(Bytes::from_static(b"world")),
    ];
    let stream = stream::iter(input_chunks);

    // Create a Limiter with a chunk size of 5
    let chunk_size = NonZeroUsize::new(5).unwrap();
    let mut limiter = Limiter::new(chunk_size, stream);

    // Validate output up to the error
    assert_eq!(limiter.next().await.unwrap().unwrap(), Bytes::from("hello"));
    match limiter.next().await {
      Some(Err(e)) => assert_eq!(e.kind(), io::ErrorKind::Other),
      _ => panic!("Expected error"),
    }
    assert_eq!(limiter.next().await.unwrap().unwrap(), Bytes::from("world"));
    assert!(limiter.next().await.is_none());
  }

  #[tokio::test]
  async fn test_limiter_chunk_boundary() {
    // Input stream with chunks matching the limit boundary
    let input_chunks = vec![
      Ok(Bytes::from_static(b"12345")),
      Ok(Bytes::from_static(b"67890")),
    ];
    let stream = stream::iter(input_chunks);

    // Create a Limiter with a chunk size of 5
    let chunk_size = NonZeroUsize::new(5).unwrap();
    let mut limiter = Limiter::new(chunk_size, stream);

    // Validate that chunks are passed through without modification
    assert_eq!(limiter.next().await.unwrap().unwrap(), Bytes::from("12345"));
    assert_eq!(limiter.next().await.unwrap().unwrap(), Bytes::from("67890"));
    assert!(limiter.next().await.is_none());
  }
}
