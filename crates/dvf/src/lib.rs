//! Data Validation Fundamentals.

use std::{fmt, path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};
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

/// An entity's visibility.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Visibility {
  /// The entity is public.
  Public,
  /// The entity is private.
  Private,
}

impl Visibility {
  /// Returns `true` if the entity is public.
  pub fn is_public(&self) -> bool { matches!(self, Visibility::Public) }
}

impl fmt::Display for Visibility {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Visibility::Public => write!(f, "Public"),
      Visibility::Private => write!(f, "Private"),
    }
  }
}

/// The size of a file.
#[nutype::nutype(derive(
  Clone,
  Serialize,
  Deserialize,
  PartialEq,
  Eq,
  Hash,
  AsRef,
))]
pub struct FileSize(u64);

impl fmt::Display for FileSize {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let size = *self.as_ref();
    if size < 1024 {
      write!(f, "{} B", size)
    } else if size < 1024 * 1024 {
      write!(f, "{:.2} KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
      write!(f, "{:.2} MB", size as f64 / const { 1024.0 * 1024.0 })
    } else {
      write!(
        f,
        "{:.2} GB",
        size as f64 / const { 1024.0 * 1024.0 * 1024.0 }
      )
    }
  }
}

impl fmt::Debug for FileSize {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_tuple("FileSize").field(self.as_ref()).finish()
  }
}
