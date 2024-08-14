use rope::Task;
use serde::{Deserialize, Serialize};

/// The FetchStoreByNameFromDb task.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FetchStoreByNameFromDbTask {
  store_name: String,
}

impl FetchStoreByNameFromDbTask {
  /// Creates a new FetchStoreByNameFromDb task. Does not run the task.
  pub fn new(store_name: String) -> Self { Self { store_name } }
}

#[async_trait::async_trait]
impl Task for FetchStoreByNameFromDbTask {
  const NAME: &'static str = "FetchStoreByNameFromDb";

  type Response = Option<core_types::Store>;
  type Error = mollusk::InternalError;
  type State = db::DbConnection;

  async fn run(
    self,
    state: Self::State,
  ) -> Result<Self::Response, Self::Error> {
    state
      .fetch_store_by_name(&self.store_name)
      .await
      .map_err(|e| {
        let error = format!("failed to fetch store by name: {}", e);
        tracing::error!("{}", &error);
        mollusk::InternalError::SurrealDbQueryError(error)
      })
  }
}
