use std::{
  collections::HashSet,
  fmt::{self, Display, Formatter},
};

use serde::{Deserialize, Serialize};

use crate::StoreRecordId;

/// A permission set for a `Store`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PermissionSet(pub HashSet<(StoreRecordId, StorePermission)>);

/// User permissions for `Store`s.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum StorePermission {
  /// The user has read access.
  Read,
  /// The user has write access.
  Write,
}

impl Display for StorePermission {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      StorePermission::Read => write!(f, "read"),
      StorePermission::Write => write!(f, "write"),
    }
  }
}
