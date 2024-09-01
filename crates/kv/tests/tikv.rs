use std::sync::LazyLock;

use kv::{
  key::Key, tikv::TikvClient, value::Value, KvPrimitive, KvTransaction,
  KvTransactional,
};
use serde::{Deserialize, Serialize};
use slugger::StrictSlug;

static GLOBAL_NS_SEGMENT: LazyLock<StrictSlug> =
  LazyLock::new(|| StrictSlug::new("nika".to_string()));

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Foo {
  pub bar: i32,
}

#[tokio::test]
#[ignore]
async fn test_tikv() {
  let client = TikvClient::new(vec!["localhost:2379"]).await.unwrap();
  let mut txn = client.begin_pessimistic_transaction().await.unwrap();

  let key = Key::new_lazy(&GLOBAL_NS_SEGMENT);
  let foo = Foo { bar: 42 };
  let value = Value::serialize(&foo).unwrap();

  txn.put(&key, value.clone()).await.unwrap();
  let value = txn.get(&key).await.unwrap().unwrap();
  assert_eq!(foo, value.deserialize().unwrap());

  txn.commit().await.unwrap();
}
