//! Provides repository traits and implementors.
//!
//! Repositories are used to abstract the storage layer from the rest of the
//! application. They are used by services to interact with table-like or object
//! storage.

mod base;
mod cache;
mod entry;
mod store;
mod temp_storage;
mod token;

pub use db;
use db::{FetchModelByIndexError, FetchModelError};
use hex::Hexagonal;
use miette::Result;
use models::EitherSlug;

pub use self::{cache::*, entry::*, store::*, temp_storage::*, token::*};

/// Defines a repository interface for models.
#[async_trait::async_trait]
pub trait ModelRepository: Hexagonal {
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

  /// Produces a list of all model IDs.
  async fn enumerate_models(&self) -> Result<Vec<Self::Model>>;
}

#[async_trait::async_trait]
impl<T, I> ModelRepository for T
where
  T: std::ops::Deref<Target = I> + Hexagonal + Sized,
  I: ModelRepository + ?Sized,
{
  type Model = I::Model;
  type ModelCreateRequest = I::ModelCreateRequest;
  type CreateError = I::CreateError;

  async fn create_model(
    &self,
    input: Self::ModelCreateRequest,
  ) -> Result<(), Self::CreateError> {
    I::create_model(self, input).await
  }
  async fn fetch_model_by_id(
    &self,
    id: models::RecordId<Self::Model>,
  ) -> Result<Option<Self::Model>, FetchModelError> {
    I::fetch_model_by_id(self, id).await
  }
  async fn fetch_model_by_index(
    &self,
    index_name: String,
    index_value: EitherSlug,
  ) -> Result<Option<Self::Model>, FetchModelByIndexError> {
    I::fetch_model_by_index(self, index_name, index_value).await
  }
  async fn enumerate_models(&self) -> Result<Vec<Self::Model>> {
    I::enumerate_models(self).await
  }
}

/// Defines a repository fetcher interface for models.
#[async_trait::async_trait]
pub trait ModelRepositoryFetcher: Hexagonal {
  /// The model type.
  type Model: models::Model;

  /// Fetches a model by its ID.
  async fn fetch(
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
  /// Produces a list of all model IDs.
  async fn enumerate_models(&self) -> Result<Vec<Self::Model>>;
}

// impl for smart pointer to dyn ModelRepositoryFetcher
#[async_trait::async_trait]
impl<T, I, M> ModelRepositoryFetcher for T
where
  T: std::ops::Deref<Target = I> + Hexagonal + Sized,
  I: ModelRepositoryFetcher<Model = M> + ?Sized,
  M: models::Model,
{
  type Model = M;

  async fn fetch(
    &self,
    id: models::RecordId<Self::Model>,
  ) -> Result<Option<Self::Model>, FetchModelError> {
    I::fetch(self, id).await
  }
  async fn fetch_model_by_index(
    &self,
    index_name: String,
    index_value: EitherSlug,
  ) -> Result<Option<Self::Model>, FetchModelByIndexError> {
    I::fetch_model_by_index(self, index_name, index_value).await
  }
  async fn enumerate_models(&self) -> Result<Vec<Self::Model>> {
    I::enumerate_models(self).await
  }
}

/// Defines a repository interface for creating models.
#[async_trait::async_trait]
pub trait ModelRepositoryCreator: Hexagonal {
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

// impl for smart pointer to dyn ModelRepositoryCreator
#[async_trait::async_trait]
impl<T, I, M, Mcr, Ce> ModelRepositoryCreator for T
where
  T: std::ops::Deref<Target = I> + Hexagonal + Sized,
  I: ModelRepositoryCreator<
      Model = M,
      ModelCreateRequest = Mcr,
      CreateError = Ce,
    > + ?Sized,
  M: models::Model,
  Mcr: std::fmt::Debug + Send + Sync + 'static,
  Ce: std::error::Error + Send + Sync + 'static,
{
  type Model = M;
  type ModelCreateRequest = Mcr;
  type CreateError = Ce;

  async fn create_model(
    &self,
    input: Self::ModelCreateRequest,
  ) -> Result<(), Self::CreateError> {
    I::create_model(self, input).await
  }
}
