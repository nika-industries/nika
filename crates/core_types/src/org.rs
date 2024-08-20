use serde::{Deserialize, Serialize};
use slugger::Slug;

/// The [`Org`] table name.
pub const ORG_TABLE_NAME: &str = "org";

/// An [`Org`] record ID.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ssr", serde(from = "crate::ssr::UlidOrThing"))]
pub struct OrgRecordId(pub ulid::Ulid);

/// An org.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Org {
  /// The org's ID.
  pub id:   OrgRecordId,
  /// The org's name.
  pub name: Slug,
}
