use serde::{Deserialize, Serialize};
use slugger::StrictSlug;

use crate::{Model, OrgRecordId, RecordId, StorageCredentials};

/// The [`Store`] table name.
pub const STORE_TABLE_NAME: &str = "store";

/// A store record ID.
pub type StoreRecordId = RecordId<Store>;

/// A store.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Store {
  /// The store's ID.
  pub id:     StoreRecordId,
  /// The store's nickname.
  pub name:   StrictSlug,
  /// The store's credentials.
  pub config: StorageCredentials,
  /// Whether the store is public.
  pub public: bool,
  /// The [`Org`](crate::Org) the store belongs to.
  pub org:    OrgRecordId,
}

impl Model for Store {
  const TABLE_NAME: &'static str = STORE_TABLE_NAME;
  const INDICES: &'static [(&'static str, crate::SlugFieldGetter<Self>)] =
    &[("name", |s| s.name.clone().into())];

  fn id(&self) -> StoreRecordId { self.id }
}
