use std::sync::LazyLock;

use kv::{key::Key, KvPrimitive};
use slugger::Slug;
use starc::Starc;

async fn prepare_redis() -> kv::redis::Redis {
  kv::redis::Redis::new(Default::default(), None, None, None)
    .await
    .expect("Failed to create Redis store")
}

static KEY_BASE: LazyLock<Slug> = LazyLock::new(|| Slug::new("kv_test_redis"));

#[tokio::test]
#[ignore]
async fn test_redis_set_and_get() {
  let store = prepare_redis().await;

  let key = Key::new(Starc::new_lazy(&KEY_BASE));
  let value = "test_redis_value";

  store
    .set(&key, value.to_string())
    .await
    .expect("Failed to set value");
  let result = store.get(&key).await.expect("Failed to get value");

  assert_eq!(result, Some(value.to_string()));
}
