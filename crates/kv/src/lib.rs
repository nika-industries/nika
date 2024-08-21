//! A generic interface for key-value stores.

pub mod key;

use std::future::Future;

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
  /// The error type for platform errors.
  type PlatformError;

  /// Get the value associated with a key.
  fn get(
    &self,
    key: &str,
  ) -> impl Future<Output = KvResult<Option<String>>> + Send;
  /// Set the value associated with a key.
  fn set(
    &self,
    key: &str,
    value: String,
  ) -> impl Future<Output = KvResult<()>> + Send;
}

/// Defines composite and first-party methods for key-value stores.
pub trait KvExtension: KvPrimitive {}

#[cfg(feature = "redis")]
pub mod redis {
  //! Contains a Redis implementation of the key-value interface.

  use fred::prelude::*;
  use futures::TryFutureExt;

  use super::*;

  /// Redis implementation of the key-value interface
  pub struct Redis {
    client:           RedisClient,
    _connection_task: fred::types::ConnectHandle,
  }

  impl Redis {
    /// Create a new Redis store.
    pub async fn new(
      config: RedisConfig,
      perf: Option<PerformanceConfig>,
      connection: Option<ConnectionConfig>,
      policy: Option<ReconnectPolicy>,
    ) -> Result<Self, RedisError> {
      let client = RedisClient::new(config, perf, connection, policy);
      let _connection_task = client.init().await?;
      Ok(Self {
        client,
        _connection_task,
      })
    }
  }

  impl KvPrimitive for Redis {
    type PlatformError = RedisError;

    fn get(
      &self,
      key: &str,
    ) -> impl Future<Output = KvResult<Option<String>>> + Send {
      self
        .client
        .get(key)
        .map_err(|e: RedisError| KvError::PlatformError(e.to_string()))
    }

    fn set(
      &self,
      key: &str,
      value: String,
    ) -> impl Future<Output = KvResult<()>> + Send {
      self
        .client
        .set(key, value, None, None, false)
        .map_err(|e: RedisError| KvError::PlatformError(e.to_string()))
    }
  }

  impl KvExtension for Redis {}
}
