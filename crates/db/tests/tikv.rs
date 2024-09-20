use db::{DatabaseAdapter, TikvAdapter};

#[tokio::test]
#[ignore]
async fn test_tikv() {
  let db = TikvAdapter::new_from_env().await.unwrap();

  let model = models::Org {
    id:   models::OrgRecordId::new(),
    name: models::StrictSlug::new(format!(
      "org-{}",
      models::Ulid::new().to_string()
    )),
  };

  db.create_model(&model).await.unwrap();

  let id = model.id;

  let new_model = db
    .fetch_model_by_id::<models::Org>(&id)
    .await
    .unwrap()
    .unwrap();

  assert_eq!(model, new_model);

  // fetch by index this time
  let new_model = db
    .fetch_model_by_index::<models::Org>("name", &model.name.clone().into())
    .await
    .unwrap()
    .unwrap();

  assert_eq!(model, new_model);
}
