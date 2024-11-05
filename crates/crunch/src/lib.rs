//! # Crunch
//! **C**ompresses **R**eally **U**nbelievably **N**icely, **C**reates
//! **H**appiness.

use dvf::CompressionAlgorithm;
use stream_tools::CompAwareAReader;

/// Trait for adapting compression algorithms.
pub trait AdaptCompression {
  /// Adapts the compression algorithm of the item to another algorithm.
  fn adapt_compression_to(
    self,
    algorithm: Option<CompressionAlgorithm>,
  ) -> Self;
}

impl AdaptCompression for CompAwareAReader {
  fn adapt_compression_to(
    self,
    desired_algo: Option<CompressionAlgorithm>,
  ) -> Self {
    let current_algo = self.algorithm();

    match (current_algo, desired_algo) {
      (
        Some(CompressionAlgorithm::Snappy),
        Some(CompressionAlgorithm::Snappy),
      ) => self,
      (Some(CompressionAlgorithm::Snappy), None) => self
        .map_stream_read_adapted(|s| {
          Box::new(snap::read::FrameEncoder::new(s))
        }),
      (None, Some(CompressionAlgorithm::Snappy)) => self
        .map_stream_read_adapted(|s| {
          Box::new(snap::read::FrameDecoder::new(s))
        }),
      (None, None) => self,
    }
  }
}
