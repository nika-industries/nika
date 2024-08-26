use kv::prelude::*;
use miette::Result;

use super::DbConnection;

impl<T: KvTransactional> DbConnection<T> {
  /// Fetches the [`models::Token`] matching the given secret from the DB.
  pub async fn fetch_token_by_secret(
    &self,
    secret: &Slug,
  ) -> Result<Option<models::Token>> {
    self.fetch_model_by_index("secret", secret).await
  }
}
