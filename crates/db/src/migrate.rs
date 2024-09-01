use std::str::FromStr;

use kv::prelude::*;
use miette::Result;

use crate::DbConnection;

impl<T: KvTransactional> DbConnection<T> {
  /// Applies test data to the database.
  pub async fn migrate(&self) -> Result<()> {
    let org = models::Org {
      id:   models::OrgRecordId(
        models::Ulid::from_str("01J53FHN8TQXTQ2JEHNX56GCTN").unwrap(),
      ),
      name: models::StrictSlug::confident("dev-org"),
    };

    let user = models::User {
      id:   models::UserRecordId(
        models::Ulid::from_str("01J53N6ARQGFTBQ41T25TAJ949").unwrap(),
      ),
      name: "John Lewis".to_string(),
      org:  org.id,
    };

    let albert_store = models::Store {
      id:     models::StoreRecordId(
        models::Ulid::from_str("01J53YYCCJW4B4QBM1CG0CHAMP").unwrap(),
      ),
      config: models::StorageCredentials::Local(
        models::LocalStorageCredentials(
          std::path::PathBuf::from_str("/tmp/albert-store").unwrap(),
        ),
      ),
      name:   models::StrictSlug::confident("albert"),
      public: false,
      org:    org.id,
    };

    let omnitoken_token = models::Token {
      id:       models::TokenRecordId(
        models::Ulid::from_str("01J53ZA38PS1P5KWCE4FMG58F0").unwrap(),
      ),
      nickname: models::StrictSlug::confident("omnitoken"),
      secret:   models::StrictSlug::confident(
        "zvka5d29dgvpujdyqa6ftnkei02i-qm1n-fjzuqfbyrq7avxbzi6ma8flxsuwe4l",
      ),
      perms:    models::PermissionSet(
        vec![
          models::Permission::StorePermission {
            store_id:   albert_store.id,
            permission: models::StorePermissionType::Read,
          },
          models::Permission::StorePermission {
            store_id:   albert_store.id,
            permission: models::StorePermissionType::Write,
          },
        ]
        .into_iter()
        .collect(),
      ),
      owner:    user.id,
      org:      org.id,
    };

    self.create_model(&org).await?;
    self.create_model(&user).await?;
    self.create_model(&albert_store).await?;
    self.create_model(&omnitoken_token).await?;

    Ok(())
  }
}
