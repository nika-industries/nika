//! TiKV key-value store implementation of [`KvTransactional`].

use std::ops::Bound;

use hex::health;
use miette::{Context, IntoDiagnostic};

use crate::{
  key::Key, value::Value, KvPrimitive, KvResult, KvTransaction, KvTransactional,
};

impl From<Key> for tikv_client::Key {
  fn from(key: Key) -> Self { key.to_string().into() }
}

/// TiKV client.
pub struct TikvClient(tikv_client::TransactionClient);

impl TikvClient {
  /// Create a new TiKV client.
  pub async fn new(endpoints: Vec<&str>) -> KvResult<Self> {
    Ok(TikvClient(
      tikv_client::TransactionClient::new(endpoints).await?,
    ))
  }

  /// Create a new TiKV client from environment variables.
  pub async fn new_from_env() -> miette::Result<Self> {
    let urls = std::env::var("TIKV_URLS")
      .into_diagnostic()
      .wrap_err("missing TIKV_URLS")?;
    let urls = urls.split(',').collect();
    let client = TikvClient::new(urls)
      .await
      .into_diagnostic()
      .context("failed to create tikv client")?;
    Ok(client)
  }
}

#[health::async_trait]
impl health::HealthReporter for TikvClient {
  fn name(&self) -> &'static str { stringify!(TikvClient) }
  async fn health_check(&self) -> health::ComponentHealth {
    health::IntrensicallyUp.into()
  }
}

impl KvTransactional for TikvClient {
  type OptimisticTransaction = TikvTransaction;
  type PessimisticTransaction = TikvTransaction;

  async fn begin_optimistic_transaction(
    &self,
  ) -> KvResult<Self::OptimisticTransaction> {
    Ok(TikvTransaction(self.0.begin_optimistic().await?))
  }

  async fn begin_pessimistic_transaction(
    &self,
  ) -> KvResult<Self::PessimisticTransaction> {
    Ok(TikvTransaction(self.0.begin_pessimistic().await?))
  }
}

/// TiKV transaction.
pub struct TikvTransaction(tikv_client::Transaction);

impl KvPrimitive for TikvTransaction {
  async fn get(&mut self, key: &Key) -> KvResult<Option<Value>> {
    Ok(self.0.get(key.clone()).await?.map(Value::from))
  }

  async fn put(&mut self, key: &Key, value: Value) -> KvResult<()> {
    self.0.put(key.clone(), value).await?;
    Ok(())
  }

  async fn insert(&mut self, key: &Key, value: Value) -> KvResult<()> {
    self.0.insert(key.clone(), value).await?;
    Ok(())
  }

  async fn scan(
    &mut self,
    start: Bound<Key>,
    end: Bound<Key>,
    limit: Option<u32>,
  ) -> KvResult<Vec<(Key, Value)>> {
    match limit {
      Some(limit) => scan(self, start, end, limit).await,
      None => scan_unlimited(self, start, end).await,
    }
  }
}

impl KvTransaction for TikvTransaction {
  async fn commit(&mut self) -> KvResult<()> {
    self.0.commit().await?;
    Ok(())
  }

  async fn rollback(&mut self) -> KvResult<()> {
    self.0.rollback().await?;
    Ok(())
  }
}

async fn scan(
  txn: &mut TikvTransaction,
  start: Bound<Key>,
  end: Bound<Key>,
  limit: u32,
) -> KvResult<Vec<(Key, Value)>> {
  let start_bound: Bound<tikv_client::Key> = match start {
    Bound::Included(k) => Bound::Included(k.into()),
    Bound::Excluded(k) => Bound::Excluded(k.into()),
    Bound::Unbounded => Bound::Unbounded,
  };
  let end_bound: Bound<tikv_client::Key> = match end {
    Bound::Included(k) => Bound::Included(k.into()),
    Bound::Excluded(k) => Bound::Excluded(k.into()),
    Bound::Unbounded => Bound::Unbounded,
  };
  let range = tikv_client::BoundRange {
    from: start_bound,
    to:   end_bound,
  };

  Ok(
    txn
      .0
      .scan(range, limit)
      .await?
      .filter_map(|kp| match Key::try_from(Vec::<u8>::from(kp.0)) {
        Ok(key) => Some((key, Value::from(kp.1))),
        Err(e) => {
          tracing::error!("failed to parse key from kv store: {}", e);
          None
        }
      })
      .collect(),
  )
}

async fn scan_unlimited(
  txn: &mut TikvTransaction,
  start: Bound<Key>,
  end: Bound<Key>,
) -> KvResult<Vec<(Key, Value)>> {
  let mut start_bound: Bound<tikv_client::Key> = match start {
    Bound::Included(k) => Bound::Included(k.into()),
    Bound::Excluded(k) => Bound::Excluded(k.into()),
    Bound::Unbounded => Bound::Unbounded,
  };
  let end_bound: Bound<tikv_client::Key> = match end {
    Bound::Included(k) => Bound::Included(k.into()),
    Bound::Excluded(k) => Bound::Excluded(k.into()),
    Bound::Unbounded => Bound::Unbounded,
  };

  let mut results = Vec::new();
  loop {
    let range = tikv_client::BoundRange {
      from: start_bound,
      to:   end_bound.clone(),
    };
    let scan_result = txn.0.scan(range.clone(), 1000).await?;
    let scan = scan_result
      .filter_map(|kp| match Key::try_from(Vec::<u8>::from(kp.0)) {
        Ok(key) => Some((key, Value::from(kp.1))),
        Err(e) => {
          tracing::error!("failed to parse key from kv store: {}", e);
          None
        }
      })
      .collect::<Vec<_>>();

    if scan.is_empty() {
      break;
    }

    start_bound = Bound::Excluded(scan.last().unwrap().0.clone().into());
    results.extend(scan);
  }

  Ok(results)
}
