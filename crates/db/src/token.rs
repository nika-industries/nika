use core_types::TOKEN_TABLE_NAME;
use surrealdb::Result as SurrealResult;

use super::DbConnection;

impl DbConnection {
  /// Fetches the [`core_types::Token`] matching the given secret from the DB.
  ///
  /// The `token` table has a unique index on the `secret` field, which is why
  /// the return type is an `Option<>` instead of a `Vec<>`.
  pub async fn fetch_token_by_secret(
    &self,
    secret: core_types::Slug,
  ) -> SurrealResult<Option<core_types::Token>> {
    self
      .use_main()
      .await?
      .query(format!(
        "SELECT * FROM {TOKEN_TABLE_NAME} WHERE secret = $secret"
      ))
      .bind(("secret", secret.as_ref()))
      .await?
      .take(0)
  }
}
