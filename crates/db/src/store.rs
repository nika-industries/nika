use kv::prelude::*;
use miette::Result;

use super::DbConnection;

impl<T: KvTransactional> DbConnection<T> {
  /// Fetches the [`models::Store`] matching the given name from the DB.
  pub async fn fetch_store_by_name(
    &self,
    name: &Slug,
  ) -> Result<Option<models::Store>> {
    self.fetch_model_by_index("name", name).await
  }
}
