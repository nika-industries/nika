//! Defines an `AsyncReadJumper`, to apply sync `Read` adapters to `AsyncRead`

use std::{
  io::{self, Read, Result},
  pin::Pin,
  sync::mpsc::{sync_channel, Receiver},
  task::{Context, Poll},
};

use tokio::io::{AsyncRead, AsyncReadExt, ReadBuf};

/// Inner adapter that implements `Read` and receives data from an `AsyncRead`.
pub struct AsyncToSyncRead {
  receiver: Receiver<Vec<u8>>,
  buffer:   Vec<u8>,
}

impl AsyncToSyncRead {
  fn new(receiver: Receiver<Vec<u8>>) -> Self {
    AsyncToSyncRead {
      receiver,
      buffer: Vec::new(),
    }
  }
}

impl Read for AsyncToSyncRead {
  fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
    // Refill buffer if empty
    if self.buffer.is_empty() {
      if let Ok(data) = self.receiver.recv() {
        self.buffer = data;
      } else {
        return Ok(0); // Channel closed
      }
    }

    // Copy data to the output buffer
    let len = buf.len().min(self.buffer.len());
    buf[..len].copy_from_slice(&self.buffer[..len]);
    self.buffer.drain(..len);
    Ok(len)
  }
}

/// Outer adapter that wraps a `Read` implementer and provides an `AsyncRead`
/// interface.
pub struct AsyncReadJumper<R: Read> {
  inner:        R,
  pending_data: Vec<u8>,
}

impl<R: Read> AsyncReadJumper<R> {
  fn new(inner: R) -> Self {
    AsyncReadJumper {
      inner,
      pending_data: Vec::new(),
    }
  }
}

impl<R: Read + Unpin> AsyncRead for AsyncReadJumper<R> {
  fn poll_read(
    mut self: Pin<&mut Self>,
    _cx: &mut Context<'_>,
    buf: &mut ReadBuf<'_>,
  ) -> Poll<io::Result<()>> {
    // First, try to use any pending data
    if !self.pending_data.is_empty() {
      let len = buf.remaining().min(self.pending_data.len());
      buf.put_slice(&self.pending_data[..len]);
      self.pending_data.drain(..len);
      return Poll::Ready(Ok(()));
    }

    // Read new data from the inner Read implementer
    let mut temp_buf = vec![0; buf.remaining()];
    match self.inner.read(&mut temp_buf) {
      Ok(0) => Poll::Ready(Ok(())), // EOF
      Ok(n) => {
        buf.put_slice(&temp_buf[..n]);
        Poll::Ready(Ok(()))
      }
      Err(e) => Poll::Ready(Err(e)),
    }
  }
}

const CHANNEL_BUFFER_SIZE: usize = 4; // Number of chunks to buffer
const CHUNK_SIZE: usize = 16 * 1024; // 16KB chunks

/// Function to create a channel-based adapter and apply an additional `Read`
/// adapter.
///
/// This is the primary way we use [`Read`] adapters on [`AsyncRead`] streams.
/// This is useful for compression adapters, for example.
///
/// It works by creating a blocking task that reads data from the [`AsyncRead`]
/// stream and sends it to a channel. The data is consumed by
/// [`AsyncToSyncRead`] which uses it to provide a sync [`Read`] impl, which can
/// be passed by value into the adapter function. The result is then wrapped in
/// an [`AsyncReadJumper`] and returned.
///
/// There's two inefficiencies here:
/// 1. There's nothing special about [`AsyncReadJumper`]; it just wraps a
///    [`Read`] implementer, so it polls blindly.
/// 2. The data is first read before it's needed. The blocking task loads the
///    data into a buffer, but this happens before its requested by the outer
///    [`AsyncRead`] implementer.
///
/// It would be better to have [`AsyncReadJumper`] be the one to read from the
/// source and feed it into the channel.
pub fn wrap_with_read_jump<R, F>(
  mut reader: R,
  adapter_fn: F,
) -> AsyncReadJumper<impl Read + Unpin + 'static>
where
  R: AsyncRead + Send + Unpin + 'static,
  F: FnOnce(
    Box<dyn Read + Send + Unpin + 'static>,
  ) -> Box<dyn Read + Send + Unpin + 'static>,
{
  // Use sync_channel with a bounded buffer for backpressure
  let (sender, receiver) = sync_channel(CHANNEL_BUFFER_SIZE);

  // Spawn a blocking task to read from AsyncRead and feed the channel
  tokio::task::spawn_blocking(move || {
    let rt = tokio::runtime::Handle::current();
    let mut buffer = vec![0; CHUNK_SIZE];

    loop {
      match rt.block_on(reader.read(&mut buffer)) {
        Ok(0) => break, // EOF
        Ok(n) => {
          // send() will block if the channel is full, providing backpressure
          if sender.send(buffer[..n].to_vec()).is_err() {
            break; // Channel closed
          }
        }
        Err(_) => break, // Error
      }
    }
  });

  // Create the sync Read adapter chain
  let sync_reader = AsyncToSyncRead::new(receiver);
  let adapted_reader = adapter_fn(Box::new(sync_reader));

  // Wrap it back in an AsyncRead interface
  AsyncReadJumper::new(adapted_reader)
}
