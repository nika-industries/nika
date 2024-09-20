//! Provides repository traits and implementors.

mod base;

use std::future::Future;

use miette::Result;
use slugger::EitherSlug;

/// Defines a repository interface for models.
pub trait ModelRepository: Clone + Send + Sync + 'static {
  /// The model type.
  type Model: models::Model;

  /// Creates a new model.
  fn create_model(
    &self,
    model: &Self::Model,
  ) -> impl Future<Output = Result<(), db::CreateModelError>> + Send;

  /// Fetches a model by its ID.
  fn fetch_model_by_id(
    &self,
    id: &models::RecordId<Self::Model>,
  ) -> impl Future<Output = Result<Option<Self::Model>>> + Send;

  /// Fetches a model by an index.
  ///
  /// Must be a valid index, defined in the model's `INDICES` constant.
  fn fetch_model_by_index(
    &self,
    index_name: &str,
    index_value: &EitherSlug,
  ) -> impl Future<Output = Result<Option<Self::Model>>> + Send;
}