use kv::*;

async fn prepare_redis() -> kv::redis::Redis {
  kv::redis::Redis::new(Default::default(), None, None, None)
    .await
    .expect("Failed to create Redis store")
}

#[tokio::test]
async fn test_redis() {
  let store = prepare_redis().await;

  let key = "test_redis";
  let value = "test_redis_value";
  store
    .set(key, value.to_string())
    .await
    .expect("Failed to set value");
  let result = store.get(key).await.expect("Failed to get value");

  assert_eq!(result, Some(value.to_string()));
}
