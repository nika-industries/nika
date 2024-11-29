//! Provides `Belt`, a byte streaming container.

use std::{
  pin::Pin,
  task::{Context, Poll},
};

use bytes::Bytes;
use tokio::sync::mpsc;

/// A byte stream container.
#[derive(Debug)]
pub struct Belt {
  inner: mpsc::Receiver<Bytes>,
}

impl Belt {
  /// Create a new Belt from an existing `mpsc::Receiver<Bytes>`
  pub fn new(receiver: mpsc::Receiver<Bytes>) -> Self {
    Self { inner: receiver }
  }

  /// Create a channel pair with a default buffer size
  pub fn channel(buffer_size: usize) -> (mpsc::Sender<Bytes>, Self) {
    let (tx, rx) = mpsc::channel(buffer_size);
    (tx, Self::new(rx))
  }
}

impl futures::Stream for Belt {
  type Item = Bytes;

  fn poll_next(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    Pin::new(&mut self.inner).poll_recv(cx)
  }
}

impl tokio::io::AsyncRead for Belt {
  fn poll_read(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &mut tokio::io::ReadBuf,
  ) -> Poll<std::io::Result<()>> {
    match futures::ready!(Pin::new(&mut self.get_mut().inner).poll_recv(cx)) {
      Some(bytes) => {
        let len = std::cmp::min(buf.remaining(), bytes.len());
        buf.put_slice(&bytes[..len]);
        Poll::Ready(Ok(()))
      }
      None => Poll::Ready(Ok(())),
    }
  }
}

#[cfg(test)]
mod tests {
  use futures::StreamExt;

  use super::*;

  #[tokio::test]
  async fn test_belt() {
    let (tx, mut stream) = Belt::channel(10);

    tx.send(Bytes::from("hello")).await.unwrap();
    tx.send(Bytes::from(" world")).await.unwrap();

    drop(tx); // Close the channel

    assert_eq!(stream.next().await, Some(Bytes::from("hello")));
    assert_eq!(stream.next().await, Some(Bytes::from(" world")));
  }
}
