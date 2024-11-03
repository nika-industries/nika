use std::{path::PathBuf, str::FromStr};

use miette::Result;
use models::{
  CachePermissionType, CacheRecordId, EntityName, EntityNickname, HumanName,
  LocalStorageCredentials, Org, Permission, PermissionSet, RecordId,
  StorageCredentials, StoreRecordId, StrictSlug, TokenRecordId, TokenSecret,
  UserRecordId,
};

use crate::DatabaseAdapter;

/// A trait for migrating the database.
pub trait Migratable {
  /// Migrates the database.
  fn migrate(&self) -> impl std::future::Future<Output = Result<()>> + Send;
}

impl<T: DatabaseAdapter> Migratable for T {
  /// Applies test data to the database.
  async fn migrate(&self) -> Result<()> {
    let org = Org {
      id:   RecordId::<Org>::from_str("01J53FHN8TQXTQ2JEHNX56GCTN").unwrap(),
      name: EntityName::new(StrictSlug::confident("dev-org")),
    };

    let user = models::User {
      id:   UserRecordId::from_str("01J53N6ARQGFTBQ41T25TAJ949").unwrap(),
      name: HumanName::try_new("John Lewis".to_string()).unwrap(),
      org:  org.id,
    };

    let local_file_store = models::Store {
      id:                 StoreRecordId::from_str("01J53YYCCJW4B4QBM1CG0CHAMP")
        .unwrap(),
      nickname:           EntityNickname::new(StrictSlug::confident(
        "local-file-store",
      )),
      credentials:        StorageCredentials::Local(LocalStorageCredentials(
        PathBuf::from_str("/tmp/local-store").unwrap(),
      )),
      compression_config: models::CompressionConfig::new(None),
      org:                org.id,
    };

    let albert_cache = models::Cache {
      id:         CacheRecordId::from_str("01J799MSHXPPY5RJ8KGHVR9GWQ")
        .unwrap(),
      name:       EntityName::new(StrictSlug::confident("albert")),
      visibility: models::Visibility::Private,
      store:      local_file_store.id,
      org:        org.id,
    };

    let omnitoken_token = models::Token {
      id:       TokenRecordId::from_str("01J53ZA38PS1P5KWCE4FMG58F0").unwrap(),
      nickname: EntityNickname::new(StrictSlug::confident("omnitoken")),
      secret:   TokenSecret::new(StrictSlug::confident(
        "zvka5d29dgvpujdyqa6ftnkei02i-qm1n-fjzuqfbyrq7avxbzi6ma8flxsuwe4l",
      )),
      perms:    PermissionSet(
        vec![
          Permission::CachePermission {
            cache_id:   albert_cache.id,
            permission: CachePermissionType::Read,
          },
          Permission::CachePermission {
            cache_id:   albert_cache.id,
            permission: CachePermissionType::Write,
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
