//! Data Validation Fundamentals.

use std::{path::PathBuf, str::FromStr};

pub use slugger;
use slugger::StrictSlug;

/// A name for an entity.
///
/// This must be fit to be used in a URL. An entity should be able to be
/// referred to by its name.
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
pub struct EntityName(StrictSlug);

/// A nickname for an entity.
///
/// Just like a name, but it has no URL constraint. An entity should not be
/// queried programmatically by users by its nickname.
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

/// A human's name.
#[nutype::nutype(
  derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Hash,
    AsRef,
    Display
  ),
  validate(not_empty, len_char_max = 255)
)]
pub struct HumanName(String);

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

/// A path in temp storage.
#[nutype::nutype(derive(
  Debug,
  Clone,
  Serialize,
  Deserialize,
  PartialEq,
  Eq,
  Hash,
  AsRef,
))]
pub struct TempStoragePath(PathBuf);

impl TempStoragePath {
  /// Creates a new temporary storage path.
  pub fn new_random() -> Self {
    Self::new(PathBuf::from_str(&ulid::Ulid::new().to_string()).unwrap())
  }
}
