use dvf::FileSize;
use serde::{Deserialize, Serialize};

/// Represents the compression status of a file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CompressionStatus {
  /// The file is compressed.
  Compressed {
    compressed_size:   FileSize,
    uncompressed_size: FileSize,
    algorithm:         CompressionAlgorithm,
  },
  /// The file is not compressed.
  Uncompressed { size: FileSize },
}

/// Represents a configuration for compression.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompressionConfig {
  algorithm: Option<CompressionAlgorithm>,
}

impl CompressionConfig {
  /// Creates a new compression configuration.
  pub fn new(algorithm: Option<CompressionAlgorithm>) -> Self {
    Self { algorithm }
  }

  /// Returns the compression algorithm.
  pub fn algorithm(&self) -> Option<CompressionAlgorithm> { self.algorithm }
}

/// Represents a compression algorithm.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
  /// The snappy compression algorithm.
  Snappy,
}
