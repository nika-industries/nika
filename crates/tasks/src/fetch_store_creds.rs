use miette::IntoDiagnostic;
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

  type Response = core_types::StorageCredentials;
  type Error = CredsFetchingError;
  type State = db::DbConnection;

  async fn run(self, db: Self::State) -> Result<Self::Response, Self::Error> {
    let creds = match self.store_name.as_ref() {
      "nika-temp" => {
        tracing::info!("using hard-coded store \"nika-temp\"");
        core_types::StorageCredentials::R2(
          core_types::R2StorageCredentials::Default {
            access_key:        std::env::var("R2_TEMP_ACCESS_KEY")
              .into_diagnostic()
              .map_err(|e| CredsFetchingError::StoreInitError(e.to_string()))?,
            secret_access_key: std::env::var("R2_TEMP_SECRET_ACCESS_KEY")
              .into_diagnostic()
              .map_err(|e| CredsFetchingError::StoreInitError(e.to_string()))?,
            endpoint:          std::env::var("R2_TEMP_ENDPOINT")
              .into_diagnostic()
              .map_err(|e| CredsFetchingError::StoreInitError(e.to_string()))?,
            bucket:            std::env::var("R2_TEMP_BUCKET")
              .into_diagnostic()
              .map_err(|e| CredsFetchingError::StoreInitError(e.to_string()))?,
          },
        )
      }
      store_name => {
        db.fetch_store_by_name(store_name.as_ref())
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
