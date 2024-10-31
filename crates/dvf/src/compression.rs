use crate::FileSize;

pub enum CompressionStatus {
  Compressed {
    compression_type: CompressionType,
    compressed_size:  FileSize,
    original_size:    FileSize,
  },
  Uncompressed {
    original_size: FileSize,
  },
}

pub enum CompressionType {
  Snappy,
}
