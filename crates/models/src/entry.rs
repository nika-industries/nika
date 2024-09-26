use serde::{Deserialize, Serialize};
use slugger::LaxSlug;

use crate::{CacheRecordId, Model, OrgRecordId, RecordId};

/// The [`Entry`] table name.
pub const ENTRY_TABLE_NAME: &str = "entry";

/// An entry record ID.
pub type EntryRecordId = RecordId<Entry>;

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
  const TABLE_NAME: &'static str = ENTRY_TABLE_NAME;
  const UNIQUE_INDICES: &'static [(
    &'static str,
    crate::SlugFieldGetter<Self>,
  )] = &[("cache-id-path", |s| {
    LaxSlug::new(format!("{}-{}", s.cache, s.path)).into()
  })];

  fn id(&self) -> EntryRecordId { self.id }
}

/// The request to create an entry.
#[derive(Clone, Debug)]
pub struct EntryCreateRequest {
  /// The entry's path.
  pub path:  LaxSlug,
  /// The entry's file size
  pub size:  u64,
  /// The entry's cache.
  pub cache: CacheRecordId,
  /// The [`Org`](crate::Org) the store belongs to.
  pub org:   OrgRecordId,
}

impl From<EntryCreateRequest> for Entry {
  fn from(req: EntryCreateRequest) -> Self {
    Self {
      id:    Default::default(),
      path:  req.path,
      size:  req.size,
      cache: req.cache,
      org:   req.org,
    }
  }
}
