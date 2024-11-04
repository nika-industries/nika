//! # Crunch
//! **C**ompresses **R**eally **U**nbelievably **N**icely, **C**reates
//! **H**appiness.

mod algorithm_to_adapter;

use models::CompressionAlgorithm;
use stream_tools::{wrap_with_read_jump, AsyncRead, DynAsyncReader};

// use self::algorithm_to_adapter::AlgorithmToAdapter;

/// Adapts a reader to compress its output, while capturing the uncompressed and
/// compressed sizes.
pub fn adapt_compress(
  algorithm: CompressionAlgorithm,
  reader: DynAsyncReader,
) -> impl AsyncRead + Send + Unpin {
  match algorithm {
    CompressionAlgorithm::Snappy => wrap_with_read_jump(reader, |s| {
      Box::new(snap::read::FrameEncoder::new(s))
    }),
  }
}

/// Adapts a reader to decompress its output, while capturing the decompressed
/// size.
pub fn adapt_decompress(
  algorithm: CompressionAlgorithm,
  reader: DynAsyncReader,
) -> impl AsyncRead + Send + Unpin {
  match algorithm {
    CompressionAlgorithm::Snappy => wrap_with_read_jump(reader, |s| {
      Box::new(snap::read::FrameDecoder::new(s))
    }),
  }
}
