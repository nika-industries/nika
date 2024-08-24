//! Provides access to the SurrealDB database.

mod store;
mod token;

use std::sync::Arc;

use kv::prelude::*;
use miette::{Context, IntoDiagnostic, Result};
use tracing::instrument;

/// The shared database type.
#[derive(Clone, Debug)]
pub struct DbConnection<T>(Arc<T>);

pub type TikvDb = DbConnection<TikvClient>;

impl TikvDb {
  /// Constructs a new [`DbConnection`]`<`[`TikvClient`]`>`.
  #[instrument]
  pub async fn new() -> Result<Self> {
    let urls = std::env::var("TIKV_URLS")
      .into_diagnostic()
      .wrap_err("missing TIKV_URLS")?;
    let urls = urls.split(',').collect();
    let client = TikvClient::new(urls).await.into_diagnostic()?;

    Ok(DbConnection(Arc::new(client)))
  }
}

fn model_index_segment<M: models::Model>(index_name: &str) -> Starc<Slug> {
  Starc::new_owned(Slug::new(format!("{}_index_{}", M::TABLE_NAME, index_name)))
}

impl<T: KvTransactional> DbConnection<T> {
  #[instrument(skip(self))]
  pub(crate) async fn create_model<M: models::Model>(
    &self,
    model: M,
  ) -> Result<()> {
    // the model itself will be stored under [model_name]:[id] -> model
    // and each index will be stored under
    // [model_name]_index_[index_name]:[index_value] -> [id]

    // calculate the key for the model
    let model_name_segment = Starc::new_owned(Slug::new(M::TABLE_NAME));
    let id_ulid: models::Ulid = model.id().into();
    let id_segment = Starc::new_owned(Slug::new(id_ulid));
    let model_key = kv::key::Key::new(model_name_segment).with(id_segment);

    // calculate the keys for the indexes
    let index_keys = M::INDICES
      .iter()
      .map(|(index_name, index_fn)| {
        kv::key::Key::new(model_index_segment::<M>(index_name))
          .with(Starc::new_owned(index_fn(&model)))
      })
      .collect::<Vec<_>>();

    // serialize the model into bytes
    let model_value = kv::value::Value::serialize(&model)
      .into_diagnostic()
      .context("failed to serialize model")?;

    // serialize the id into bytes
    let id_value = kv::value::Value::serialize(&id_ulid)
      .into_diagnostic()
      .context("failed to serialize id")?;

    // begin a transaction
    let mut txn = self
      .0
      .begin_pessimistic_transaction()
      .await
      .into_diagnostic()
      .context("failed to begin pessimistic transaction")?;

    // insert the model and indexes
    txn
      .insert(&model_key, model_value)
      .await
      .into_diagnostic()
      .context("failed to insert model")?;

    for index_key in index_keys {
      txn
        .insert(&index_key, id_value.clone())
        .await
        .into_diagnostic()
        .context("failed to insert index")?;
    }

    // commit the transaction
    txn
      .commit()
      .await
      .into_diagnostic()
      .context("failed to commit transaction")?;

    Ok(())
  }
}
