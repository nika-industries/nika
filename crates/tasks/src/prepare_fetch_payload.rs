use mollusk::{common::*, PrepareFetchPayloadError};
use serde::{Deserialize, Serialize};

/// The FetchStoreCreds task.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrepareFetchPayloadTask {
  /// The name of the store to fetch from.
  pub store_name: String,
  /// The path to fetch from the store.
  pub path:       String,
  /// The token being used to fetch the path.
  pub token:      String,
}

#[async_trait::async_trait]
impl rope::Task for PrepareFetchPayloadTask {
  const NAME: &'static str = "FetchStoreCreds";

  type Response = PrepareFetchPayload;
  type Error = PrepareFetchPayloadError;
  type State = db::DbConnection;

  async fn run(self, db: Self::State) -> Result<Self::Response, Self::Error> {
    todo!()
  }
}

pub struct PrepareFetchPayload {}
