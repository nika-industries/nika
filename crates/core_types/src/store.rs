use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub const STORE_TABLE_NAME: &str = "store";

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ssr", serde(from = "crate::ssr::UlidOrThing"))]
pub struct StoreRecordId(pub ulid::Ulid);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Store {
  pub id:     StoreRecordId,
  pub config: StorageCredentials,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum StorageCredentials {
  Local(LocalStorageCredentials),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LocalStorageCredentials(pub PathBuf);
