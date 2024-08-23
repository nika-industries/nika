//! A generic interface for key-value stores.

pub mod key;
#[cfg(feature = "tikv")]
pub mod tikv;
pub mod value;

use std::future::Future;

use self::{key::Key, value::Value};

/// Represents errors that can occur when interacting with a key-value store.
#[derive(Debug, Clone, thiserror::Error, miette::Diagnostic)]
pub enum KvError {
  /// An error occurred in the underlying platform.
  #[error("platform error: {0}")]
  PlatformError(String),
}

#[cfg(feature = "tikv")]
impl From<tikv_client::Error> for KvError {
  fn from(error: tikv_client::Error) -> Self {
    KvError::PlatformError(error.to_string())
  }
}

/// Represents the result of a key-value operation.
pub type KvResult<T> = Result<T, KvError>;

/// Defines primitive methods for operating key-value stores.
pub trait KvPrimitive {
  /// Get the value of a key.
  fn get(
    &mut self,
    key: &Key,
  ) -> impl Future<Output = KvResult<Option<Value>>> + Send;
  /// Set the value of a key.
  fn put(
    &mut self,
    key: &Key,
    value: Value,
  ) -> impl Future<Output = KvResult<()>> + Send;
  /// Set the value of a key, only if it does not exist.
  fn insert(
    &mut self,
    key: &Key,
    value: Value,
  ) -> impl Future<Output = KvResult<()>> + Send;
}

/// Defines methods on transactions.
pub trait KvTransaction: KvPrimitive {
  /// Commit the transaction.
  fn commit(&mut self) -> impl Future<Output = KvResult<()>> + Send;
  /// Rollback the transaction.
  fn rollback(&mut self) -> impl Future<Output = KvResult<()>> + Send;
}

/// Defines methods and types for performing transactions on a key-value store.
pub trait KvTransactional {
  /// The type of optimistic transactions.
  type OptimisticTransaction: KvPrimitive + KvTransaction;
  /// The type of pessimistic transactions.
  type PessimisticTransaction: KvPrimitive + KvTransaction;

  /// Begin an optimistic transaction.
  fn begin_optimistic_transaction(
    &self,
  ) -> impl Future<Output = KvResult<Self::OptimisticTransaction>> + Send;
  /// Begin a pessimistic transaction.
  fn begin_pessimistic_transaction(
    &self,
  ) -> impl Future<Output = KvResult<Self::PessimisticTransaction>> + Send;
}
