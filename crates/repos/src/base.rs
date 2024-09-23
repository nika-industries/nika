use std::{future::Future, marker::PhantomData};

pub use db::CreateModelError;
pub(crate) use db::{DatabaseAdapter, FetchModelByIndexError, FetchModelError};
use miette::Result;

use crate::ModelRepository;

/// Provides a base repository implementation for any model.
///
/// This is private and cannot be used directly. Each model's implementation
/// of `ModelRepository` needs to be a concrete type, even if it's just a
/// shell for this type, so that extra logic can be added later if needed.
pub(crate) struct BaseRepository<M: models::Model, DB: DatabaseAdapter> {
  db_adapter: DB,
  _phantom:   PhantomData<M>,
}

impl<M: models::Model, DB: DatabaseAdapter> Clone for BaseRepository<M, DB> {
  fn clone(&self) -> Self {
    Self {
      db_adapter: self.db_adapter.clone(),
      _phantom:   PhantomData,
    }
  }
}

impl<M: models::Model, DB: DatabaseAdapter> BaseRepository<M, DB> {
  pub fn new(db_adapter: DB) -> Self {
    Self {
      db_adapter,
      _phantom: PhantomData,
    }
  }
}

impl<M: models::Model, DB: DatabaseAdapter> ModelRepository
  for BaseRepository<M, DB>
{
  type Model = M;
  type ModelCreateRequest = M;
  type CreateError = CreateModelError;

  fn create_model(
    &self,
    input: Self::ModelCreateRequest,
  ) -> impl Future<Output = Result<(), CreateModelError>> + Send {
    self.db_adapter.create_model::<Self::Model>(input)
  }

  fn fetch_model_by_id(
    &self,
    id: models::RecordId<Self::Model>,
  ) -> impl Future<Output = Result<Option<Self::Model>, FetchModelError>> + Send
  {
    self.db_adapter.fetch_model_by_id(id)
  }

  fn fetch_model_by_index(
    &self,
    index_name: String,
    index_value: slugger::EitherSlug,
  ) -> impl Future<Output = Result<Option<Self::Model>, FetchModelByIndexError>> + Send
  {
    self
      .db_adapter
      .fetch_model_by_index(index_name, index_value)
  }
}
