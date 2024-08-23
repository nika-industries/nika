use std::sync::LazyLock;

use kv::{
  key::Key, tikv::TikvClient, value::Value, KvPrimitive, KvTransaction,
  KvTransactional,
};
use slugger::Slug;

static GLOBAL_NS_SEGMENT: LazyLock<Slug> =
  LazyLock::new(|| Slug::new("nika".to_string()));

#[tokio::test]
#[ignore]
async fn test_tikv() {
  let client = TikvClient::new(vec!["localhost:2379"]).await.unwrap();
  let mut txn = client.begin_pessimistic_transaction().await.unwrap();

  let key = Key::new_lazy(&GLOBAL_NS_SEGMENT);
  let value: Value = "world".to_string().into_bytes().into();

  txn.put(&key, value.clone()).await.unwrap();
  assert_eq!(txn.get(&key).await.unwrap().unwrap(), value);

  txn.commit().await.unwrap();
}
