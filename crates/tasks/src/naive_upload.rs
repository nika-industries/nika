use prime_domain::{
  models::{self, LaxSlug, StrictSlug},
  DynPrimeDomainService,
};
use serde::{Deserialize, Serialize};

/// The health check task.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NaiveUploadTask {
  /// The target store name.
  pub cache_name:        StrictSlug,
  /// The target path.
  pub path:              LaxSlug,
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

    let temp_reader = prime_domain_service
      .read_from_temp_storage(self.temp_storage_path.clone())
      .await
      .expect("failed to read from temp storage");
    let c_status = prime_domain_service
      .write_to_store(cache.store, self.path.clone(), temp_reader)
      .await
      .expect("failed to write to store");

    // create an Entry
    let entry_cr = models::EntryCreateRequest {
      path: models::LaxSlug::new(self.path.to_string().to_string()),
      c_status,
      cache: cache.id,
      org: cache.org,
    };

    prime_domain_service
      .create_entry(entry_cr)
      .await
      .expect("failed to create entry");

    Ok(())
  }
}
