use std::{
  io::Error,
  pin::Pin,
  sync::Arc,
  task::{Context, Poll},
};

use tokio::{
  io::{AsyncRead, ReadBuf},
  sync::mpsc,
};

/// A struct that maintains a count of bytes read asynchronously.
#[derive(Clone)]
pub struct Counter {
  count: Arc<tokio::sync::Mutex<u64>>,
}

impl Counter {
  fn new() -> Self {
    Self {
      count: Arc::new(tokio::sync::Mutex::new(0)),
    }
  }

  async fn increment(&self, bytes: u64) {
    let mut count = self.count.lock().await;
    *count += bytes;
  }

  /// Returns the current size of the count.
  pub async fn current_size(&self) -> u64 {
    let count = self.count.lock().await;
    *count
  }
}

/// An async reader that counts the number of bytes read.
pub struct CountedAsyncReader<R> {
  reader:     R,
  counter_tx: mpsc::Sender<u64>,
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
    let (counter_tx, mut counter_rx) = mpsc::channel(100);
    let counter = Counter::new();

    // Spawn a task to update the counter asynchronously
    let counter_clone = counter.clone();
    tokio::spawn(async move {
      while let Some(bytes) = counter_rx.recv().await {
        counter_clone.increment(bytes).await;
      }
    });

    (Self { reader, counter_tx }, counter)
  }
}

impl<R> AsyncRead for CountedAsyncReader<R>
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
      let bytes_read = buf.filled().len() as u64;
      // Send the number of bytes read to the counter
      let _ = this.counter_tx.try_send(bytes_read);
    }

    poll
  }
}
