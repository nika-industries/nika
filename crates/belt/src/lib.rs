//! Provides `Belt`, a byte streaming container.

mod counter;
mod limiter;
mod source;

use std::{
  io::Result,
  num::NonZeroUsize,
  pin::Pin,
  sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
  },
  task::{Context, Poll},
};

use bytes::Bytes;
use futures::{Stream, StreamExt};
use tokio::{io::AsyncBufRead, sync::mpsc};

pub use self::counter::Counter;
use self::{limiter::Limiter, source::BytesSource};

#[derive(Debug)]
enum MaybeLimitedSource {
  Unlimited(BytesSource),
  Limited(Limiter<BytesSource>),
}

impl Stream for MaybeLimitedSource {
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

/// A byte stream container.
#[derive(Debug)]
pub struct Belt {
  inner: MaybeLimitedSource,
  count: Arc<AtomicU64>,
}

impl Belt {
  /// Create a new Belt from an existing `mpsc::Receiver<Bytes>`
  pub fn from_channel(
    receiver: mpsc::Receiver<Result<Bytes>>,
    limit: Option<NonZeroUsize>,
  ) -> Self {
    Self {
      inner: match limit {
        Some(limit) => MaybeLimitedSource::Limited(Limiter::new(
          limit,
          BytesSource::Channel(receiver),
        )),
        None => MaybeLimitedSource::Unlimited(BytesSource::Channel(receiver)),
      },
      count: Arc::new(AtomicU64::new(0)),
    }
  }

  /// Create a new Belt from an existing `impl Stream<Item = Bytes>`
  pub fn from_stream(
    stream: impl Stream<Item = Result<Bytes>> + Send + Unpin + 'static,
    limit: Option<NonZeroUsize>,
  ) -> Self {
    Self {
      inner: match limit {
        Some(limit) => MaybeLimitedSource::Limited(Limiter::new(
          limit,
          BytesSource::Erased(Box::new(stream)),
        )),
        None => {
          MaybeLimitedSource::Unlimited(BytesSource::Erased(Box::new(stream)))
        }
      },
      count: Arc::new(AtomicU64::new(0)),
    }
  }

  /// Create a new Belt from an existing `impl AsyncBufRead`
  pub fn from_async_read(
    reader: impl AsyncBufRead + Send + Unpin + 'static,
    limit: Option<NonZeroUsize>,
  ) -> Self {
    Self {
      inner: match limit {
        Some(limit) => MaybeLimitedSource::Limited(Limiter::new(
          limit,
          BytesSource::AsyncBufRead(tokio_util::io::ReaderStream::new(
            Box::new(reader),
          )),
        )),
        None => MaybeLimitedSource::Unlimited(BytesSource::AsyncBufRead(
          tokio_util::io::ReaderStream::new(Box::new(reader)),
        )),
      },
      count: Arc::new(AtomicU64::new(0)),
    }
  }

  /// Create a channel pair with a default buffer size
  pub fn new_channel(
    buffer_size: usize,
    limit: Option<NonZeroUsize>,
  ) -> (mpsc::Sender<Result<Bytes>>, Self) {
    let (tx, rx) = mpsc::channel(buffer_size);
    (tx, Self::from_channel(rx, limit))
  }

  /// Get a tracking counter for the total number of bytes read from this
  /// [`Belt`].
  pub fn counter(&self) -> Counter { Counter::new(self.count.clone()) }

  /// Convert this Belt into an [`AsyncBufRead`](tokio::io::AsyncBufRead)
  /// implementer.
  pub fn to_async_buf_read(self) -> tokio_util::io::StreamReader<Self, Bytes> {
    tokio_util::io::StreamReader::new(self)
  }
}

impl Stream for Belt {
  type Item = Result<Bytes>;

  fn poll_next(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    let poll_result = self.inner.poll_next_unpin(cx);

    if let Poll::Ready(Some(Ok(bytes))) = &poll_result {
      self.count.fetch_add(bytes.len() as u64, Ordering::Release);
    }

    poll_result
  }
}

#[cfg(test)]
mod tests {
  use futures::StreamExt;
  use tokio::io::AsyncReadExt;

  use super::*;

  #[tokio::test]
  async fn test_belt_from_channel() {
    let (tx, mut belt) = Belt::new_channel(10, None);
    let counter = belt.counter();

    tx.send(Ok(Bytes::from("hello"))).await.unwrap();
    tx.send(Ok(Bytes::from(" world"))).await.unwrap();

    drop(tx); // Close the channel

    assert_eq!(
      belt.next().await.transpose().unwrap(),
      Some(Bytes::from("hello"))
    );
    assert_eq!(
      belt.next().await.transpose().unwrap(),
      Some(Bytes::from(" world"))
    );
    assert_eq!(counter.current(), 11);
  }

  #[tokio::test]
  async fn test_belt_from_stream() {
    let stream = futures::stream::iter(vec![
      Ok(Bytes::from("hello")),
      Ok(Bytes::from(" world")),
    ]);
    let belt = Belt::from_stream(stream, None);
    let counter = belt.counter();

    let mut belt = belt;
    assert_eq!(
      belt.next().await.transpose().unwrap(),
      Some(Bytes::from("hello"))
    );
    assert_eq!(
      belt.next().await.transpose().unwrap(),
      Some(Bytes::from(" world"))
    );
    assert_eq!(counter.current(), 11);
  }

  #[tokio::test]
  async fn test_belt_from_async_read() {
    let reader = std::io::Cursor::new(b"hello world");
    let mut belt = Belt::from_async_read(reader, Some(5.try_into().unwrap()));
    let counter = belt.counter();

    assert_eq!(
      belt.next().await.transpose().unwrap(),
      Some(Bytes::from("hello"))
    );
    assert_eq!(
      belt.next().await.transpose().unwrap(),
      Some(Bytes::from(" worl"))
    );
    assert_eq!(
      belt.next().await.transpose().unwrap(),
      Some(Bytes::from("d"))
    );
    assert_eq!(counter.current(), 11);
  }

  #[tokio::test]
  async fn test_belt_to_async_read() {
    let stream = futures::stream::iter(vec![
      Ok(Bytes::from("hello")),
      Ok(Bytes::from(" world")),
    ]);
    let belt = Belt::from_stream(stream, None);
    let counter = belt.counter();

    let mut reader = belt.to_async_buf_read();
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).await.unwrap();

    assert_eq!(buf, b"hello world");
    assert_eq!(counter.current(), 11);
  }

  #[tokio::test]
  async fn test_belt_to_async_read_error() {
    let (tx, belt) = Belt::new_channel(10, None);
    let counter = belt.counter();

    tx.send(Ok(Bytes::from("hello"))).await.unwrap();
    tx.send(Err(std::io::Error::new(std::io::ErrorKind::Other, "oh no")))
      .await
      .unwrap();

    drop(tx); // Close the channel

    let mut reader = belt.to_async_buf_read();
    let mut buf = Vec::new();
    let err = reader.read_to_end(&mut buf).await.unwrap_err();

    assert_eq!(buf, b"hello");
    assert_eq!(err.to_string(), "oh no");
    assert_eq!(counter.current(), 5);
  }

  #[tokio::test]
  async fn test_belt_to_async_read_channel_partial() {
    let (tx, belt) = Belt::new_channel(10, None);
    let counter = belt.counter();

    tx.send(Ok(Bytes::from("hello world"))).await.unwrap();

    drop(tx); // Close the channel

    let mut reader = belt.to_async_buf_read();
    let mut buf = [0; 5];
    reader.read_exact(&mut buf).await.unwrap();

    assert_eq!(&buf, b"hello");
    // the whole bytes object was consumed
    assert_eq!(counter.current(), 11);

    let (mut belt, buf) = reader.into_inner_with_chunk();
    // there are no chunks left
    assert_eq!(belt.next().await.transpose().unwrap(), None);
    // the bytes that weren't read from the StreamReader
    assert_eq!(buf, Some(Bytes::from_static(b" world")));
  }
}
