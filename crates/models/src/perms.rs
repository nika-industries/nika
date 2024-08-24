use std::{
  collections::HashSet,
  fmt::{self, Display, Formatter},
};

use serde::{Deserialize, Serialize};

use crate::StoreRecordId;

/// A permission set for a `Store`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PermissionSet(pub HashSet<Permission>);

/// A permission.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Permission {
  /// A store permission.
  StorePermission {
    /// The store that the permission is for.
    store_id:   StoreRecordId,
    /// The store permission type.
    permission: StorePermissionType,
  },
}

/// The types of permissions that can be granted to a `User` for a `Store`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum StorePermissionType {
  /// The user has read access.
  Read,
  /// The user has write access.
  Write,
}

impl Display for StorePermissionType {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      StorePermissionType::Read => write!(f, "read"),
      StorePermissionType::Write => write!(f, "write"),
    }
  }
}
