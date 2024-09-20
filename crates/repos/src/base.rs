use std::{future::Future, marker::PhantomData};

use miette::Result;

use crate::ModelRepository;

/// Provides a base repository implementation for any model.
///
/// This is private and cannot be used directly. Each model's implementation
/// of `ModelRepository` needs to be a concrete type, even if it's just a
/// shell for this type, so that extra logic can be added later if needed.
#[derive(Clone)]
pub(crate) struct BaseRepository<M: models::Model, DB: db::DatabaseAdapter> {
  db_adapter: DB,
  _phantom:   PhantomData<M>,
}

impl<M: models::Model, DB: db::DatabaseAdapter> BaseRepository<M, DB> {
  pub fn new(db_adapter: DB) -> Self {
    Self {
      db_adapter,
      _phantom: PhantomData,
    }
  }
}

impl<M: models::Model, DB: db::DatabaseAdapter> ModelRepository
  for BaseRepository<M, DB>
{
  type Model = M;

  fn create_model(
    &self,
    model: &Self::Model,
  ) -> impl Future<Output = Result<(), db::CreateModelError>> + Send {
    self.db_adapter.create_model(model)
  }

  fn fetch_model_by_id(
    &self,
    id: &models::RecordId<Self::Model>,
  ) -> impl Future<Output = Result<Option<Self::Model>>> + Send {
    self.db_adapter.fetch_model_by_id(id)
  }

  fn fetch_model_by_index(
    &self,
    index_name: &str,
    index_value: &slugger::EitherSlug,
  ) -> impl Future<Output = Result<Option<Self::Model>>> + Send {
    self
      .db_adapter
      .fetch_model_by_index(index_name, index_value)
  }
}
