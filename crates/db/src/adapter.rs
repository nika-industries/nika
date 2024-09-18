use std::future::Future;

use kv::prelude::*;
use miette::Result;

/// An adapter for a model-based database.
pub trait DatabaseAdapter: Send + Sync + 'static {
  /// Creates a new model.
  fn create_model<M: models::Model>(
    &self,
    model: &M,
  ) -> impl Future<Output = Result<(), CreateModelError>> + Send;
  /// Fetches a model by its ID.
  fn fetch_model_by_id<M: models::Model>(
    &self,
    id: &models::RecordId<M>,
  ) -> impl Future<Output = Result<Option<M>>> + Send;
  /// Fetches a model by an index.
  ///
  /// Must be a valid index, defined in the model's `INDICES` constant.
  fn fetch_model_by_index<M: models::Model>(
    &self,
    index_name: &str,
    index_value: &EitherSlug,
  ) -> impl Future<Output = Result<Option<M>>> + Send;
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
  /// A database error occurred.
  #[error("db error: {0}")]
  #[diagnostic_source]
  DbError(miette::Report),
}

impl From<miette::Report> for CreateModelError {
  fn from(e: miette::Report) -> Self { Self::DbError(e) }
}
