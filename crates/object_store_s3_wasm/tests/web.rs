use std::sync::Arc;

use futures::TryStreamExt;
use object_store::ObjectStore;
use object_store_s3_wasm::S3;
use wasm_bindgen_test::{wasm_bindgen_test_configure, *};

wasm_bindgen_test_configure!(run_in_browser);

// docker run \
// -p 9000:9000 \
// -p 9090:9090 \
// --name minio1 \
// -e "MINIO_ROOT_USER=root" \
// -e "MINIO_ROOT_PASSWORD=password" \
// quay.io/minio/minio server /data --console-address ":9090"

#[wasm_bindgen_test]
async fn it_works() {
  let s3: Arc<dyn ObjectStore> = Arc::new(
    S3::builder()
      .endpoint("http://localhost:9000")
      .region("us-east-1")
      .bucket("test")
      .access_key_id("UYCQnNlCugeb1BmZtauK")
      .secret_access_key("wiAL4vRJs7cshy6fCpHDEQrhx8oWefk4UNyMpM6V")
      .build()
      .expect("Failed to create s3 client"),
  );

  let test = "Wasm rocks".to_owned();

  s3.put(&"folder/wasm.txt".into(), test.into())
    .await
    .expect("Failed to upload bytes");

  let objects = s3
    .list(None)
    .try_collect::<Vec<_>>()
    .await
    .expect("Failed to list objects");

  assert_eq!(objects[0].location.to_string(), "folder/wasm.txt");

  let content = s3
    .get(&"folder/wasm.txt".into())
    .await
    .expect("Failed to get file content.");

  assert_eq!(
    String::from_utf8(
      content
        .bytes()
        .await
        .expect("Failed to get file content.")
        .into()
    )
    .expect("Failed to convert bytes to utf8"),
    "Wasm rocks"
  );

  s3.delete(&"folder/wasm.txt".into())
    .await
    .expect("Failed to delte object");
}
