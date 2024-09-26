use serde::{Deserialize, Serialize};
use slugger::StrictSlug;

use crate::{Model, OrgRecordId, RecordId, StoreRecordId};

/// The [`Cache`] table name.
pub const CACHE_TABLE_NAME: &str = "cache";

/// A cache record ID.
pub type CacheRecordId = RecordId<Cache>;

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
  const TABLE_NAME: &'static str = CACHE_TABLE_NAME;
  const UNIQUE_INDICES: &'static [(
    &'static str,
    crate::SlugFieldGetter<Self>,
  )] = &[("name", |s| s.name.clone().into())];

  fn id(&self) -> CacheRecordId { self.id }
}

/// The request to create a cache.
#[derive(Clone, Debug)]
pub struct CacheCreateRequest {
  /// The cache's nickname.
  pub name:   StrictSlug,
  /// Whether the store is public.
  pub public: bool,
  /// The cache's backing store.
  pub store:  StoreRecordId,
  /// The [`Org`](crate::Org) the store belongs to.
  pub org:    OrgRecordId,
}

impl From<CacheCreateRequest> for Cache {
  fn from(req: CacheCreateRequest) -> Self {
    Self {
      id:     Default::default(),
      name:   req.name,
      public: req.public,
      store:  req.store,
      org:    req.org,
    }
  }
}
