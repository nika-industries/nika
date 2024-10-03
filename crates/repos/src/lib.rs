//! Provides repository traits and implementors.

mod base;
mod cache;
mod entry;
mod store;
mod temp_storage;
mod token;

pub use db::{FetchModelByIndexError, FetchModelError};
use miette::Result;
use slugger::EitherSlug;

pub use self::{cache::*, entry::*, store::*, temp_storage::*, token::*};

/// Defines a repository interface for models.
#[async_trait::async_trait]
pub trait ModelRepository: Send + Sync + 'static {
  /// The model type.
  type Model: models::Model;
  /// The request type for creating a model.
  type ModelCreateRequest: std::fmt::Debug + Send + Sync + 'static;
  /// The error type for creating a model.
  type CreateError: std::error::Error + Send + Sync + 'static;

  /// Creates a new model.
  async fn create_model(
    &self,
    input: Self::ModelCreateRequest,
  ) -> Result<(), Self::CreateError>;

  /// Fetches a model by its ID.
  async fn fetch_model_by_id(
    &self,
    id: models::RecordId<Self::Model>,
  ) -> Result<Option<Self::Model>, FetchModelError>;

  /// Fetches a model by an index.
  ///
  /// Must be a valid index, defined in the model's `INDICES` constant.
  async fn fetch_model_by_index(
    &self,
    index_name: String,
    index_value: EitherSlug,
  ) -> Result<Option<Self::Model>, FetchModelByIndexError>;
}

/// Defines a repository fetcher interface for models.
#[async_trait::async_trait]
pub trait ModelRepositoryFetcher: Send + Sync + 'static {
  /// The model type.
  type Model: models::Model;

  /// Fetches a model by its ID.
  async fn fetch(
    &self,
    id: models::RecordId<Self::Model>,
  ) -> Result<Option<Self::Model>, FetchModelError>;
}

/// Defines a repository interface for creating models.
#[async_trait::async_trait]
pub trait ModelRepositoryCreator: Send + Sync + 'static {
  /// The model type.
  type Model: models::Model;
  /// The request type for creating a model.
  type ModelCreateRequest: std::fmt::Debug + Send + Sync + 'static;
  /// The error type for creating a model.
  type CreateError: std::error::Error + Send + Sync + 'static;

  /// Creates a new model.
  async fn create_model(
    &self,
    input: Self::ModelCreateRequest,
  ) -> Result<(), Self::CreateError>;
}
