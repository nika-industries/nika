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
    key: &Key,
  ) -> impl Future<Output = KvResult<Option<String>>> + Send {
    self
      .client
      .get(key.to_string())
      .map_err(|e: RedisError| KvError::PlatformError(e.to_string()))
  }

  fn set(
    &self,
    key: &Key,
    value: String,
  ) -> impl Future<Output = KvResult<()>> + Send {
    self
      .client
      .set(key.to_string(), value, None, None, false)
      .map_err(|e: RedisError| KvError::PlatformError(e.to_string()))
  }
}

impl KvExtension for Redis {}
