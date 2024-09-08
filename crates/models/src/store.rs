use std::fmt;

use serde::{Deserialize, Serialize};
use slugger::StrictSlug;

use crate::{Model, OrgRecordId, StorageCredentials};

/// The [`Store`] table name.
pub const STORE_TABLE_NAME: &str = "store";

/// A [`Store`] record ID.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StoreRecordId(pub ulid::Ulid);

impl From<StoreRecordId> for ulid::Ulid {
  fn from(id: StoreRecordId) -> ulid::Ulid { id.0 }
}

impl fmt::Display for StoreRecordId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

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
  type Id = StoreRecordId;
  const TABLE_NAME: &'static str = STORE_TABLE_NAME;
  const INDICES: &'static [(&'static str, crate::SlugFieldGetter<Self>)] =
    &[("name", |s| s.name.clone().into())];

  fn id(&self) -> Self::Id { self.id }
}
