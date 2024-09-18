use serde::{Deserialize, Serialize};
use slugger::StrictSlug;

use crate::{Model, RecordId};

/// The [`Org`] table name.
pub const ORG_TABLE_NAME: &str = "org";

/// An org record ID.
pub type OrgRecordId = RecordId<Org>;

/// An org.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Org {
  /// The org's ID.
  pub id:   OrgRecordId,
  /// The org's name.
  pub name: StrictSlug,
}

impl Model for Org {
  const TABLE_NAME: &'static str = ORG_TABLE_NAME;
  const INDICES: &'static [(&'static str, crate::SlugFieldGetter<Self>)] =
    &[("name", |org| org.name.clone().into())];

  fn id(&self) -> OrgRecordId { self.id }
}
