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

    prime_domain_service
      .create_entry(
        cache.id,
        self.path,
        prime_domain_service
          .read_from_temp_storage(self.temp_storage_path)
          .await
          .expect("failed to read from temp storage"),
      )
      .await
      .expect("failed to create entry");

    Ok(())
  }
}
