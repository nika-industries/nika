use std::{
  io::{self, Read, Result},
  pin::Pin,
  sync::mpsc::{self, Receiver, Sender},
  task::{Context, Poll},
};

use tokio::{
  io::{AsyncRead, AsyncReadExt, ReadBuf},
  runtime::Handle,
};

/// Inner adapter that implements `Read` and receives data from an `AsyncRead`.
struct AsyncToSyncRead {
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
  inner: R,
}

impl<R: Read> AsyncReadJumper<R> {
  fn new(inner: R) -> Self { AsyncReadJumper { inner } }
}

impl<R: Read + Unpin> AsyncRead for AsyncReadJumper<R> {
  fn poll_read(
    mut self: Pin<&mut Self>,
    _cx: &mut Context<'_>,
    buf: &mut ReadBuf<'_>,
  ) -> Poll<io::Result<()>> {
    let mut temp_buf = vec![0; buf.remaining()];
    match self.inner.read(&mut temp_buf) {
      Ok(n) => {
        if n == 0 {
          Poll::Ready(Ok(())) // EOF
        } else {
          buf.put_slice(&temp_buf[..n]);
          Poll::Ready(Ok(()))
        }
      }
      Err(e) => Poll::Ready(Err(e)),
    }
  }
}

/// Function to create a channel-based adapter and apply an additional `Read`
/// adapter.
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
  let (sender, receiver): (Sender<Vec<u8>>, Receiver<Vec<u8>>) =
    mpsc::channel();
  let handle = Handle::current();

  // Use a blocking task to pull data from the `AsyncRead` and send it through
  // the channel
  tokio::task::spawn_blocking(move || {
    let mut buffer = vec![0; 1024];
    loop {
      let n = match handle.block_on(reader.read(&mut buffer)) {
        Ok(n) if n > 0 => n,
        _ => break, // EOF or error
      };

      if sender.send(buffer[..n].to_vec()).is_err() {
        break; // Channel closed
      }
    }
  });

  // Create the inner adapter
  let inner_read_adapter = AsyncToSyncRead::new(receiver);

  // Apply the additional adapter function to the `Read` implementer
  let adapted_inner = adapter_fn(Box::new(inner_read_adapter));

  // Wrap it in the outer adapter
  AsyncReadJumper::new(adapted_inner)
}
