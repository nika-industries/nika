use std::str::FromStr;

use miette::Result;

use crate::DatabaseAdapter;

/// A trait for migrating the database.
pub trait Migratable {
  /// Migrates the database.
  fn migrate(&self) -> impl std::future::Future<Output = Result<()>> + Send;
}

impl<T: DatabaseAdapter> Migratable for T {
  /// Applies test data to the database.
  async fn migrate(&self) -> Result<()> {
    let org = models::Org {
      id:   models::RecordId::<models::Org>::from_str(
        "01J53FHN8TQXTQ2JEHNX56GCTN",
      )
      .unwrap(),
      name: models::StrictSlug::confident("dev-org"),
    };

    let user = models::User {
      id:   models::UserRecordId::from_str("01J53N6ARQGFTBQ41T25TAJ949")
        .unwrap(),
      name: "John Lewis".to_string(),
      org:  org.id,
    };

    let local_file_store = models::Store {
      id:     models::StoreRecordId::from_str("01J53YYCCJW4B4QBM1CG0CHAMP")
        .unwrap(),
      config: models::StorageCredentials::Local(
        models::LocalStorageCredentials(
          std::path::PathBuf::from_str("/tmp/local-store").unwrap(),
        ),
      ),
      name:   models::StrictSlug::confident("local-file-store"),
      public: false,
      org:    org.id,
    };

    let albert_cache = models::Cache {
      id:     models::CacheRecordId::from_str("01J799MSHXPPY5RJ8KGHVR9GWQ")
        .unwrap(),
      name:   models::StrictSlug::confident("albert"),
      public: false,
      store:  local_file_store.id,
      org:    org.id,
    };

    let omnitoken_token = models::Token {
      id:       models::TokenRecordId::from_str("01J53ZA38PS1P5KWCE4FMG58F0")
        .unwrap(),
      nickname: models::StrictSlug::confident("omnitoken"),
      secret:   models::StrictSlug::confident(
        "zvka5d29dgvpujdyqa6ftnkei02i-qm1n-fjzuqfbyrq7avxbzi6ma8flxsuwe4l",
      ),
      perms:    models::PermissionSet(
        vec![
          models::Permission::CachePermission {
            store_id:   local_file_store.id,
            permission: models::CachePermissionType::Read,
          },
          models::Permission::CachePermission {
            store_id:   local_file_store.id,
            permission: models::CachePermissionType::Write,
          },
        ]
        .into_iter()
        .collect(),
      ),
      owner:    user.id,
      org:      org.id,
    };

    self.create_model(org).await?;
    self.create_model(user).await?;
    self.create_model(local_file_store).await?;
    self.create_model(albert_cache).await?;
    self.create_model(omnitoken_token).await?;

    Ok(())
  }
}
