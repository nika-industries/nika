use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use storage::StorageClientGenerator;

/// The health check task.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NaiveUploadTask {
  /// The target store name.
  pub cache_name:        String,
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
  type State = db::TikvDb;

  async fn run(self, db: Self::State) -> Result<Self::Response, Self::Error> {
    let cache = crate::FetchCacheByNameFromDbTask::new(self.cache_name)
      .run(db.clone())
      .await
      .expect("failed to fetch cache")
      .expect("cache not found");

    let store =
      crate::FetchModelByIdFromDbTask::<models::Store>::new(cache.store)
        .run(db.clone())
        .await
        .expect("failed to fetch store");

    let target_client = crate::FetchStoreCredsTask {
      store_name: store.name.to_string(),
    }
    .run(db.clone())
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

    // create an Entry
    let entry = models::Entry {
      id:    models::EntryRecordId(models::Ulid::new()),
      path:  models::LaxSlug::new(self.path.to_string_lossy().to_string()),
      size:  0,
      cache: cache.id,
      org:   cache.org,
    };

    db.create_model(&entry)
      .await
      .expect("failed to create entry");

    Ok(())
  }
}
