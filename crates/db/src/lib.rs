//! Provides access to the database.

mod consumptive;
mod migrate;
mod store;
mod token;

use std::sync::Arc;

use kv::prelude::*;
use miette::{Context, IntoDiagnostic, Result};
use tracing::instrument;

use self::consumptive::ConsumptiveTransaction;

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
    let client = TikvClient::new(urls)
      .await
      .into_diagnostic()
      .context("failed to create tikv client")?;

    Ok(DbConnection(Arc::new(client)))
  }
}

fn model_key<M: models::Model>(id: &M::Id) -> Key {
  let model_name_segment = StrictSlug::new(M::TABLE_NAME);
  let id_ulid: models::Ulid = id.clone().into();
  let id_segment = StrictSlug::new(id_ulid.to_string());
  Key::new(model_name_segment).with(id_segment)
}

fn model_index_segment<M: models::Model>(index_name: &str) -> StrictSlug {
  StrictSlug::new(format!("{}_index_{}", M::TABLE_NAME, index_name))
}

/// Rollback fn for "recoverable" - effectively, *consumable* - errors.
pub(crate) async fn rollback<T: KvTransaction>(mut txn: T) -> Result<()> {
  txn
    .rollback()
    .await
    .context("failed to rollback transaction")
}

/// Rollback fn for "unrecoverable" errors (unexpected error paths).
pub(crate) async fn rollback_error<T: KvTransaction>(
  txn: T,
  error: miette::Report,
  context: &'static str,
) -> miette::Report {
  if let Err(e) = rollback(txn).await {
    tracing::error!("failed to rollback transaction: {:?}", e);
    return e;
  }
  let e = error.wrap_err(context);
  tracing::error!("{:?}", e);
  e
}

pub(crate) async fn commit<T: KvTransaction>(mut txn: T) -> Result<()> {
  txn.commit().await.context("failed to commit transaction")
}

/// Errors that can occur when creating a model.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum CreateModelError {
  /// A model with that ID already exists.
  #[error("model with that ID already exists")]
  ModelAlreadyExists,
  /// An index with that value already exists.
  #[error("index {index_name:?} with value \"{index_value}\" already exists")]
  IndexAlreadyExists {
    /// The name of the index.
    index_name:  String,
    /// The value of the index.
    index_value: EitherSlug,
  },
  /// A database error occurred.
  #[error("db error: {0}")]
  #[diagnostic_source]
  DbError(miette::Report),
}

impl From<miette::Report> for CreateModelError {
  fn from(e: miette::Report) -> Self { Self::DbError(e) }
}

impl<T: KvTransactional> DbConnection<T> {
  /// Creates a new model.
  #[instrument(skip(self, model), fields(id = model.id().to_string(), table = M::TABLE_NAME))]
  pub async fn create_model<M: models::Model>(
    &self,
    model: &M,
  ) -> Result<(), CreateModelError> {
    tracing::info!(
      "creating model with id `{}` on table {:?}",
      model.id(),
      M::TABLE_NAME
    );

    // the model itself will be stored under [model_name]:[id] -> model
    // and each index will be stored under
    // [model_name]_index_[index_name]:[index_value] -> [id]

    // calculate the key for the model
    let model_key = model_key::<M>(&model.id());
    let id_ulid: models::Ulid = model.id().clone().into();

    // serialize the model into bytes
    let model_value = kv::value::Value::serialize(&model)
      .into_diagnostic()
      .context("failed to serialize model")
      .map_err(CreateModelError::DbError)?;

    // serialize the id into bytes
    let id_value = kv::value::Value::serialize(&id_ulid)
      .into_diagnostic()
      .context("failed to serialize id")?;

    // begin a transaction
    let txn = self
      .0
      .begin_pessimistic_transaction()
      .await
      .context("failed to begin pessimistic transaction")
      .map_err(CreateModelError::DbError)?;

    // check if the model exists
    let (txn, exists) = txn
      .csm_exists(&model_key)
      .await
      .context("failed to check if model exists")?;
    if exists {
      return Err(CreateModelError::ModelAlreadyExists);
    }

    // insert the model
    let mut txn = txn
      .csm_insert(&model_key, model_value)
      .await
      .context("failed to insert model")?;

    // insert the indexes
    for (index_name, index_fn) in M::INDICES.iter() {
      // calculate the key for the index
      let index_key = kv::key::Key::new(model_index_segment::<M>(index_name))
        .with_either(index_fn(model));

      // check if the index exists already
      let (_txn, exists) = txn
        .csm_exists(&index_key)
        .await
        .context("failed to check if index exists")?;
      txn = _txn;
      if exists {
        return Err(CreateModelError::IndexAlreadyExists {
          index_name:  index_name.to_string(),
          index_value: index_fn(model),
        });
      }

      // insert the index
      txn = txn
        .csm_insert(&index_key, id_value.clone())
        .await
        .context("failed to insert index")?;
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

    let txn = self
      .0
      .begin_optimistic_transaction()
      .await
      .context("failed to begin optimistic transaction")?;

    let (txn, model_value) = txn.csm_get(&model_key).await?;

    commit(txn).await?;

    model_value
      .map(|value| kv::value::Value::deserialize(value))
      .transpose()
      .into_diagnostic()
      .context("failed to deserialize model")
  }

  /// Fetches a model by an index.
  ///
  /// Must be a valid index, defined in the model's `INDICES` constant.
  #[instrument(skip(self))]
  pub async fn fetch_model_by_index<M: models::Model>(
    &self,
    index_name: &str,
    index_value: &StrictSlug,
  ) -> Result<Option<M>> {
    let index_key = kv::key::Key::new(model_index_segment::<M>(index_name))
      .with(index_value.clone());

    let txn = self
      .0
      .begin_optimistic_transaction()
      .await
      .context("failed to begin optimistic transaction")?;

    let (txn, id_value) = txn.csm_get(&index_key).await?;

    commit(txn).await?;

    let id = id_value
      .map(kv::value::Value::deserialize::<M::Id>)
      .transpose()
      .into_diagnostic()
      .context("failed to deserialize id")?;

    let id = match id {
      Some(id) => id,
      None => return Ok(None),
    };

    let model = match self.fetch_model_by_id::<M>(&id).await? {
      Some(model) => model,
      None => {
        miette::bail!(
          "model with id `{}` not found, but index {:?} exists",
          id,
          index_name
        );
      }
    };

    Ok(Some(model))
  }
}
