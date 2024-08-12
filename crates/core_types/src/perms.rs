use std::collections::HashSet;

use serde::{Deserialize, Serialize};

/// A permission set for a `Store`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PermissionSet(pub HashSet<(ulid::Ulid, StorePermission)>);

/// User permissions for `Store`s.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum StorePermission {
  /// The user has read access.
  Read,
  /// The user has write access.
  Write,
}
