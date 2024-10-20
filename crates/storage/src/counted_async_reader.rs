use std::{
  io::Error,
  pin::Pin,
  task::{Context, Poll},
};

use tokio::io::{AsyncRead, ReadBuf};

pub struct CountedAsyncReader<'a, R> {
  reader:  R,
  counter: &'a mut u64,
}

impl<'a, R> CountedAsyncReader<'a, R> {
  pub fn new(reader: R, counter: &'a mut u64) -> Self {
    Self { reader, counter }
  }
}

impl<'a, R> AsyncRead for CountedAsyncReader<'a, R>
where
  R: AsyncRead + Unpin,
{
  fn poll_read(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &mut ReadBuf<'_>,
  ) -> Poll<Result<(), Error>> {
    let this = self.get_mut();
    let poll = Pin::new(&mut this.reader).poll_read(cx, buf);

    if let Poll::Ready(Ok(())) = poll {
      *this.counter += buf.filled().len() as u64;
    }

    poll
  }
}
