use serde::{Deserialize, Serialize};
use slugger::Slug;

use crate::{OrgRecordId, StorageCredentials};

/// The [`Store`] table name.
pub const STORE_TABLE_NAME: &str = "store";

/// A [`Store`] record ID.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ssr", serde(from = "crate::ssr::UlidOrThing"))]
pub struct StoreRecordId(pub ulid::Ulid);

/// A store.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Store {
  /// The store's ID.
  pub id:       StoreRecordId,
  /// The store's nickname.
  pub nickname: Slug,
  /// The store's credentials.
  pub config:   StorageCredentials,
  /// Whether the store is public.
  pub public:   bool,
  /// The [`Org`](crate::Org) the store belongs to.
  pub org:      OrgRecordId,
}
