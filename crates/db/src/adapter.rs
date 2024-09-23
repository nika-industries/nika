use std::future::Future;

use kv::prelude::*;
use miette::Result;

/// An adapter for a model-based database.
pub trait DatabaseAdapter: Clone + Send + Sync + 'static {
  /// Creates a new model.
  fn create_model<M: models::Model>(
    &self,
    model: M,
  ) -> impl Future<Output = Result<(), CreateModelError>> + Send;
  /// Fetches a model by its ID.
  fn fetch_model_by_id<M: models::Model>(
    &self,
    id: models::RecordId<M>,
  ) -> impl Future<Output = Result<Option<M>, FetchModelError>> + Send;
  /// Fetches a model by an index.
  ///
  /// Must be a valid index, defined in the model's
  /// [`UNIQUE_INDICES`](models::Model::UNIQUE_INDICES) constant.
  fn fetch_model_by_index<M: models::Model>(
    &self,
    index_name: String,
    index_value: EitherSlug,
  ) -> impl Future<Output = Result<Option<M>, FetchModelByIndexError>> + Send;
}

/// Errors that can occur when creating a model.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum CreateModelError {
  /// A model with that ID already exists.
  #[error("model with that ID already exists")]
  ModelAlreadyExists,
  /// An index with that value already exists.
  #[error("index {index_name:?} with value \"{index_value}\" already exists")]
  IndexAlreadyExists {
    /// The name of the index.
    index_name:  String,
    /// The value of the index.
    index_value: EitherSlug,
  },
  /// An error occurred while deserializing or serializing the model
  #[error("failed to deserialize or serialize model")]
  #[diagnostic_source]
  Serde(miette::Report),
  /// A retryable transaction error occurred.
  #[error("retryable transaction error: {0}")]
  #[diagnostic_source]
  RetryableTransaction(miette::Report),
  /// A database error occurred.
  #[error("db error: {0}")]
  #[diagnostic_source]
  Db(miette::Report),
}

/// Errors that can occur when fetching a model.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum FetchModelError {
  /// An error occurred while deserializing or serializing the model
  #[error("failed to deserialize or serialize model")]
  #[diagnostic_source]
  Serde(miette::Report),
  /// A retryable transaction error occurred.
  #[error("retryable transaction error: {0}")]
  #[diagnostic_source]
  RetryableTransaction(miette::Report),
  /// A database error occurred.
  #[error("db error: {0}")]
  #[diagnostic_source]
  Db(miette::Report),
}

/// Errors that can occur when fetching a model by index.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum FetchModelByIndexError {
  /// The index does not exist.
  #[error("index {index_name:?} does not exist")]
  IndexDoesNotExist {
    /// The name of the index.
    index_name: String,
  },
  /// The index is malformed.
  #[error("index {index_name:?} is malformed")]
  IndexMalformed {
    /// The name of the index.
    index_name:  String,
    /// The value of the index.
    index_value: EitherSlug,
  },
  /// An error occurred while deserializing or serializing the model
  #[error("failed to deserialize or serialize model")]
  #[diagnostic_source]
  Serde(miette::Report),
  /// A retryable transaction error occurred.
  #[error("retryable transaction error: {0}")]
  #[diagnostic_source]
  RetryableTransaction(miette::Report),
  /// A database error occurred.
  #[error("db error: {0}")]
  #[diagnostic_source]
  Db(miette::Report),
}

impl From<FetchModelError> for FetchModelByIndexError {
  fn from(err: FetchModelError) -> Self {
    match err {
      FetchModelError::Db(err) => FetchModelByIndexError::Db(err),
      FetchModelError::RetryableTransaction(err) => {
        FetchModelByIndexError::RetryableTransaction(err)
      }
      FetchModelError::Serde(err) => FetchModelByIndexError::Serde(err),
    }
  }
}
