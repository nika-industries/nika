use serde::{Deserialize, Serialize};

use crate::{Model, Org, RecordId};

/// The [`User`] table name.
pub const USER_TABLE_NAME: &str = "user";

/// A user record ID.
pub type UserRecordId = RecordId<User>;

/// A user.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
  /// The user's ID.
  pub id:   RecordId<User>,
  /// The user's name.
  pub name: String,
  /// The user's org.
  pub org:  RecordId<Org>,
}

impl Model for User {
  const TABLE_NAME: &'static str = USER_TABLE_NAME;
  const INDICES: &'static [(&'static str, crate::SlugFieldGetter<Self>)] = &[];

  fn id(&self) -> RecordId<User> { self.id }
}
