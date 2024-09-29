use std::sync::Arc;

use mollusk::*;
use prime_domain::{
  models::{self, LaxSlug, StrictSlug, TokenRecordId, TokenSecret},
  CacheService, EntryService, StoreService, TokenService,
};
use serde::{Deserialize, Serialize};

/// The FetchStoreCreds task.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrepareFetchPayloadTask {
  /// The name of the cache to fetch from.
  pub cache_name:   StrictSlug,
  /// The token being used to fetch the path.
  pub token_id:     Option<TokenRecordId>,
  /// The secret of the token being used to fetch the path.
  pub token_secret: Option<TokenSecret>,
  /// The path to fetch from the cache.
  pub path:         LaxSlug,
}

#[async_trait::async_trait]
impl rope::Task for PrepareFetchPayloadTask {
  const NAME: &'static str = "PrepareFetchPayload";

  type Response = models::StorageCredentials;
  type Error = PrepareFetchPayloadError;
  type State = (
    Arc<Box<dyn CacheService>>,
    Arc<Box<dyn StoreService>>,
    Arc<Box<dyn TokenService>>,
    Arc<Box<dyn EntryService>>,
  );

  async fn run(self, db: Self::State) -> Result<Self::Response, Self::Error> {
    let PrepareFetchPayloadTask {
      cache_name,
      token_id,
      token_secret,
      path,
    } = self;

    let (cache_service, store_service, token_service, entry_service) = db;

    let cache = cache_service
      .find_by_name(cache_name.clone())
      .await
      .map_err(|e| {
        PrepareFetchPayloadError::InternalError(InternalError(format!("{e:?}")))
      })?
      .ok_or(NoMatchingCacheError(cache_name.to_string()))?;

    let store = store_service
      .fetch(cache.store)
      .await
      .map_err(|e| InternalError(format!("{e:?}")))?
      .ok_or(InternalError(format!("store not found: {:?}", cache.store)))?;

    // run through authentication
    if !cache.public {
      // if the store is not public, we must have a token
      let token_id = token_id
        .ok_or(UnauthenticatedStoreAccessError(cache_name.to_string()))?;
      let token_secret = token_secret
        .ok_or(UnauthenticatedStoreAccessError(cache_name.to_string()))?;

      let required_permission = models::Permission::CachePermission {
        store_id:   store.id,
        permission: models::CachePermissionType::Read,
      };
      let required_permission_set =
        models::PermissionSet::from_iter(vec![required_permission.clone()]);

      let token = token_service
        .verify_token_id_and_secret(token_id, token_secret.clone())
        .await
        .map_err(|e| {
          PrepareFetchPayloadError::InternalError(InternalError(format!(
            "{e:?}"
          )))
        })?;
      let authorized = token.authorized(&required_permission_set);

      if !authorized {
        Err(UnauthorizedStoreAccessError {
          store_name: store.name.clone().into_inner(),
          permission: models::CachePermissionType::Read,
        })?;
      }
    }

    let _entry = entry_service
      .find_by_entry_id_and_path(cache.id, path.clone())
      .await
      .map_err(|e| InternalError(format!("{e:?}")))?;

    Ok(store.config)
  }
}
