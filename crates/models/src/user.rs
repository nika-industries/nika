use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{Model, OrgRecordId};

/// The [`User`] table name.
pub const USER_TABLE_NAME: &str = "user";

/// A [`User`] record ID.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserRecordId(pub ulid::Ulid);

impl From<UserRecordId> for ulid::Ulid {
  fn from(id: UserRecordId) -> ulid::Ulid { id.0 }
}

impl fmt::Display for UserRecordId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

/// A user.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
  /// The user's ID.
  pub id:   UserRecordId,
  /// The user's name.
  pub name: String,
  /// The user's org.
  pub org:  OrgRecordId,
}

impl Model for User {
  type Id = UserRecordId;
  const TABLE_NAME: &'static str = USER_TABLE_NAME;
  const INDICES: &'static [(&'static str, crate::SlugFieldGetter<Self>)] = &[];

  fn id(&self) -> Self::Id { self.id }
}
