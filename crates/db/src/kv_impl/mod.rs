//! Key-value store implementation.

mod consumptive;

use std::{ops::Bound, sync::LazyLock};

use hex::health;
use kv::prelude::*;
use miette::{Context, IntoDiagnostic, Result};
use tracing::instrument;

use self::consumptive::ConsumptiveTransaction;
use crate::{
  adapter::{FetchModelByIndexError, FetchModelError},
  CreateModelError, DatabaseAdapter,
};

/// A TiKV-based database adapter.
#[derive(Clone)]
pub struct KvDatabaseAdapter<KV: KvTransactional>(KV);

impl<KV: KvTransactional> KvDatabaseAdapter<KV> {
  /// Creates a new TiKV adapter.
  pub fn new(kv_store: KV) -> Self {
    tracing::info!("creating new `KvDatabaseAdapter` instance");
    Self(kv_store)
  }
}

static INDEX_NS_SEGMENT: LazyLock<StrictSlug> =
  LazyLock::new(|| StrictSlug::new("index".to_string()));
static MODEL_NS_SEGMENT: LazyLock<StrictSlug> =
  LazyLock::new(|| StrictSlug::new("model".to_string()));

fn model_base_key<M: models::Model>(id: &models::RecordId<M>) -> Key {
  let id_ulid: models::Ulid = (*id).into();
  Key::new_lazy(&MODEL_NS_SEGMENT)
    .with(StrictSlug::new(M::TABLE_NAME.to_string()))
    .with(StrictSlug::new(id_ulid.to_string()))
}

fn index_base_key<M: models::Model>(index_name: &str) -> Key {
  Key::new_lazy(&INDEX_NS_SEGMENT)
    .with(StrictSlug::new(M::TABLE_NAME.to_string()))
    .with(StrictSlug::new(index_name))
}

#[async_trait::async_trait]
impl<KV: KvTransactional> DatabaseAdapter for KvDatabaseAdapter<KV> {
  #[instrument(skip(self, model), fields(id = model.id().to_string(), table = M::TABLE_NAME))]
  async fn create_model<M: models::Model>(
    &self,
    model: M,
  ) -> Result<M, CreateModelError> {
    tracing::info!("creating model");

    // the model itself will be stored under [model_name]:[id] -> model
    // and each index will be stored under
    // [model_name]_index_[index_name]:[index_value] -> [id]

    // calculate the key for the model
    let model_key = model_base_key::<M>(&model.id());
    let id_ulid: models::Ulid = model.id().into();

    // serialize the model into bytes
    let model_value = kv::value::Value::serialize(&model)
      .into_diagnostic()
      .context("failed to serialize model")
      .map_err(CreateModelError::Serde)?;

    // serialize the id into bytes
    let id_value = kv::value::Value::serialize(&id_ulid)
      .into_diagnostic()
      .context("failed to serialize id")
      .map_err(CreateModelError::Serde)?;

    // begin a transaction
    let txn = self
      .0
      .begin_pessimistic_transaction()
      .await
      .context("failed to begin pessimistic transaction")
      .map_err(CreateModelError::Db)?;

    // check if the model exists
    let (txn, exists) = txn
      .csm_exists(&model_key)
      .await
      .context("failed to check if model exists")
      .map_err(CreateModelError::Db)?;
    if exists {
      return Err(CreateModelError::ModelAlreadyExists);
    }

    // insert the model
    let mut txn = txn
      .csm_insert(&model_key, model_value)
      .await
      .context("failed to insert model")
      .map_err(CreateModelError::Db)?;

    // insert the indexes
    for (index_name, index_fn) in M::UNIQUE_INDICES.iter() {
      // calculate the key for the index
      let index_key =
        index_base_key::<M>(index_name).with_either(index_fn(&model));

      // check if the index exists already
      let (_txn, exists) = txn
        .csm_exists(&index_key)
        .await
        .context("failed to check if index exists")
        .map_err(CreateModelError::Db)?;
      txn = _txn;
      if exists {
        return Err(CreateModelError::IndexAlreadyExists {
          index_name:  index_name.to_string(),
          index_value: index_fn(&model),
        });
      }

      // insert the index
      txn = txn
        .csm_insert(&index_key, id_value.clone())
        .await
        .context("failed to insert index")
        .map_err(CreateModelError::Db)?;
    }

    txn
      .to_commit()
      .await
      .map_err(CreateModelError::RetryableTransaction)?;

    Ok(model)
  }

  #[instrument(skip(self), fields(table = M::TABLE_NAME))]
  async fn fetch_model_by_id<M: models::Model>(
    &self,
    id: models::RecordId<M>,
  ) -> Result<Option<M>, FetchModelError> {
    tracing::info!("fetching model with id");

    let model_key = model_base_key::<M>(&id);

    let txn = self
      .0
      .begin_optimistic_transaction()
      .await
      .context("failed to begin optimistic transaction")
      .map_err(FetchModelError::RetryableTransaction)?;

    let (txn, model_value) =
      txn.csm_get(&model_key).await.map_err(FetchModelError::Db)?;

    txn
      .to_commit()
      .await
      .map_err(FetchModelError::RetryableTransaction)?;

    model_value
      .map(|value| kv::value::Value::deserialize(value))
      .transpose()
      .into_diagnostic()
      .context("failed to deserialize model")
      .map_err(FetchModelError::Serde)
  }

  #[instrument(skip(self), fields(table = M::TABLE_NAME))]
  async fn fetch_model_by_index<M: models::Model>(
    &self,
    index_name: String,
    index_value: EitherSlug,
  ) -> Result<Option<M>, FetchModelByIndexError> {
    tracing::info!("fetching model by index");

    if !M::UNIQUE_INDICES
      .iter()
      .any(|(name, _)| name == &index_name)
    {
      return Err(FetchModelByIndexError::IndexDoesNotExistOnModel {
        index_name,
      });
    }

    let index_key =
      index_base_key::<M>(&index_name).with_either(index_value.clone());

    let txn = self
      .0
      .begin_optimistic_transaction()
      .await
      .context("failed to begin optimistic transaction")
      .map_err(FetchModelByIndexError::RetryableTransaction)?;

    let (txn, id_value) = txn
      .csm_get(&index_key)
      .await
      .map_err(FetchModelByIndexError::Db)?;

    txn
      .to_commit()
      .await
      .map_err(FetchModelByIndexError::RetryableTransaction)?;

    let id = id_value
      .map(kv::value::Value::deserialize::<models::RecordId<M>>)
      .transpose()
      .into_diagnostic()
      .context("failed to deserialize id")
      .map_err(FetchModelByIndexError::Serde)?;

    let id = match id {
      Some(id) => id,
      None => return Ok(None),
    };

    let model = match self
      .fetch_model_by_id::<M>(id)
      .await
      .map_err(FetchModelByIndexError::from)?
    {
      Some(model) => model,
      None => {
        return Err(FetchModelByIndexError::IndexMalformed {
          index_name,
          index_value,
        });
      }
    };

    Ok(Some(model))
  }

  #[instrument(skip(self), fields(table = M::TABLE_NAME))]
  async fn enumerate_models<M: models::Model>(&self) -> Result<Vec<M>> {
    let first_key = model_base_key::<M>(&models::RecordId::<M>::MIN());
    let last_key = model_base_key::<M>(&models::RecordId::<M>::MAX());

    let txn = self
      .0
      .begin_optimistic_transaction()
      .await
      .context("failed to begin optimistic transaction")
      .map_err(FetchModelError::RetryableTransaction)?;

    let (txn, scan_results) = txn
      .csm_scan(Bound::Included(first_key), Bound::Included(last_key), 1000)
      .await
      .map_err(FetchModelError::Db)?;

    txn
      .to_commit()
      .await
      .map_err(FetchModelError::RetryableTransaction)?;

    let ids = scan_results
      .into_iter()
      .map(|(_, value)| {
        Value::deserialize::<M>(value)
          .into_diagnostic()
          .context("failed to deserialize value into model")
          .map_err(FetchModelError::Serde)
          .map_err(miette::Report::from)
      })
      .collect::<Result<Vec<M>>>()?;

    Ok(ids)
  }
}

#[async_trait::async_trait]
impl<KV: KvTransactional> health::HealthReporter for KvDatabaseAdapter<KV> {
  fn name(&self) -> &'static str { stringify!(TikvAdapter) }
  async fn health_check(&self) -> health::ComponentHealth {
    health::AdditiveComponentHealth::from_futures(Some(self.0.health_report()))
      .await
      .into()
  }
}
