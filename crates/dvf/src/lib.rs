//! Data Validation Fundamentals.

pub use slugger;
use slugger::StrictSlug;

/// A nickname for an entity.
#[nutype::nutype(derive(
  Debug,
  Clone,
  Serialize,
  Deserialize,
  PartialEq,
  Eq,
  Hash,
  AsRef,
  Display
))]
pub struct EntityNickname(StrictSlug);

/// A token secret.
#[nutype::nutype(derive(
  Debug,
  Clone,
  Serialize,
  Deserialize,
  PartialEq,
  Eq,
  Hash,
  AsRef,
  Display
))]
pub struct TokenSecret(StrictSlug);
