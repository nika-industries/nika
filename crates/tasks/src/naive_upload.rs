use std::path::PathBuf;

use prime_domain::{
  models::{self, StrictSlug},
  DynPrimeDomainService,
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
  type State = DynPrimeDomainService;

  #[tracing::instrument(name = "NaiveUpload", skip(self, state))]
  async fn run(
    self,
    state: Self::State,
  ) -> Result<Self::Response, Self::Error> {
    let prime_domain_service = state;

    tracing::info!("fetching cache");
    let cache = prime_domain_service
      .find_cache_by_name(self.cache_name.clone())
      .await
      .expect("failed to fetch cache")
      .expect("cache not found");

    tracing::info!("fetching store");
    let store = prime_domain_service
      .fetch_store_by_id(cache.store)
      .await
      .expect("failed to fetch store")
      .expect("store not found");

    tracing::info!("connecting to user storage");
    let target_client = prime_domain_service
      .connect_to_user_storage(store.config)
      .await
      .unwrap();

    let temp_reader = prime_domain_service
      .read_from_temp_storage(self.temp_storage_path.clone())
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

    prime_domain_service
      .create_entry(entry_cr)
      .await
      .expect("failed to create entry");

    Ok(())
  }
}
