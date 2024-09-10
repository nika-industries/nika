use mollusk::*;
use serde::{Deserialize, Serialize};

/// The FetchStoreCreds task.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrepareFetchPayloadTask {
  /// The name of the cache to fetch from.
  pub cache_name:   String,
  /// The token being used to fetch the path.
  pub token_secret: Option<String>,
  /// The path to fetch from the cache.
  pub path:         models::LaxSlug,
}

#[async_trait::async_trait]
impl rope::Task for PrepareFetchPayloadTask {
  const NAME: &'static str = "PrepareFetchPayload";

  type Response = models::StorageCredentials;
  type Error = PrepareFetchPayloadError;
  type State = db::TikvDb;

  async fn run(self, db: Self::State) -> Result<Self::Response, Self::Error> {
    let PrepareFetchPayloadTask {
      cache_name,
      token_secret,
      path,
    } = self;

    let cache = crate::FetchCacheByNameFromDbTask::new(cache_name.clone())
      .run(db.clone())
      .await?
      .ok_or(NoMatchingCacheError(cache_name.clone()))?;

    let store =
      crate::FetchModelByIdFromDbTask::<models::Store>::new(cache.store)
        .run(db.clone())
        .await?;

    // run through authentication
    if !cache.public {
      // if the store is not public, we must have a token
      let token_secret = token_secret
        .ok_or(UnauthenticatedStoreAccessError(cache_name.clone()))?;

      let required_permission = models::Permission::CachePermission {
        store_id:   store.id,
        permission: models::CachePermissionType::Read,
      };
      let authorized = crate::ConfirmTokenBySecretHasPermissionTask::new(
        token_secret,
        required_permission.clone(),
      )
      .run(db.clone())
      .await;
      match authorized {
        Ok(true) => (),
        Ok(false) => Err(UnauthorizedStoreAccessError {
          store_name: store.name.clone().into_inner(),
          permission: models::CachePermissionType::Read,
        })?,
        Err(ConfirmTokenBySecretHasPermissionError::NonExistentToken(e)) => {
          Err(e)?
        }
        Err(ConfirmTokenBySecretHasPermissionError::MalformedTokenSecret(
          e,
        )) => Err(e)?,
        Err(ConfirmTokenBySecretHasPermissionError::InternalError(e)) => {
          Err(e)?
        }
      };
    }

    let _entry =
      crate::FetchEntryByCacheIdAndPathFromDbTask::new(cache.id, path.clone())
        .run(db.clone())
        .await?
        .ok_or(MissingPathError {
          path: path.clone().to_string(),
        })?;

    Ok(store.config)
  }
}
