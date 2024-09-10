use std::fmt;

use serde::{Deserialize, Serialize};
use slugger::StrictSlug;

use crate::{Model, OrgRecordId, StoreRecordId};

/// The [`Cache`] table name.
pub const CACHE_TABLE_NAME: &str = "cache";

/// A [`Cache`] record ID.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CacheRecordId(pub ulid::Ulid);

impl From<CacheRecordId> for ulid::Ulid {
  fn from(id: CacheRecordId) -> ulid::Ulid { id.0 }
}

impl fmt::Display for CacheRecordId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

/// A cache.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cache {
  /// The cache's ID.
  pub id:     CacheRecordId,
  /// The cache's nickname.
  pub name:   StrictSlug,
  /// Whether the store is public.
  pub public: bool,
  /// The cache's backing store.
  pub store:  StoreRecordId,
  /// The [`Org`](crate::Org) the store belongs to.
  pub org:    OrgRecordId,
}

impl Model for Cache {
  type Id = CacheRecordId;
  const TABLE_NAME: &'static str = CACHE_TABLE_NAME;
  const INDICES: &'static [(&'static str, crate::SlugFieldGetter<Self>)] =
    &[("name", |s| s.name.clone().into())];

  fn id(&self) -> Self::Id { self.id }
}
