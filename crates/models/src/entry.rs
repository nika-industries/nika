use std::fmt;

use serde::{Deserialize, Serialize};
use slugger::LaxSlug;

use crate::{cache::CacheRecordId, Model, OrgRecordId};

/// The [`Entry`] table name.
pub const ENTRY_TABLE_NAME: &str = "entry";

/// A [`Entry`] record ID.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntryRecordId(pub ulid::Ulid);

impl From<EntryRecordId> for ulid::Ulid {
  fn from(id: EntryRecordId) -> ulid::Ulid { id.0 }
}

impl fmt::Display for EntryRecordId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

/// An entry.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Entry {
  /// The entry's ID.
  pub id:    EntryRecordId,
  /// The entry's path.
  pub path:  LaxSlug,
  /// The entry's file size
  pub size:  u64,
  /// The entry's cache.
  pub cache: CacheRecordId,
  /// The [`Org`](crate::Org) the store belongs to.
  pub org:   OrgRecordId,
}

impl Model for Entry {
  type Id = EntryRecordId;
  const TABLE_NAME: &'static str = ENTRY_TABLE_NAME;
  const INDICES: &'static [(&'static str, crate::SlugFieldGetter<Self>)] =
    &[("cache-id-path", |s| {
      LaxSlug::new(format!("{}-{}", s.cache, s.path)).into()
    })];

  fn id(&self) -> Self::Id { self.id }
}
