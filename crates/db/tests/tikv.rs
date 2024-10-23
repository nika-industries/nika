use db::DatabaseAdapter;

#[tokio::test]
#[ignore]
async fn test_tikv() {
  let tikv_store = db::kv::tikv::TikvClient::new_from_env().await.unwrap();
  let db = db::KvDatabaseAdapter::new(tikv_store);

  let model = models::Org {
    id:   models::OrgRecordId::new(),
    name: models::EntityName::new(models::StrictSlug::new(format!(
      "org-{}",
      models::Ulid::new().to_string()
    ))),
  };

  db.create_model(model.clone()).await.unwrap();

  let id = model.id;

  let new_model = db
    .fetch_model_by_id::<models::Org>(id)
    .await
    .unwrap()
    .unwrap();

  assert_eq!(model, new_model);

  // fetch by index this time
  let new_model = db
    .fetch_model_by_index::<models::Org>(
      "name".to_string(),
      model.name.clone().into_inner().into(),
    )
    .await
    .unwrap()
    .unwrap();

  assert_eq!(model, new_model);
}
