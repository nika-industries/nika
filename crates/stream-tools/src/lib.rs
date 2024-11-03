//! Various stream tools.

use tokio::io::AsyncRead;

pub use self::counted_async_reader::CountedAsyncReader;

/// Trait alias for `Box<dyn AsyncReader + ...>`
pub type DynAsyncReader = Box<dyn AsyncRead + Send + Unpin + 'static>;

mod counted_async_reader {
  use std::{
    io::Error,
    pin::Pin,
    task::{Context, Poll},
  };

  use tokio::io::{AsyncRead, ReadBuf};

  /// An async reader that counts the number of bytes read.
  pub struct CountedAsyncReader<'a, R> {
    reader:  R,
    counter: &'a mut u64,
  }

  impl<'a, R> CountedAsyncReader<'a, R> {
    /// Create a new `CountedAsyncReader`.
    pub fn new(reader: R, counter: &'a mut u64) -> Self {
      Self { reader, counter }
    }
  }

  impl<R> AsyncRead for CountedAsyncReader<'_, R>
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
}
