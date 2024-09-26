use serde::{Deserialize, Serialize};
use slugger::StrictSlug;

use crate::{Model, OrgRecordId, PermissionSet, RecordId, UserRecordId};

/// The [`Token`] table name.
pub const TOKEN_TABLE_NAME: &str = "token";

/// A token record ID.
pub type TokenRecordId = RecordId<Token>;

/// A token.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Token {
  /// The token's ID.
  pub id:       TokenRecordId,
  /// The token's nickname.
  pub nickname: StrictSlug,
  /// The token's secret.
  pub secret:   StrictSlug,
  /// The token's permissions.
  pub perms:    PermissionSet,
  /// The token's owner (a User).
  pub owner:    UserRecordId,
  /// THe token's org.
  pub org:      OrgRecordId,
}

impl Model for Token {
  const TABLE_NAME: &'static str = TOKEN_TABLE_NAME;
  const UNIQUE_INDICES: &'static [(
    &'static str,
    crate::SlugFieldGetter<Self>,
  )] = &[("secret", |t| t.secret.clone().into())];

  fn id(&self) -> TokenRecordId { self.id }
}

impl Token {
  /// Check if the token has the given permissions.
  pub fn authorized(&self, perms: &PermissionSet) -> bool {
    self.perms.contains_set(perms)
  }
}

/// A token create request.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TokenCreateRequest {
  /// The token's nickname.
  pub nickname: StrictSlug,
  /// The token's secret.
  pub secret:   StrictSlug,
  /// The token's permissions.
  pub perms:    PermissionSet,
  /// The token's owner (a User).
  pub owner:    UserRecordId,
  /// The token's org.
  pub org:      OrgRecordId,
}

impl From<TokenCreateRequest> for Token {
  fn from(input: TokenCreateRequest) -> Self {
    Self {
      id:       TokenRecordId::default(),
      nickname: input.nickname,
      secret:   input.secret,
      perms:    input.perms,
      owner:    input.owner,
      org:      input.org,
    }
  }
}
