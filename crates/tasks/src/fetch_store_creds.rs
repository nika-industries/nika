use mollusk::CredsFetchingError;
use serde::{Deserialize, Serialize};

/// The FetchStoreCreds task.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FetchStoreCredsTask {
  /// The name of the store to fetch credentials for.
  pub store_name: String,
}

#[async_trait::async_trait]
impl rope::Task for FetchStoreCredsTask {
  const NAME: &'static str = "FetchStoreCreds";

  type Response = models::StorageCredentials;
  type Error = CredsFetchingError;
  type State = db::TikvDb;

  async fn run(self, db: Self::State) -> Result<Self::Response, Self::Error> {
    let creds = match self.store_name.as_ref() {
      "nika-temp" => {
        tracing::debug!("using hard-coded store \"nika-temp\"");
        storage::temp::get_temp_storage_creds().map_err(|e| {
          CredsFetchingError::TempStorageCredsError(e.to_string())
        })?
      }
      store_name => {
        db.fetch_model_by_index::<models::Store>(
          "name",
          &slugger::StrictSlug::new(store_name).into(),
        )
        .await
        .map_err(|e| {
          CredsFetchingError::SurrealDbStoreRetrievalError(e.to_string())
        })?
        .ok_or_else(|| {
          CredsFetchingError::NoMatchingStore(store_name.to_string())
        })?
        .config
      }
    };

    Ok(creds)
  }
}
