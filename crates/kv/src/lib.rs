//! A generic interface for key-value stores.
//!
//! Specifically, this crate is for **transactional** key-value stores. We also
//! assume that the key and value types are both byte arrays. The primary
//! interface is [`KvTransactional`], which provides methods for beginning
//! transactions.
//!
//! The transactions themselves implement [`KvPrimitive`] and [`KvTransaction`],
//! which provide basic operations and transaction-specific operations,
//! respectively.
//!
//! Other highlights include a zero-copy segment-based key encoding scheme, and
//! automatic messagepack ser/de for values.
//!
//! `tikv` is the only supported platform at the moment.
//!
//! This crate is yet to be hexagonalized.

pub mod key;
mod retryable;
#[cfg(feature = "tikv")]
pub mod tikv;
pub mod txn_ext;
pub mod value;

use std::{future::Future, ops::Bound};

use self::{key::Key, value::Value};

/// Re-exports commonly used types and traits.
pub mod prelude {
  pub use slugger::*;

  #[cfg(feature = "tikv")]
  pub use crate::tikv::TikvClient;
  pub use crate::{
    key::Key, txn_ext::KvTransactionExt, value::Value, KvError, KvPrimitive,
    KvResult, KvTransaction, KvTransactional,
  };
}

/// Represents errors that can occur when interacting with a key-value store.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum KvError {
  /// An error occurred in the underlying platform.
  #[error("platform error: {0}")]
  #[diagnostic(transparent)]
  PlatformError(miette::Report),
}

#[cfg(feature = "tikv")]
impl From<tikv_client::Error> for KvError {
  fn from(error: tikv_client::Error) -> Self {
    KvError::PlatformError(miette::Report::from_err(error))
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
  /// Scan the keyspace.
  fn scan(
    &mut self,
    start: Bound<Key>,
    end: Bound<Key>,
    limit: u32,
  ) -> impl Future<Output = KvResult<Vec<(Key, Value)>>> + Send;
}

/// Defines methods on transactions.
pub trait KvTransaction {
  /// Commit the transaction.
  fn commit(&mut self) -> impl Future<Output = KvResult<()>> + Send;
  /// Rollback the transaction.
  fn rollback(&mut self) -> impl Future<Output = KvResult<()>> + Send;
}

/// Defines methods and types for performing transactions on a key-value store.
pub trait KvTransactional: hex::Hexagonal {
  /// The type of optimistic transactions.
  type OptimisticTransaction: KvPrimitive
    + KvTransaction
    + Send
    + Sync
    + 'static;
  /// The type of pessimistic transactions.
  type PessimisticTransaction: KvPrimitive
    + KvTransaction
    + Send
    + Sync
    + 'static;

  /// Begin an optimistic transaction.
  fn begin_optimistic_transaction(
    &self,
  ) -> impl Future<Output = KvResult<Self::OptimisticTransaction>> + Send;
  /// Begin a pessimistic transaction.
  fn begin_pessimistic_transaction(
    &self,
  ) -> impl Future<Output = KvResult<Self::PessimisticTransaction>> + Send;
}
