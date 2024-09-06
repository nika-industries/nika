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
  let model_name_segment = StrictSlug::new(M::TABLE_NAME);
  let id_ulid: models::Ulid = id.clone().into();
  let id_segment = StrictSlug::new(id_ulid.to_string());
  Key::new(model_name_segment).with(id_segment)
}

fn model_index_segment<M: models::Model>(index_name: &str) -> StrictSlug {
  StrictSlug::new(format!("{}_index_{}", M::TABLE_NAME, index_name))
}

/// Rollback fn for "recoverable" - effectively, *consumable* - errors.
async fn rollback<T: KvTransaction>(mut txn: T) -> Result<()> {
  txn
    .rollback()
    .await
    .into_diagnostic()
    .context("failed to rollback transaction")
}

/// Rollback fn for "unrecoverable" errors (unexpected error paths).
async fn rollback_error<T: KvTransaction>(
  txn: T,
  error: miette::Report,
  context: &'static str,
) -> miette::Report {
  if let Err(e) = rollback(txn).await {
    tracing::error!("failed to rollback transaction: {:#}", e);
    return e;
  }
  let e = Err::<(), miette::Report>(error)
    .context(context)
    .unwrap_err();
  tracing::error!("{:#}", e);
  e
}

async fn commit<T: KvTransaction>(mut txn: T) -> Result<()> {
  txn
    .commit()
    .await
    .into_diagnostic()
    .context("failed to commit transaction")
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
    index_value: StrictSlug,
  },
  /// A database error occurred.
  #[error("db error: {0}")]
  #[diagnostic_source]
  DbError(miette::Report),
}

impl<T: KvTransactional> DbConnection<T> {
  /// Creates a new model.
  #[instrument(skip(self))]
  pub async fn create_model<M: models::Model>(
    &self,
    model: &M,
  ) -> Result<(), CreateModelError> {
    tracing::debug!(
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
      .context("failed to serialize id")
      .map_err(CreateModelError::DbError)?;

    // begin a transaction
    let mut txn = self
      .0
      .begin_pessimistic_transaction()
      .await
      .into_diagnostic()
      .context("failed to begin pessimistic transaction")
      .map_err(CreateModelError::DbError)?;

    // check if the model exists already
    match txn.get(&model_key).await {
      Ok(Some(_)) => {
        rollback(txn).await.map_err(CreateModelError::DbError)?;
        return Err(CreateModelError::ModelAlreadyExists);
      }
      Ok(None) => {}
      Err(e) => {
        return Err(CreateModelError::DbError(
          rollback_error(
            txn,
            e.into(),
            "failed to check if model already exists",
          )
          .await,
        ));
      }
    }

    // insert the model
    if let Err(e) = txn.insert(&model_key, model_value).await {
      return Err(CreateModelError::DbError(
        rollback_error(txn, e.into(), "failed to insert model").await,
      ));
    }

    // insert the indexes
    for (index_name, index_fn) in M::INDICES.iter() {
      // calculate the key for the index
      let index_key = kv::key::Key::new(model_index_segment::<M>(index_name))
        .with(index_fn(model));

      // check if the index exists already
      match txn.get(&index_key).await {
        Ok(Some(_)) => {
          rollback(txn).await.map_err(CreateModelError::DbError)?;
          return Err(CreateModelError::IndexAlreadyExists {
            index_name:  index_name.to_string(),
            index_value: index_fn(model),
          });
        }
        Ok(None) => {}
        Err(e) => {
          return Err(CreateModelError::DbError(
            rollback_error(
              txn,
              e.into(),
              "failed to check if index already exists",
            )
            .await,
          ));
        }
      }

      // insert the index
      if let Err(e) = txn.insert(&index_key, id_value.clone()).await {
        return Err(CreateModelError::DbError(
          rollback_error(txn, e.into(), "failed to insert index").await,
        ));
      }
    }

    commit(txn).await.map_err(CreateModelError::DbError)?;

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
        return Err(rollback_error(txn, e.into(), "failed to get model").await);
      }
    };

    match model_value {
      Some(value) => match kv::value::Value::deserialize(value) {
        Ok(value) => {
          commit(txn).await?;
          Ok(Some(value))
        }
        Err(e) => {
          return Err(
            rollback_error(
              txn,
              Err::<(), _>(e).into_diagnostic().unwrap_err(),
              "failed to deserialize model",
            )
            .await,
          );
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
    index_value: &StrictSlug,
  ) -> Result<Option<M>> {
    let index_key = kv::key::Key::new(model_index_segment::<M>(index_name))
      .with(index_value.clone());

    let mut txn = self
      .0
      .begin_optimistic_transaction()
      .await
      .into_diagnostic()
      .context("failed to begin optimistic transaction")?;

    let id_value = match txn.get(&index_key).await {
      Ok(value) => value,
      Err(e) => {
        return Err(rollback_error(txn, e.into(), "failed to get id").await);
      }
    };

    let id = match id_value {
      Some(value) => match kv::value::Value::deserialize(value) {
        Ok(value) => value,
        Err(e) => {
          return Err(
            rollback_error(
              txn,
              Err::<(), _>(e).into_diagnostic().unwrap_err(),
              "failed to deserialize id",
            )
            .await,
          );
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
        return Err(
          rollback_error(txn, e, "failed to fetch model by id").await,
        );
      }
    };

    commit(txn).await?;

    Ok(model)
  }
}
