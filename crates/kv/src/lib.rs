//! A generic interface for key-value stores.

pub mod key;
#[cfg(feature = "redis")]
pub mod redis;

use std::future::Future;

use self::key::Key;

/// Represents errors that can occur when interacting with a key-value store.
#[derive(Debug, Clone, thiserror::Error, miette::Diagnostic)]
pub enum KvError {
  /// An error occurred in the underlying platform.
  #[error("platform error: {0}")]
  PlatformError(String),
}

type KvResult<T> = Result<T, KvError>;

/// Defines methods common to all key-value stores.
pub trait KvPrimitive {
  /// Get the value associated with a key.
  fn get(
    &self,
    key: &Key,
  ) -> impl Future<Output = KvResult<Option<String>>> + Send;
  /// Set the value associated with a key.
  fn set(
    &self,
    key: &Key,
    value: String,
  ) -> impl Future<Output = KvResult<()>> + Send;
}

/// Defines composite and first-party methods for key-value stores.
pub trait KvExtension: KvPrimitive {}
