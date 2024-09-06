use std::fmt;

use serde::{Deserialize, Serialize};
use slugger::StrictSlug;

use crate::Model;

/// The [`Org`] table name.
pub const ORG_TABLE_NAME: &str = "org";

/// An [`Org`] record ID.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrgRecordId(pub ulid::Ulid);

impl From<OrgRecordId> for ulid::Ulid {
  fn from(id: OrgRecordId) -> ulid::Ulid { id.0 }
}

impl fmt::Display for OrgRecordId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}
/// An org.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Org {
  /// The org's ID.
  pub id:   OrgRecordId,
  /// The org's name.
  pub name: StrictSlug,
}

impl Model for Org {
  type Id = OrgRecordId;
  const TABLE_NAME: &'static str = ORG_TABLE_NAME;
  const INDICES: &'static [(&'static str, crate::SlugFieldGetter<Self>)] =
    &[("name", |org| org.name.clone())];

  fn id(&self) -> Self::Id { self.id }
}
