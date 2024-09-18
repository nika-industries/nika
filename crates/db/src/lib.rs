//! Provides access to the database.

mod consumptive;
mod migrate;

use std::{
  future::Future,
  sync::{Arc, LazyLock},
};

use kv::prelude::*;
use miette::{Context, IntoDiagnostic, Result};
use tracing::instrument;

use self::consumptive::ConsumptiveTransaction;
pub use self::migrate::Migratable;

/// A TiKV-based database adapter.
pub struct TikvAdapter(Arc<kv::tikv::TikvClient>);

impl TikvAdapter {
  /// Creates a new TiKV adapter.
  pub async fn new(endpoints: Vec<&str>) -> Result<Self> {
    Ok(Self(Arc::new(kv::tikv::TikvClient::new(endpoints).await?)))
  }

  /// Creates a new TiKV adapter from environment variables.
  pub async fn new_from_env() -> Result<Self> {
    Ok(Self(Arc::new(kv::tikv::TikvClient::new_from_env().await?)))
  }
}

impl KvTransactional for TikvAdapter {
  type OptimisticTransaction = kv::tikv::TikvTransaction;
  type PessimisticTransaction = kv::tikv::TikvTransaction;

  async fn begin_optimistic_transaction(
    &self,
  ) -> KvResult<Self::OptimisticTransaction> {
    self.0.begin_optimistic_transaction().await
  }

  async fn begin_pessimistic_transaction(
    &self,
  ) -> KvResult<Self::PessimisticTransaction> {
    self.0.begin_pessimistic_transaction().await
  }
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

/// An adapter for a model-based database.
pub trait DatabaseAdapter: Send + Sync + 'static {
  /// Creates a new model.
  fn create_model<M: models::Model>(
    &self,
    model: &M,
  ) -> impl Future<Output = Result<(), CreateModelError>> + Send;
  /// Fetches a model by its ID.
  fn fetch_model_by_id<M: models::Model>(
    &self,
    id: &M::Id,
  ) -> impl Future<Output = Result<Option<M>>> + Send;
  /// Fetches a model by an index.
  ///
  /// Must be a valid index, defined in the model's `INDICES` constant.
  fn fetch_model_by_index<M: models::Model>(
    &self,
    index_name: &str,
    index_value: &EitherSlug,
  ) -> impl Future<Output = Result<Option<M>>> + Send;
}

static INDEX_NS_SEGMENT: LazyLock<StrictSlug> =
  LazyLock::new(|| StrictSlug::new("index".to_string()));
static MODEL_NS_SEGMENT: LazyLock<StrictSlug> =
  LazyLock::new(|| StrictSlug::new("model".to_string()));

fn model_base_key<M: models::Model>(id: &M::Id) -> Key {
  let id_ulid: models::Ulid = id.clone().into();
  Key::new_lazy(&MODEL_NS_SEGMENT)
    .with(StrictSlug::new(M::TABLE_NAME.to_string()))
    .with(StrictSlug::new(id_ulid.to_string()))
}

fn index_base_key<M: models::Model>(index_name: &str) -> Key {
  Key::new_lazy(&INDEX_NS_SEGMENT)
    .with(StrictSlug::new(M::TABLE_NAME.to_string()))
    .with(StrictSlug::new(index_name))
}

/// Rollback fn for "recoverable" - effectively, *consumable* - errors.
pub(crate) async fn rollback<T: KvTransaction>(mut txn: T) -> Result<()> {
  txn
    .rollback()
    .await
    .context("failed to rollback transaction")
}

/// Rollback fn for "unrecoverable" errors (unexpected error paths).
pub(crate) async fn rollback_with_error<T: KvTransaction>(
  txn: T,
  error: miette::Report,
  context: &'static str,
) -> miette::Report {
  if let Err(e) = rollback(txn).await {
    tracing::error!("failed to rollback transaction: {:?}", e);
    return e;
  }
  let e = error.wrap_err(context);
  tracing::error!("unrecoverable rollback: {:?}", e);
  e
}

pub(crate) async fn commit<T: KvTransaction>(mut txn: T) -> Result<()> {
  txn.commit().await.context("failed to commit transaction")
}

impl<T> DatabaseAdapter for T
where
  T: KvTransactional + Send + Sync + 'static,
  <T as kv::KvTransactional>::OptimisticTransaction: Send,
  <T as kv::KvTransactional>::PessimisticTransaction: Send,
{
  #[instrument(skip(self, model), fields(id = model.id().to_string(), table = M::TABLE_NAME))]
  async fn create_model<M: models::Model>(
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
    let model_key = model_base_key::<M>(&model.id());
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
      let index_key =
        index_base_key::<M>(index_name).with_either(index_fn(model));

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

  #[instrument(skip(self))]
  async fn fetch_model_by_id<M: models::Model>(
    &self,
    id: &M::Id,
  ) -> Result<Option<M>> {
    let model_key = model_base_key::<M>(id);

    let txn = self
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

  #[instrument(skip(self))]
  async fn fetch_model_by_index<M: models::Model>(
    &self,
    index_name: &str,
    index_value: &EitherSlug,
  ) -> Result<Option<M>> {
    let index_key =
      index_base_key::<M>(index_name).with_either(index_value.clone());

    let txn = self
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
