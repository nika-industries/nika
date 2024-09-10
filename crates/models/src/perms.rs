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
  CachePermission {
    /// The store that the permission is for.
    store_id:   StoreRecordId,
    /// The store permission type.
    permission: CachePermissionType,
  },
}

/// The types of permissions that can be granted to a `User` for a `Store`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CachePermissionType {
  /// The user has read access.
  Read,
  /// The user has write access.
  Write,
}

impl Display for CachePermissionType {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      CachePermissionType::Read => write!(f, "read"),
      CachePermissionType::Write => write!(f, "write"),
    }
  }
}
