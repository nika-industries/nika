use rope::Task;
use serde::{Deserialize, Serialize};

/// The ConfirmTokenBySecretHasPermission task.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfirmTokenBySecretHasPermissionTask {
  token_secret: String,
  permission:   core_types::Permission,
}

impl ConfirmTokenBySecretHasPermissionTask {
  /// Creates a new ConfirmTokenBySecretHasPermission task. Does not run the
  /// task.
  pub fn new(token_secret: String, permission: core_types::Permission) -> Self {
    Self {
      token_secret,
      permission,
    }
  }
}

#[async_trait::async_trait]
impl Task for ConfirmTokenBySecretHasPermissionTask {
  const NAME: &'static str = "ConfirmTokenBySecretHasPermission";

  type Response = bool;
  type Error = mollusk::ConfirmTokenBySecretHasPermissionError;
  type State = db::DbConnection;

  async fn run(
    self,
    state: Self::State,
  ) -> Result<Self::Response, Self::Error> {
    let token_secret_slug = slugger::Slug::new(self.token_secret.clone());

    // make sure the secret is valid as a token secret
    if !core_types::validate_token_secret(&token_secret_slug)
      || !token_secret_slug.as_ref().eq(&self.token_secret)
    {
      Err(mollusk::MalformedTokenSecretError {
        token: self.token_secret.clone(),
      })?
    }

    // fetch the token
    let token = state
      .fetch_token_by_secret(token_secret_slug)
      .await
      .map_err(|e| {
        let error = format!("failed to fetch token by secret: {}", e);
        tracing::error!("{}", &error);
        mollusk::InternalError::SurrealDbQueryError(error)
      })?;

    // make sure the token exists
    let token = token.ok_or_else(|| mollusk::NonExistentTokenError {
      token: self.token_secret.clone(),
    })?;

    Ok(token.perms.0.contains(&self.permission))
  }
}
