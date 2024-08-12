use serde::{Deserialize, Serialize};

use crate::{slug::Slug, PermissionSet, UserRecordId};

/// The [`Token`] table name.
pub const TOKEN_TABLE_NAME: &str = "token";

/// A [`Token`] record ID.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ssr", serde(from = "crate::ssr::UlidOrThing"))]
pub struct TokenRecordId(pub ulid::Ulid);

/// A token.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Token {
  /// The token's ID.
  pub id:       TokenRecordId,
  /// The token's nickname.
  pub nickname: Slug,
  /// The token's owner.
  pub org:      UserRecordId,
  /// The token's permissions.
  pub perms:    PermissionSet,
}
