//! TiKV key-value store implementation.

use std::mem::ManuallyDrop;

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
}

impl KvTransactional for TikvClient {
  type OptimisticTransaction = TikvTransaction;
  type PessimisticTransaction = TikvTransaction;

  async fn begin_optimistic_transaction(
    &self,
  ) -> KvResult<Self::OptimisticTransaction> {
    Ok(TikvTransaction(ManuallyDrop::new(
      self.0.begin_optimistic().await?,
    )))
  }

  async fn begin_pessimistic_transaction(
    &self,
  ) -> KvResult<Self::PessimisticTransaction> {
    Ok(TikvTransaction(ManuallyDrop::new(
      self.0.begin_pessimistic().await?,
    )))
  }
}

/// TiKV transaction.
pub struct TikvTransaction(ManuallyDrop<tikv_client::Transaction>);

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
