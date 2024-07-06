use core_types::STORE_TABLE_NAME;
use surrealdb::Result as SurrealResult;

use super::DbConnection;

impl DbConnection {
  /// Fetches the [`core_types::Store`] matching the given name from the DB.
  pub async fn fetch_store_by_name(
    &self,
    name: &str,
  ) -> SurrealResult<Option<core_types::Store>> {
    self
      .use_main()
      .await?
      .query(format!(
        "SELECT * FROM {STORE_TABLE_NAME} WHERE name = $name"
      ))
      .bind(("name", name))
      .await?
      .take(0)
  }
}
