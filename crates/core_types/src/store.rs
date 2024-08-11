use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
pub use self::ssr::*;

/// The [`Store`] table name.
pub const STORE_TABLE_NAME: &str = "store";

/// A [`Store`] record ID.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ssr", serde(from = "crate::ssr::UlidOrThing"))]
pub struct StoreRecordId(pub ulid::Ulid);

#[cfg(feature = "ssr")]
mod ssr {
  use serde::{Deserialize, Serialize};

  use super::StoreRecordId;
  use crate::storage_creds::StorageCredentials;

  /// A store.
  #[derive(Clone, Debug, Serialize, Deserialize)]
  pub struct Store {
    /// The store's ID.
    pub id:     StoreRecordId,
    /// The store's credentials.
    pub config: StorageCredentials,
  }
}
