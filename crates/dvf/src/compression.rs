use serde::{Deserialize, Serialize};

use crate::FileSize;

/// Represents the compression status of a file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CompressionStatus {
  /// The file is compressed.
  Compressed {
    /// The compressed size of the file.
    compressed_size:   FileSize,
    /// The uncompressed size of the file.
    uncompressed_size: FileSize,
    /// The compression algorithm used to compress the file.
    algorithm:         CompressionAlgorithm,
  },
  /// The file is not compressed.
  Uncompressed {
    /// The uncompressed size of the file.
    size: FileSize,
  },
}

impl CompressionStatus {
  /// Returns the compression algorithm used to compress the file.
  pub fn algorithm(&self) -> Option<CompressionAlgorithm> {
    match self {
      Self::Compressed { algorithm, .. } => Some(*algorithm),
      Self::Uncompressed { .. } => None,
    }
  }
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
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
  /// The Zstandard compression algorithm.
  Zstd,
}
