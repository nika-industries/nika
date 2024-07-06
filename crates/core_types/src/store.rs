use serde::{Deserialize, Serialize};

pub const STORE_TABLE_NAME: &str = "store";

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ssr", serde(from = "crate::ssr::UlidOrThing"))]
pub struct StoreRecordId(pub ulid::Ulid);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Store {
  pub id: StoreRecordId,
}
