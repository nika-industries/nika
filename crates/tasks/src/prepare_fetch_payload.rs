use mollusk::*;
use prime_domain::{
  models::{self, LaxSlug, StrictSlug, TokenRecordId, TokenSecret},
  DynPrimeDomainService,
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
  type State = DynPrimeDomainService;

  async fn run(
    self,
    state: Self::State,
  ) -> Result<Self::Response, Self::Error> {
    let PrepareFetchPayloadTask {
      cache_name,
      token_id,
      token_secret,
      path,
    } = self;

    let prime_domain_service = state;

    let cache = prime_domain_service
      .find_cache_by_name(cache_name.clone())
      .await
      .map_err(|e| {
        PrepareFetchPayloadError::InternalError(InternalError(format!("{e:?}")))
      })?
      .ok_or(NoMatchingCacheError(cache_name.to_string()))?;

    let store = prime_domain_service
      .fetch_store_by_id(cache.store)
      .await
      .map_err(|e| InternalError(format!("{e:?}")))?
      .ok_or(InternalError(format!("store not found: {:?}", cache.store)))?;

    // run through authentication
    if matches!(cache.visibility, models::Visibility::Private) {
      // if the store is not public, we must have a token
      let token_id = token_id
        .ok_or(UnauthenticatedStoreAccessError(cache_name.to_string()))?;
      let token_secret = token_secret
        .ok_or(UnauthenticatedStoreAccessError(cache_name.to_string()))?;

      let required_permission = models::Permission::CachePermission {
        cache_id:   cache.id,
        permission: models::CachePermissionType::Read,
      };
      let required_permission_set =
        models::PermissionSet::from_iter(vec![required_permission.clone()]);

      let token = prime_domain_service
        .verify_token_id_and_secret(token_id, token_secret.clone())
        .await
        .map_err(|e| {
          PrepareFetchPayloadError::InternalError(InternalError(format!(
            "{e:?}"
          )))
        })?;
      let authorized = token.authorized(&required_permission_set);

      if !authorized {
        Err(UnauthorizedCacheAccessError {
          cache_name: cache.name.clone().into_inner().into_inner(),
          permission: models::CachePermissionType::Read,
        })?;
      }
    }

    let _entry = prime_domain_service
      .find_entry_by_id_and_path(cache.id, path.clone())
      .await
      .map_err(|e| InternalError(format!("{e:?}")))?;

    Ok(store.config)
  }
}
