use serde::{Deserialize, Serialize};
use slugger::Slug;

use crate::{OrgRecordId, PermissionSet, UserRecordId};

/// The [`Token`] table name.
pub const TOKEN_TABLE_NAME: &str = "token";

/// A [`Token`] record ID.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TokenRecordId(pub ulid::Ulid);

/// A token.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Token {
  /// The token's ID.
  pub id:       TokenRecordId,
  /// The token's nickname.
  pub nickname: Slug,
  /// The token's secret.
  pub secret:   Slug,
  /// The token's permissions.
  pub perms:    PermissionSet,
  /// The token's owner.
  pub owner:    UserRecordId,
  /// THe token's org.
  pub org:      OrgRecordId,
}

/// Validates a token secret. Returns `true` if the secret is valid.
pub fn validate_token_secret(input: impl AsRef<str>) -> bool {
  let secret = input.as_ref();
  if secret.len() != 64 {
    return false;
  }
  secret
    .chars()
    .all(|c| c.is_ascii_alphanumeric() || c == '-')
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_valid_secret() {
    let secret =
      "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u1v2w3x4y5z6-7-8-9-abcde";
    assert!(validate_token_secret(secret));
  }

  #[test]
  fn test_invalid_secret_length() {
    let secret = "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u1v2w3x4y5z6-7-8-";
    assert!(!validate_token_secret(secret));
  }

  #[test]
  fn test_invalid_secret_characters() {
    let secret = "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u1v2w3x4y5z6-7-8-@";
    assert!(!validate_token_secret(secret));
  }

  #[test]
  fn test_empty_secret() {
    let secret = "";
    assert!(!validate_token_secret(secret));
  }
}
