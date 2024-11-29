//! Provides `Belt`, a byte streaming container.

mod limiter;

use std::{
  fmt,
  io::Result,
  num::NonZeroUsize,
  pin::Pin,
  task::{Context, Poll},
};

use bytes::Bytes;
use futures::{Stream, StreamExt};
use tokio::sync::mpsc;

use self::limiter::Limiter;

#[derive(Debug)]
enum MaybeLimitedBeltSource {
  Unlimited(BeltSource),
  Limited(Limiter<BeltSource>),
}

impl Stream for MaybeLimitedBeltSource {
  type Item = Result<Bytes>;

  fn poll_next(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    match &mut *self {
      Self::Unlimited(source) => source.poll_next_unpin(cx),
      Self::Limited(limited) => limited.poll_next_unpin(cx),
    }
  }
}

enum BeltSource {
  Channel(mpsc::Receiver<Result<Bytes>>),
  Erased(Box<dyn Stream<Item = Result<Bytes>> + Send + Sync + Unpin>),
}

impl futures::Stream for BeltSource {
  type Item = Result<Bytes>;

  fn poll_next(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    match &mut *self {
      Self::Channel(rx) => rx.poll_recv(cx),
      Self::Erased(stream) => Pin::new(stream).poll_next(cx),
    }
  }
}

impl fmt::Debug for BeltSource {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Channel(c) => f.debug_tuple("Channel").field(c).finish(),
      Self::Erased(_) => f.debug_tuple("Erased").finish(),
    }
  }
}

/// A byte stream container.
#[derive(Debug)]
pub struct Belt {
  inner: MaybeLimitedBeltSource,
}

impl Belt {
  /// Create a new Belt from an existing `mpsc::Receiver<Bytes>`
  pub fn from_channel(
    receiver: mpsc::Receiver<Result<Bytes>>,
    limit: Option<NonZeroUsize>,
  ) -> Self {
    Self {
      inner: match limit {
        Some(limit) => MaybeLimitedBeltSource::Limited(Limiter::new(
          limit,
          BeltSource::Channel(receiver),
        )),
        None => {
          MaybeLimitedBeltSource::Unlimited(BeltSource::Channel(receiver))
        }
      },
    }
  }

  /// Create a new Belt from an existing `Stream<Item = Bytes>`
  pub fn from_stream<S>(stream: S, limit: Option<NonZeroUsize>) -> Self
  where
    S: Stream<Item = Result<Bytes>> + Send + Sync + Unpin + 'static,
  {
    Self {
      inner: match limit {
        Some(limit) => MaybeLimitedBeltSource::Limited(Limiter::new(
          limit,
          BeltSource::Erased(Box::new(stream)),
        )),
        None => MaybeLimitedBeltSource::Unlimited(BeltSource::Erased(
          Box::new(stream),
        )),
      },
    }
  }

  /// Create a channel pair with a default buffer size
  pub fn channel(
    buffer_size: usize,
    limit: Option<NonZeroUsize>,
  ) -> (mpsc::Sender<Result<Bytes>>, Self) {
    let (tx, rx) = mpsc::channel(buffer_size);
    (tx, Self::from_channel(rx, limit))
  }

  /// Convert this Belt into an [`AsyncRead`] stream
  pub fn to_async_read(self) -> tokio_util::io::StreamReader<Self, Bytes> {
    tokio_util::io::StreamReader::new(self)
  }
}

impl Stream for Belt {
  type Item = Result<Bytes>;

  fn poll_next(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    self.inner.poll_next_unpin(cx)
  }
}

#[cfg(test)]
mod tests {
  use futures::StreamExt;
  use tokio::io::AsyncReadExt;

  use super::*;

  #[tokio::test]
  async fn test_belt_channel() {
    let (tx, mut stream) = Belt::channel(10, None);

    tx.send(Ok(Bytes::from("hello"))).await.unwrap();
    tx.send(Ok(Bytes::from(" world"))).await.unwrap();

    drop(tx); // Close the channel

    assert_eq!(
      stream.next().await.transpose().unwrap(),
      Some(Bytes::from("hello"))
    );
    assert_eq!(
      stream.next().await.transpose().unwrap(),
      Some(Bytes::from(" world"))
    );
  }

  #[tokio::test]
  async fn test_belt_erased() {
    let stream = futures::stream::iter(vec![
      Ok(Bytes::from("hello")),
      Ok(Bytes::from(" world")),
    ]);
    let belt = Belt::from_stream(stream, None);

    let mut belt = belt;
    assert_eq!(
      belt.next().await.transpose().unwrap(),
      Some(Bytes::from("hello"))
    );
    assert_eq!(
      belt.next().await.transpose().unwrap(),
      Some(Bytes::from(" world"))
    );
  }

  #[tokio::test]
  async fn test_belt_to_async_read() {
    let stream = futures::stream::iter(vec![
      Ok(Bytes::from("hello")),
      Ok(Bytes::from(" world")),
    ]);
    let belt = Belt::from_stream(stream, None);

    let mut reader = belt.to_async_read();
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).await.unwrap();

    assert_eq!(buf, b"hello world");
  }

  #[tokio::test]
  async fn test_belt_to_async_read_channel() {
    let (tx, stream) = Belt::channel(10, None);

    tx.send(Ok(Bytes::from("hello"))).await.unwrap();
    tx.send(Ok(Bytes::from(" world"))).await.unwrap();

    drop(tx); // Close the channel

    let mut reader = stream.to_async_read();
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).await.unwrap();

    assert_eq!(buf, b"hello world");
  }

  #[tokio::test]
  async fn test_belt_to_async_read_channel_error() {
    let (tx, stream) = Belt::channel(10, None);

    tx.send(Ok(Bytes::from("hello"))).await.unwrap();
    tx.send(Err(std::io::Error::new(std::io::ErrorKind::Other, "oh no")))
      .await
      .unwrap();

    drop(tx); // Close the channel

    let mut reader = stream.to_async_read();
    let mut buf = Vec::new();
    let err = reader.read_to_end(&mut buf).await.unwrap_err();

    assert_eq!(err.to_string(), "oh no");
    assert_eq!(buf, b"hello");
  }

  #[tokio::test]
  async fn test_belt_to_async_read_channel_partial() {
    let (tx, stream) = Belt::channel(10, None);

    tx.send(Ok(Bytes::from("hello"))).await.unwrap();
    tx.send(Ok(Bytes::from(" world"))).await.unwrap();

    drop(tx); // Close the channel

    let mut reader = stream.to_async_read();
    let mut buf = [0; 5];
    reader.read_exact(&mut buf).await.unwrap();

    assert_eq!(&buf, b"hello");
  }
}
