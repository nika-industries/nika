use std::{
  fmt,
  io::Result,
  pin::Pin,
  task::{Context, Poll},
};

use bytes::Bytes;
use futures::Stream;
use tokio::{io::AsyncBufRead, sync::mpsc};

pub(crate) enum BytesSource {
  Channel(mpsc::Receiver<Result<Bytes>>),
  Erased(Box<dyn Stream<Item = Result<Bytes>> + Send + Unpin>),
  AsyncBufRead(
    tokio_util::io::ReaderStream<Box<dyn AsyncBufRead + Send + Unpin>>,
  ),
}

impl futures::Stream for BytesSource {
  type Item = Result<Bytes>;

  fn poll_next(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    match &mut *self {
      Self::Channel(rx) => rx.poll_recv(cx),
      Self::Erased(stream) => Pin::new(stream).poll_next(cx),
      Self::AsyncBufRead(reader) => Pin::new(reader).poll_next(cx),
    }
  }
}

impl fmt::Debug for BytesSource {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Channel(c) => f.debug_tuple("Channel").field(c).finish(),
      Self::Erased(_) => f.debug_tuple("Erased").finish(),
      Self::AsyncBufRead(_) => f.debug_tuple("AsyncBufRead").finish(),
    }
  }
}
