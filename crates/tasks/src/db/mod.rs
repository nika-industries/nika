use rope::Task;
use serde::{Deserialize, Serialize};

/// The FetchModelByIdFromDb task.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FetchModelByIdFromDbTask<M: models::Model> {
  id: M::Id,
}

impl<M: models::Model> FetchModelByIdFromDbTask<M> {
  /// Create a new FetchModelByIdFromDb task.
  pub fn new(id: M::Id) -> Self { Self { id } }
}

#[async_trait::async_trait]
impl<M: models::Model> Task for FetchModelByIdFromDbTask<M> {
  const NAME: &'static str = "FetchModelByIdFromDb";

  type Response = M;
  type Error = mollusk::InternalError;
  type State = db::TikvDb;

  async fn run(
    self,
    state: Self::State,
  ) -> Result<Self::Response, Self::Error> {
    state
      .fetch_model_by_id::<M>(&self.id)
      .await
      .map_err(|e| {
        let error = format!("failed to fetch model by id: {}", e);
        tracing::error!("{}", &error);
        mollusk::InternalError::SurrealDbQueryError(error)
      })?
      .ok_or_else(|| mollusk::InternalError::MissingModelError {
        model_name: M::TABLE_NAME.to_string(),
        model_id:   self.id.to_string(),
      })
  }
}

/// The FetchModelByIndexFromDb task.
///
/// This isn't public because at the moment we're using explicit concrete
/// aliases (see below) to make sure that we don't use indices that don't exist,
/// since we're not enforcing with the type system at the moment.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct FetchModelByIndexFromDbTask<M: models::Model> {
  index:    String,
  value:    String,
  _phantom: std::marker::PhantomData<M>,
}

impl<M: models::Model> FetchModelByIndexFromDbTask<M> {
  pub fn new(index: String, value: String) -> Self {
    Self {
      index,
      value,
      _phantom: std::marker::PhantomData,
    }
  }
}

#[async_trait::async_trait]
impl<M: models::Model> Task for FetchModelByIndexFromDbTask<M> {
  const NAME: &'static str = "FetchModelByIndexFromDb";

  type Response = Option<M>;
  type Error = mollusk::InternalError;
  type State = db::TikvDb;

  async fn run(
    self,
    state: Self::State,
  ) -> Result<Self::Response, Self::Error> {
    state
      .fetch_model_by_index(
        &self.index,
        &models::LaxSlug::new(self.value).into(),
      )
      .await
      .map_err(|e| {
        let error = format!("failed to fetch model by index: {}", e);
        tracing::error!("{}", &error);
        mollusk::InternalError::SurrealDbQueryError(error)
      })
  }
}

/// The FetchStoreByNameFromDb task.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FetchCacheByNameFromDbTask {
  inner: FetchModelByIndexFromDbTask<models::Cache>,
}

impl FetchCacheByNameFromDbTask {
  /// Create a new FetchStoreByNameFromDb task.
  pub fn new(cache_name: String) -> Self {
    Self {
      inner: FetchModelByIndexFromDbTask::<models::Cache>::new(
        "name".to_string(),
        cache_name,
      ),
    }
  }
}

#[async_trait::async_trait]
impl Task for FetchCacheByNameFromDbTask {
  const NAME: &'static str = "FetchCacheByNameFromDb";

  type Response = Option<models::Cache>;
  type Error = mollusk::InternalError;
  type State = db::TikvDb;

  async fn run(
    self,
    state: Self::State,
  ) -> Result<Self::Response, Self::Error> {
    self.inner.run(state).await
  }
}

/// The FetchEntryByCacheIdAndPathFromDb task.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FetchEntryByCacheIdAndPathFromDbTask {
  inner: FetchModelByIndexFromDbTask<models::Entry>,
}

impl FetchEntryByCacheIdAndPathFromDbTask {
  /// Create a new FetchEntryByCacheIdAndPathFromDb task.
  pub fn new(cache_id: models::CacheRecordId, path: models::LaxSlug) -> Self {
    Self {
      inner: FetchModelByIndexFromDbTask::<models::Entry>::new(
        "cache-id-path".to_string(),
        format!("{}-{}", cache_id, path),
      ),
    }
  }
}

#[async_trait::async_trait]
impl Task for FetchEntryByCacheIdAndPathFromDbTask {
  const NAME: &'static str = "FetchEntryByCacheIdAndPathFromDb";

  type Response = Option<models::Entry>;
  type Error = mollusk::InternalError;
  type State = db::TikvDb;

  async fn run(
    self,
    state: Self::State,
  ) -> Result<Self::Response, Self::Error> {
    self.inner.run(state).await
  }
}

// /// The FetchStoreByNameFromDb task.
// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub struct FetchStoreByNameFromDbTask {
//   inner: FetchModelByIndexFromDbTask<models::Store>,
// }

// impl FetchStoreByNameFromDbTask {
//   pub fn new(store_name: String) -> Self {
//     Self {
//       inner: FetchModelByIndexFromDbTask::<models::Store>::new(
//         "name".to_string(),
//         store_name,
//       ),
//     }
//   }
// }

// #[async_trait::async_trait]
// impl Task for FetchStoreByNameFromDbTask {
//   const NAME: &'static str = "FetchStoreByNameFromDb";

//   type Response = Option<models::Store>;
//   type Error = mollusk::InternalError;
//   type State = db::TikvDb;

//   async fn run(
//     self,
//     state: Self::State,
//   ) -> Result<Self::Response, Self::Error> {
//     self.inner.run(state).await
//   }
// }
