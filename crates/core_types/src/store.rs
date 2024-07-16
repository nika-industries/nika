use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
pub use self::ssr::*;

pub const STORE_TABLE_NAME: &str = "store";

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ssr", serde(from = "crate::ssr::UlidOrThing"))]
pub struct StoreRecordId(pub ulid::Ulid);

#[cfg(feature = "ssr")]
mod ssr {
  use std::path::PathBuf;

  use serde::{Deserialize, Serialize};

  use super::StoreRecordId;

  #[derive(Clone, Debug, Serialize, Deserialize)]
  pub struct Store {
    pub id:     StoreRecordId,
    pub config: StorageCredentials,
  }

  #[derive(Serialize, Deserialize, Clone, Debug)]
  pub enum StorageCredentials {
    Local(LocalStorageCredentials),
    R2(R2StorageCredentials),
  }

  #[derive(Serialize, Deserialize, Clone, Debug)]
  pub struct LocalStorageCredentials(pub PathBuf);

  #[derive(Serialize, Deserialize, Clone, Debug)]
  pub struct R2StorageCredentials {}
}
