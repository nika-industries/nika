use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use storage::StorageClientGenerator;

/// The health check task.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NaiveUploadTask {
  /// The target store name.
  pub store_name:        String,
  /// The target path.
  pub path:              PathBuf,
  /// The temporary storage path where the payload is currently stored.
  pub temp_storage_path: storage::temp::TempStoragePath,
}

#[async_trait::async_trait]
impl rope::Task for NaiveUploadTask {
  const NAME: &'static str = "NaiveUpload";

  type Response = ();
  type Error = ();
  type State = db::DbConnection;

  async fn run(
    self,
    _state: Self::State,
  ) -> Result<Self::Response, Self::Error> {
    let target_client = crate::FetchStoreCredsTask {
      store_name: self.store_name,
    }
    .run(_state.clone())
    .await
    .unwrap()
    .client()
    .await
    .unwrap();

    let temp_client = storage::temp::get_temp_storage_creds()
      .unwrap()
      .client()
      .await
      .unwrap();

    let temp_reader =
      temp_client.read(&self.temp_storage_path.0).await.unwrap();
    target_client.write(&self.path, temp_reader).await.unwrap();

    Ok(())
  }
}
