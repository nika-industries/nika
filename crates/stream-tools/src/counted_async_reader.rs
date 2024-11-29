use std::{
  io::Error,
  pin::Pin,
  sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
  },
  task::{Context, Poll},
};

use tokio::io::{AsyncRead, ReadBuf};

/// A struct that maintains a count of bytes read asynchronously.
#[derive(Clone)]
pub struct Counter {
  count: Arc<AtomicU64>,
}

impl Counter {
  /// Returns the current size of the count.
  pub async fn current_size(&self) -> u64 { self.count.load(Ordering::Acquire) }
}

/// An async reader that counts the number of bytes read.
pub struct CountedAsyncReader<R> {
  reader:  R,
  counter: Counter,
}

impl<R> CountedAsyncReader<R> {
  /// Creates a new `CountedAsyncReader` and returns it along with a `Counter`.
  ///
  /// # Arguments
  /// * `reader` - The async reader that will be wrapped.
  ///
  /// # Returns
  /// A tuple containing the `CountedAsyncReader` and a `Counter`.
  pub fn new(reader: R) -> (Self, Counter) {
    let counter = Counter {
      count: Arc::new(AtomicU64::new(0)),
    };

    let reader = CountedAsyncReader {
      reader,
      counter: counter.clone(),
    };
    (reader, counter)
  }
}

impl<R: Unpin> Unpin for CountedAsyncReader<R> {}

impl<R> AsyncRead for CountedAsyncReader<R>
where
  R: AsyncRead + Unpin,
{
  fn poll_read(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &mut ReadBuf<'_>,
  ) -> Poll<Result<(), Error>> {
    let CountedAsyncReader { reader, counter } = self.get_mut();

    let poll_result = Pin::new(reader).poll_read(cx, buf);
    if let Poll::Ready(Ok(())) = &poll_result {
      counter
        .count
        .fetch_add(buf.filled().len() as u64, Ordering::Release);
    }

    poll_result
  }
}
