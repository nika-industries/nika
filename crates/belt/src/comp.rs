use std::pin::Pin;

use async_compression::tokio::bufread::{ZstdDecoder, ZstdEncoder};
use bytes::Bytes;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncRead, BufReader};
use tokio_util::io::StreamReader;

use crate::Belt;

type ArBelt = StreamReader<Belt, Bytes>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CompressionAlgorithm {
  Zstd,
}

pub(crate) enum CompressionAdapter {
  FromZstd(BufReader<ZstdDecoder<ArBelt>>),
  ToZstd(BufReader<ZstdEncoder<ArBelt>>),
}

impl CompressionAdapter {
  pub(crate) fn from_zstd(belt: Belt) -> Self {
    Self::FromZstd(BufReader::new(ZstdDecoder::new(belt.to_async_buf_read())))
  }
  pub(crate) fn to_zstd(belt: Belt) -> Self {
    Self::ToZstd(BufReader::new(ZstdEncoder::new(belt.to_async_buf_read())))
  }
}

impl AsyncRead for CompressionAdapter {
  fn poll_read(
    self: Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
    buf: &mut tokio::io::ReadBuf,
  ) -> std::task::Poll<std::io::Result<()>> {
    match self.get_mut() {
      Self::FromZstd(inner) => Pin::new(inner).poll_read(cx, buf),
      Self::ToZstd(inner) => Pin::new(inner).poll_read(cx, buf),
    }
  }
}

impl AsyncBufRead for CompressionAdapter {
  fn poll_fill_buf(
    self: Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<std::io::Result<&[u8]>> {
    match self.get_mut() {
      Self::FromZstd(inner) => Pin::new(inner).poll_fill_buf(cx),
      Self::ToZstd(inner) => Pin::new(inner).poll_fill_buf(cx),
    }
  }

  fn consume(self: Pin<&mut Self>, amt: usize) {
    match self.get_mut() {
      Self::FromZstd(inner) => inner.consume(amt),
      Self::ToZstd(inner) => inner.consume(amt),
    }
  }
}

#[cfg(test)]
mod tests {
  use tokio::io::AsyncReadExt;

  use super::*;
  use crate::{comp::CompressionAlgorithm, Belt};

  #[tokio::test]
  async fn test_compression_adapter_roundtrip_zstd() {
    let stream = futures::stream::iter(vec![
      Ok(Bytes::from("hello")),
      Ok(Bytes::from(" world")),
    ]);
    let belt = Belt::from_stream(stream, None);

    let belt = belt.adapt_to_comp(CompressionAlgorithm::Zstd);
    let belt = belt.adapt_to_no_comp();

    let mut reader = BufReader::new(belt.to_async_buf_read());
    let mut buf = vec![0; 1024];

    let n = reader.read(&mut buf).await.unwrap();
    assert_eq!(&buf[..n], b"hello world");
  }
}
