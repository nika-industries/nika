use mollusk::*;
use serde::{Deserialize, Serialize};

/// The FetchStoreCreds task.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrepareFetchPayloadTask {
  /// The name of the store to fetch from.
  pub store_name:   String,
  /// The token being used to fetch the path.
  pub token_secret: Option<String>,
}

#[async_trait::async_trait]
impl rope::Task for PrepareFetchPayloadTask {
  const NAME: &'static str = "PrepareFetchPayload";

  type Response = core_types::StorageCredentials;
  type Error = PrepareFetchPayloadError;
  type State = db::DbConnection;

  async fn run(self, db: Self::State) -> Result<Self::Response, Self::Error> {
    let PrepareFetchPayloadTask {
      store_name,
      token_secret,
    } = self;

    let store = crate::FetchStoreByNameFromDbTask::new(store_name.clone())
      .run(db.clone())
      .await?
      .ok_or(NoMatchingStoreError(store_name.clone()))?;

    // return early if the store is public
    if store.public {
      return Ok(store.config);
    }

    // if the store is not public, we must have a token
    let token_secret = token_secret
      .ok_or(UnauthenticatedStoreAccessError(store_name.clone()))?;

    let required_permission = core_types::Permission::StorePermission {
      store_id:   store.id,
      permission: core_types::StorePermissionType::Read,
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
        store_name: store.nickname.clone().into_inner(),
        permission: core_types::StorePermissionType::Read,
      })?,
      Err(ConfirmTokenBySecretHasPermissionError::NonExistentToken(e)) => {
        Err(e)?
      }
      Err(ConfirmTokenBySecretHasPermissionError::MalformedTokenSecret(e)) => {
        Err(e)?
      }
      Err(ConfirmTokenBySecretHasPermissionError::InternalError(e)) => Err(e)?,
    };

    Ok(store.config)
  }
}
