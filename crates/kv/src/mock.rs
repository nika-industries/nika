//! A mock implementation of [`KvTransactional`]. Follows TiKV semantics around
//! transactions.

use std::{
  collections::{HashMap, HashSet},
  ops::Bound,
  sync::Arc,
};

use hex::health;
use tokio::sync::{Mutex, RwLock};

use crate::{
  key::Key, value::Value, KvError, KvPrimitive, KvResult, KvTransaction,
  KvTransactional,
};

/// A mock key-value store.
pub struct MockStore {
  data:  RwLock<HashMap<Key, Value>>,
  locks: Mutex<HashSet<Key>>, // Set of keys currently locked
}

impl MockStore {
  /// Create a new mock store.
  pub fn new() -> Arc<Self> {
    Arc::new(Self {
      data:  RwLock::new(HashMap::new()),
      locks: Mutex::new(HashSet::new()),
    })
  }

  /// Screw with the internal data of the store. This is useful for testing.
  pub fn screw_with_internal_data(&self) -> &RwLock<HashMap<Key, Value>> {
    &self.data
  }
}

impl KvTransactional for Arc<MockStore> {
  type OptimisticTransaction = OptimisticTransaction;
  type PessimisticTransaction = PessimisticTransaction;

  async fn begin_optimistic_transaction(
    &self,
  ) -> KvResult<Self::OptimisticTransaction> {
    Ok(OptimisticTransaction {
      store:     self.clone(),
      read_set:  HashMap::new(),
      write_set: HashMap::new(),
    })
  }

  async fn begin_pessimistic_transaction(
    &self,
  ) -> KvResult<Self::PessimisticTransaction> {
    Ok(PessimisticTransaction {
      store:       self.clone(),
      locked_keys: HashSet::new(),
      write_set:   HashMap::new(),
    })
  }
}

#[health::async_trait]
impl health::HealthReporter for MockStore {
  fn name(&self) -> &'static str { stringify!(MockStore) }
  async fn health_check(&self) -> health::ComponentHealth {
    health::IntrensicallyUp.into()
  }
}

/// A transaction error.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum TransactionError {
  /// The key was locked.
  #[error("key locked: `{0}`")]
  KeyLocked(Key),
  /// There was a key conflict.
  #[error("key conflict: `{0}`")]
  KeyConflict(Key),
  /// Some other error occurred.
  #[error("other: {0:?}")]
  Other(String),
}

impl From<TransactionError> for KvError {
  fn from(error: TransactionError) -> Self {
    KvError::PlatformError(error.into())
  }
}

/// An optimistic transaction.
pub struct OptimisticTransaction {
  store:     Arc<MockStore>,
  read_set:  HashMap<Key, Option<Value>>,
  write_set: HashMap<Key, Value>,
}

impl Drop for OptimisticTransaction {
  fn drop(&mut self) {
    if !self.read_set.is_empty() || !self.write_set.is_empty() {
      panic!(
        "Optimistic transaction dropped without commit or rollback. Read Set: \
         {:?}, Write Set: {:?}",
        self.read_set, self.write_set
      );
    }
  }
}

impl OptimisticTransaction {
  async fn check_conflicts(&self) -> Result<(), TransactionError> {
    for (key, value) in &self.read_set {
      if value.as_ref() != self.store.data.read().await.get(key) {
        return Err(TransactionError::KeyConflict(key.clone()));
      }
    }
    Ok(())
  }
}

impl KvPrimitive for OptimisticTransaction {
  async fn get(&mut self, key: &Key) -> KvResult<Option<Value>> {
    let data = self.store.data.read().await;
    let value = data.get(key).cloned();
    self.read_set.insert(key.clone(), value.clone());
    Ok(value)
  }
  async fn put(&mut self, key: &Key, value: Value) -> KvResult<()> {
    self
      .read_set
      .insert(key.clone(), self.store.data.read().await.get(key).cloned());
    self.write_set.insert(key.clone(), value);
    Ok(())
  }
  async fn insert(&mut self, key: &Key, value: Value) -> KvResult<()> {
    let data = self.store.data.write().await;
    if data.contains_key(key) {
      return Err(KvError::PlatformError(miette::miette!(
        "Key already exists"
      )));
    }
    self.read_set.insert(key.clone(), data.get(key).cloned());
    self.write_set.insert(key.clone(), value.clone());
    Ok(())
  }
  async fn scan(
    &mut self,
    start: Bound<Key>,
    end: Bound<Key>,
    limit: Option<u32>,
  ) -> KvResult<Vec<(Key, Value)>> {
    let data = self.store.data.read().await;
    let mut result = Vec::new();
    for (key, value) in data.iter() {
      if match &start {
        Bound::Included(start) => key >= start,
        Bound::Excluded(start) => key > start,
        Bound::Unbounded => true,
      } && match &end {
        Bound::Included(end) => key <= end,
        Bound::Excluded(end) => key < end,
        Bound::Unbounded => true,
      } {
        self.read_set.insert(key.clone(), Some(value.clone()));
        result.push((key.clone(), value.clone()));
        if let Some(limit) = limit {
          if result.len() == limit as usize {
            break;
          }
        }
      }
    }
    Ok(result)
  }
  async fn delete(&mut self, key: &Key) -> KvResult<bool> {
    self
      .read_set
      .insert(key.clone(), self.store.data.read().await.get(key).cloned());
    self.write_set.insert(key.clone(), Value::new(vec![]));
    Ok(true)
  }
}

impl KvTransaction for OptimisticTransaction {
  async fn commit(&mut self) -> KvResult<()> {
    self.check_conflicts().await?;
    let mut data = self.store.data.write().await;
    for (key, value) in self.write_set.drain() {
      data.insert(key, value);
    }
    self.read_set.clear();
    Ok(())
  }

  async fn rollback(&mut self) -> KvResult<()> {
    self.read_set.clear();
    self.write_set.clear();
    Ok(())
  }
}

/// A pessimistic transaction.
pub struct PessimisticTransaction {
  store:       Arc<MockStore>,
  locked_keys: HashSet<Key>,
  write_set:   HashMap<Key, Value>,
}

impl Drop for PessimisticTransaction {
  fn drop(&mut self) {
    if !self.locked_keys.is_empty() || !self.write_set.is_empty() {
      panic!(
        "Pessimistic transaction dropped without commit or rollback. Keys: \
         {:?}, Write Set: {:?}",
        self.locked_keys, self.write_set
      );
    }
  }
}

impl PessimisticTransaction {
  async fn lock_key(&mut self, key: &Key) -> Result<(), TransactionError> {
    if self.locked_keys.contains(key) {
      return Ok(());
    }
    let mut locks = self.store.locks.lock().await;
    if locks.contains(key) {
      return Err(TransactionError::KeyLocked(key.clone()));
    }
    locks.insert(key.clone());
    self.locked_keys.insert(key.clone());
    Ok(())
  }

  async fn unlock_keys(&self) {
    let mut locks = self.store.locks.lock().await;
    for key in &self.locked_keys {
      locks.remove(key);
    }
  }
}

impl KvPrimitive for PessimisticTransaction {
  async fn get(&mut self, key: &Key) -> KvResult<Option<Value>> {
    self.lock_key(key).await?;
    let data = self.store.data.read().await;
    Ok(data.get(key).cloned())
  }
  async fn put(&mut self, key: &Key, value: Value) -> KvResult<()> {
    self.lock_key(key).await?;
    self.write_set.insert(key.clone(), value);
    Ok(())
  }
  async fn insert(&mut self, key: &Key, value: Value) -> KvResult<()> {
    self.lock_key(key).await?;
    if self.store.data.read().await.contains_key(key) {
      return Err(KvError::PlatformError(miette::miette!(
        "Key already exists"
      )));
    }
    self.write_set.insert(key.clone(), value);
    Ok(())
  }
  async fn scan(
    &mut self,
    start: Bound<Key>,
    end: Bound<Key>,
    limit: Option<u32>,
  ) -> KvResult<Vec<(Key, Value)>> {
    let data = self.store.data.read().await;
    let mut result = Vec::new();
    for (key, value) in data.iter() {
      if match &start {
        Bound::Included(start) => key >= start,
        Bound::Excluded(start) => key > start,
        Bound::Unbounded => true,
      } && match &end {
        Bound::Included(end) => key <= end,
        Bound::Excluded(end) => key < end,
        Bound::Unbounded => true,
      } {
        result.push((key.clone(), value.clone()));
        if let Some(limit) = limit {
          if result.len() == limit as usize {
            break;
          }
        }
      }
    }
    Ok(result)
  }
  async fn delete(&mut self, key: &Key) -> KvResult<bool> {
    self.lock_key(key).await?;
    self.write_set.insert(key.clone(), Value::new(vec![]));
    Ok(true)
  }
}

impl KvTransaction for PessimisticTransaction {
  async fn commit(&mut self) -> KvResult<()> {
    let mut data = self.store.data.write().await;
    for (key, value) in self.write_set.drain() {
      data.insert(key, value);
    }
    self.unlock_keys().await;
    self.locked_keys.clear();
    self.write_set.clear();
    Ok(())
  }

  async fn rollback(&mut self) -> KvResult<()> {
    self.unlock_keys().await;
    self.locked_keys.clear();
    self.write_set.clear();
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::{ops::Bound, sync::Arc};

  use slugger::StrictSlug;

  use super::*;

  #[tokio::test]
  async fn test_optimistic_transaction() {
    let store = Arc::new(MockStore {
      data:  RwLock::new(HashMap::new()),
      locks: Mutex::new(HashSet::new()),
    });

    let mut txn = store.begin_optimistic_transaction().await.unwrap();

    // Test inserting a key-value pair
    let key = Key::new(StrictSlug::new("key1"));
    let value = Value::from("value1");
    txn.put(&key, value.clone()).await.unwrap();

    // Commit the transaction
    txn.commit().await.unwrap();

    // Verify the value is in the store
    let read_value = store.data.read().await.get(&key).cloned();
    assert_eq!(read_value, Some(value));
  }

  #[tokio::test]
  async fn test_pessimistic_transaction() {
    let store = Arc::new(MockStore {
      data:  RwLock::new(HashMap::new()),
      locks: Mutex::new(HashSet::new()),
    });

    let mut txn = store.begin_pessimistic_transaction().await.unwrap();

    // Test inserting a key-value pair
    let key = Key::new(StrictSlug::new("key2"));
    let value = Value::from("value2");
    txn.put(&key, value.clone()).await.unwrap();

    // Commit the transaction
    txn.commit().await.unwrap();

    // Verify the value is in the store
    let read_value = store.data.read().await.get(&key).cloned();
    assert_eq!(read_value, Some(value));
  }

  #[tokio::test]
  async fn test_conflict_in_optimistic_transaction() {
    let store = Arc::new(MockStore {
      data:  RwLock::new(HashMap::new()),
      locks: Mutex::new(HashSet::new()),
    });

    // Insert initial data
    store
      .data
      .write()
      .await
      .insert(Key::new(StrictSlug::new("key3")), Value::from("value3"));

    let mut txn = store.begin_optimistic_transaction().await.unwrap();

    // Read the key to add it to the read set
    txn
      .get(&Key::new(StrictSlug::new("key3")))
      .await
      .unwrap()
      .unwrap();

    // Start a new transaction that will conflict
    let mut txn2 = store.begin_optimistic_transaction().await.unwrap();

    // Modify the key that was read in txn
    txn2
      .put(
        &Key::new(StrictSlug::new("key3")),
        Value::from("other_value"),
      )
      .await
      .unwrap();

    txn2.commit().await.unwrap();

    // Commit should fail due to conflict
    let commit_result = txn.commit().await;
    assert!(commit_result.is_err());

    txn.rollback().await.unwrap();
  }

  #[tokio::test]
  async fn test_lock_in_pessimistic_transaction() {
    let store = Arc::new(MockStore {
      data:  RwLock::new(HashMap::new()),
      locks: Mutex::new(HashSet::new()),
    });

    let mut txn1 = store.begin_pessimistic_transaction().await.unwrap();
    let mut txn2 = store.begin_pessimistic_transaction().await.unwrap();

    let key = Key::new(StrictSlug::new("key4"));

    // Lock the key in txn1
    txn1.put(&key, Value::from("value4")).await.unwrap();

    // Attempt to modify the same key in txn2
    let result = txn2.put(&key, Value::from("other_value")).await;

    // Verify txn2 fails due to lock
    assert!(result.is_err());

    // Commit txn1 and release the lock
    txn1.commit().await.unwrap();

    // Now txn2 should succeed
    let result = txn2.put(&key, Value::from("other_value")).await;
    assert!(result.is_ok());

    txn2.rollback().await.unwrap();
  }

  #[tokio::test]
  async fn test_scan_operation() {
    let store = Arc::new(MockStore {
      data:  RwLock::new(HashMap::new()),
      locks: Mutex::new(HashSet::new()),
    });

    // Populate store with test data
    let keys_values = vec![
      (Key::new(StrictSlug::new("a")), Value::from("1")),
      (Key::new(StrictSlug::new("b")), Value::from("2")),
      (Key::new(StrictSlug::new("c")), Value::from("3")),
    ];
    {
      let mut data = store.data.write().await;
      for (key, value) in &keys_values {
        data.insert(key.clone(), value.clone());
      }
    }

    let mut txn = store.begin_optimistic_transaction().await.unwrap();

    // Scan a range
    let result = txn
      .scan(
        Bound::Included(Key::new(StrictSlug::new("a"))),
        Bound::Included(Key::new(StrictSlug::new("b"))),
        None,
      )
      .await
      .unwrap();

    assert_eq!(result.len(), 2);

    // switch to hashmap for comparison
    let result_map: HashMap<Key, Value> = result.into_iter().collect();
    let expected_map: HashMap<Key, Value> = keys_values
      .iter()
      .filter(|(key, _)| *(key.to_string()) <= *"b")
      .cloned()
      .collect();
    assert_eq!(result_map, expected_map);

    txn.commit().await.unwrap();
  }
}
