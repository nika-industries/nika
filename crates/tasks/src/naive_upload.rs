use std::path::PathBuf;

use prime_domain::{
  models::{self, StrictSlug},
  DynCacheService, DynEntryService, DynStoreService, DynTempStorageService,
  DynUserStorageService, UserStorageClient,
};
use serde::{Deserialize, Serialize};

/// The health check task.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NaiveUploadTask {
  /// The target store name.
  pub cache_name:        StrictSlug,
  /// The target path.
  pub path:              PathBuf,
  /// The temporary storage path where the payload is currently stored.
  pub temp_storage_path: models::TempStoragePath,
}

#[async_trait::async_trait]
impl rope::Task for NaiveUploadTask {
  const NAME: &'static str = "NaiveUpload";

  type Response = ();
  type Error = ();
  type State = (
    DynCacheService,
    DynStoreService,
    DynEntryService,
    DynTempStorageService,
    DynUserStorageService,
  );

  #[tracing::instrument(name = "NaiveUpload", skip(self, state))]
  async fn run(
    self,
    state: Self::State,
  ) -> Result<Self::Response, Self::Error> {
    let (
      cache_service,
      store_service,
      entry_service,
      temp_storage_service,
      user_storage_service,
    ) = state;

    tracing::info!("fetching cache");
    let cache = cache_service
      .find_by_name(self.cache_name.clone())
      .await
      .expect("failed to fetch cache")
      .expect("cache not found");

    tracing::info!("fetching store");
    let store = store_service
      .fetch(cache.store)
      .await
      .expect("failed to fetch store")
      .expect("store not found");

    tracing::info!("connecting to user storage");
    let target_client =
      user_storage_service.connect(store.config).await.unwrap();

    let temp_reader = temp_storage_service
      .read(self.temp_storage_path.clone())
      .await
      .unwrap();
    let file_size = target_client.write(&self.path, temp_reader).await.unwrap();

    // create an Entry
    let entry_cr = models::EntryCreateRequest {
      path:  models::LaxSlug::new(self.path.to_string_lossy().to_string()),
      size:  file_size,
      cache: cache.id,
      org:   cache.org,
    };

    entry_service
      .create_model(entry_cr)
      .await
      .expect("failed to create entry");

    Ok(())
  }
}
