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
  ///
  /// If we're properly creating IDs randomly, this should be very rare. It's
  /// more likely that the same instance of a model is being inserted multiple
  /// times.
  #[error("model with that ID already exists")]
  ModelAlreadyExists,
  /// An index with that value already exists.
  ///
  /// This is a constraint violation, and should be handled by the caller. It
  /// means that one of the indices, listed in the model's
  /// [`UNIQUE_INDICES`](models::Model::UNIQUE_INDICES) constant, already
  /// exists in the database.
  #[error("index {index_name:?} with value \"{index_value}\" already exists")]
  IndexAlreadyExists {
    /// The name of the index.
    index_name:  String,
    /// The value of the index.
    index_value: EitherSlug,
  },
  /// An error occurred while deserializing or serializing the model.
  ///
  /// This is a bug. Since we're serializing and deserializing to messagepack,
  /// it's most likely that this results from an improper deserialization
  /// caused by trying to deserialize to the wrong type.
  #[error("failed to deserialize or serialize model")]
  #[diagnostic_source]
  Serde(miette::Report),
  /// A retryable transaction error occurred.
  ///
  /// This is not a bug, but a transient error. It should be retried.
  #[error("retryable transaction error: {0}")]
  #[diagnostic_source]
  RetryableTransaction(miette::Report),
  /// A database error occurred.
  ///
  /// THis is an unknown error. Something we didn't expect to fail failed.
  #[error("db error: {0}")]
  #[diagnostic_source]
  Db(miette::Report),
}

/// Errors that can occur when fetching a model.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum FetchModelError {
  /// An error occurred while deserializing or serializing the model.
  ///
  /// This is a bug. Since we're serializing and deserializing to messagepack,
  /// it's most likely that this results from an improper deserialization
  /// caused by trying to deserialize to the wrong type.
  #[error("failed to deserialize or serialize model")]
  #[diagnostic_source]
  Serde(miette::Report),
  /// A retryable transaction error occurred.
  ///
  /// This is not a bug, but a transient error. It should be retried.
  #[error("retryable transaction error: {0}")]
  #[diagnostic_source]
  RetryableTransaction(miette::Report),
  /// A database error occurred.
  ///
  /// THis is an unknown error. Something we didn't expect to fail failed.
  #[error("db error: {0}")]
  #[diagnostic_source]
  Db(miette::Report),
}

/// Errors that can occur when fetching a model by index.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum FetchModelByIndexError {
  /// The index does not exist.
  ///
  /// This is a usage bug. We should only be fetching by indices that are
  /// defined in the model's [`UNIQUE_INDICES`](models::Model::UNIQUE_INDICES)
  /// constant.
  #[error("index {index_name:?} does not exist")]
  IndexDoesNotExistOnModel {
    /// The name of the index.
    index_name: String,
  },
  /// The index is malformed.
  ///
  /// This means that the index exists and points to an ID, but the model with
  /// that ID does not exist. This is a bug, because we should be cleaning up
  /// old indices when models are deleted.
  #[error("index {index_name:?} is malformed")]
  IndexMalformed {
    /// The name of the index.
    index_name:  String,
    /// The value of the index.
    index_value: EitherSlug,
  },
  /// An error occurred while fetching the indexed model.
  ///
  /// This means after we fetched the index, we tried to fetch the model by its
  /// ID and failed.
  #[error("failed to fetch indexed model")]
  #[diagnostic_source]
  FailedToFetchIndexedModel(#[from] FetchModelError),
  /// An error occurred while deserializing or serializing the model.
  ///
  /// This is a bug. Since we're serializing and deserializing to messagepack,
  /// it's most likely that this results from an improper deserialization
  /// caused by trying to deserialize to the wrong type.
  #[error("failed to deserialize or serialize model")]
  #[diagnostic_source]
  Serde(miette::Report),
  /// A retryable transaction error occurred.
  ///
  /// This is not a bug, but a transient error. It should be retried.
  #[error("retryable transaction error: {0}")]
  #[diagnostic_source]
  RetryableTransaction(miette::Report),
  /// A database error occurred.
  ///
  /// THis is an unknown error. Something we didn't expect to fail failed.
  #[error("db error: {0}")]
  #[diagnostic_source]
  Db(miette::Report),
}
