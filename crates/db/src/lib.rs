//! Provides access to the database.

mod migrate;
mod store;
mod token;

use std::sync::Arc;

use kv::prelude::*;
use miette::{Context, IntoDiagnostic, Result};
use tracing::instrument;

/// The shared database type.
#[derive(Debug)]
pub struct DbConnection<T>(Arc<T>);

impl<T> Clone for DbConnection<T> {
  fn clone(&self) -> Self { Self(self.0.clone()) }
}

/// A TiKV-backed database client.
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

fn model_key<M: models::Model>(id: &M::Id) -> Key {
  let model_name_segment = Starc::new_owned(Slug::new(M::TABLE_NAME));
  let id_ulid: models::Ulid = id.clone().into();
  let id_segment = Starc::new_owned(Slug::new(id_ulid.to_string()));
  Key::new(model_name_segment).with(id_segment)
}

fn model_index_segment<M: models::Model>(index_name: &str) -> Starc<Slug> {
  Starc::new_owned(Slug::new(format!("{}_index_{}", M::TABLE_NAME, index_name)))
}

async fn rollback<T: KvTransaction>(mut txn: T) -> Result<()> {
  txn
    .rollback()
    .await
    .into_diagnostic()
    .context("failed to rollback transaction")
}

async fn commit<T: KvTransaction>(mut txn: T) -> Result<()> {
  txn
    .commit()
    .await
    .into_diagnostic()
    .context("failed to commit transaction")
}

impl<T: KvTransactional> DbConnection<T> {
  /// Creates a new model.
  #[instrument(skip(self))]
  pub async fn create_model<M: models::Model>(&self, model: &M) -> Result<()> {
    // the model itself will be stored under [model_name]:[id] -> model
    // and each index will be stored under
    // [model_name]_index_[index_name]:[index_value] -> [id]

    // calculate the key for the model
    let model_key = model_key::<M>(&model.id());
    let id_ulid: models::Ulid = model.id().clone().into();

    // calculate the keys for the indexes
    let index_keys = M::INDICES
      .iter()
      .map(|(index_name, index_fn)| {
        kv::key::Key::new(model_index_segment::<M>(index_name))
          .with(Starc::new_owned(index_fn(model)))
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
    if let Err(e) = txn
      .insert(&model_key, model_value)
      .await
      .into_diagnostic()
      .context("failed to insert model")
    {
      rollback(txn).await?;
      return Err(e);
    }

    for index_key in index_keys {
      if let Err(e) = txn
        .insert(&index_key, id_value.clone())
        .await
        .into_diagnostic()
        .context("failed to insert index")
      {
        rollback(txn).await?;
        return Err(e);
      }
    }

    commit(txn).await?;

    Ok(())
  }

  /// Fetches a model by its ID.
  #[instrument(skip(self))]
  pub async fn fetch_model_by_id<M: models::Model>(
    &self,
    id: &M::Id,
  ) -> Result<Option<M>> {
    let model_key = model_key::<M>(id);

    let mut txn = self
      .0
      .begin_optimistic_transaction()
      .await
      .into_diagnostic()
      .context("failed to begin optimistic transaction")?;

    let model_value = match txn.get(&model_key).await {
      Ok(value) => value,
      Err(e) => {
        rollback(txn).await?;
        return Err(e).into_diagnostic().context("failed to get model");
      }
    };

    match model_value {
      Some(value) => match kv::value::Value::deserialize(value) {
        Ok(value) => {
          commit(txn).await?;
          Ok(Some(value))
        }
        Err(e) => {
          rollback(txn).await?;
          Err(e)
            .into_diagnostic()
            .context("failed to deserialize model")
        }
      },
      None => {
        commit(txn).await?;
        Ok(None)
      }
    }
  }

  /// Fetches a model by an index.
  ///
  /// Must be a valid index, defined in the model's `INDICES` constant.
  #[instrument(skip(self))]
  pub async fn fetch_model_by_index<M: models::Model>(
    &self,
    index_name: &str,
    index_value: &Slug,
  ) -> Result<Option<M>> {
    let index_key = kv::key::Key::new(model_index_segment::<M>(index_name))
      .with(Starc::new_owned(index_value.clone()));

    let mut txn = self
      .0
      .begin_optimistic_transaction()
      .await
      .into_diagnostic()
      .context("failed to begin optimistic transaction")?;

    let id_value = match txn.get(&index_key).await {
      Ok(value) => value,
      Err(e) => {
        rollback(txn).await?;
        return Err(e).into_diagnostic().context("failed to get id");
      }
    };

    let id = match id_value {
      Some(value) => match kv::value::Value::deserialize(value) {
        Ok(value) => value,
        Err(e) => {
          rollback(txn).await?;
          return Err(e).into_diagnostic().context("failed to deserialize id");
        }
      },
      None => {
        commit(txn).await?;
        return Ok(None);
      }
    };

    let model = match self.fetch_model_by_id::<M>(&id).await {
      Ok(value) => value,
      Err(e) => {
        rollback(txn).await?;
        return Err(e).context("failed to fetch model by id");
      }
    };

    commit(txn).await?;

    Ok(model)
  }
}
