use serde::{Deserialize, Serialize};

use crate::OrgRecordId;

/// The [`User`] table name.
pub const USER_TABLE_NAME: &str = "user";

/// A [`User`] record ID.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ssr", serde(from = "crate::ssr::UlidOrThing"))]
pub struct UserRecordId(pub ulid::Ulid);

/// A user.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
  /// The user's ID.
  pub id:   UserRecordId,
  /// The user's name.
  pub name: String,
  /// The user's org.
  pub org:  OrgRecordId,
}
