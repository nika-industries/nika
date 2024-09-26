use std::{fmt, hash::Hash, marker::PhantomData, str::FromStr};

use serde::{Deserialize, Serialize};
pub use ulid::Ulid;

// Generally we have to implement these traits manually that we'd normally
// derive because of the `PhantomData` field; the derives assume that the `T`
// generic also has to implement the trait we're deriving.

/// A record ID for a [`Model`](crate::Model) implementer.
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct RecordId<T>(Ulid, #[serde(skip)] PhantomData<T>);

impl<T> fmt::Debug for RecordId<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_tuple("RecordId").field(&self.0).finish()
  }
}

impl<T> fmt::Display for RecordId<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl<T> Clone for RecordId<T> {
  fn clone(&self) -> Self { *self }
}

impl<T> Copy for RecordId<T> {}

impl<T> PartialEq<Self> for RecordId<T> {
  fn eq(&self, other: &Self) -> bool { self.0.eq(&other.0) }
}

impl<T> Eq for RecordId<T> {}

impl<T> Hash for RecordId<T> {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.0.hash(state) }
}

impl<T> PartialOrd for RecordId<T> {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl<T> Ord for RecordId<T> {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering { self.0.cmp(&other.0) }
}

impl<T> TryFrom<String> for RecordId<T> {
  type Error = ulid::DecodeError;
  fn try_from(value: String) -> Result<Self, Self::Error> {
    Ulid::from_str(&value).map(Self::from_ulid)
  }
}

impl<T> RecordId<T> {
  /// Creates a new [`RecordId`].
  pub fn new() -> Self { Self(Ulid::new(), PhantomData) }
  /// Creates a new [`RecordId`] from a [`Ulid`].
  pub fn from_ulid(ulid: Ulid) -> Self { Self(ulid, PhantomData) }
}

impl<T> FromStr for RecordId<T> {
  type Err = ulid::DecodeError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self(Ulid::from_str(s)?, PhantomData))
  }
}

impl<T> From<RecordId<T>> for Ulid {
  fn from(id: RecordId<T>) -> Ulid { id.0 }
}

impl<T> Default for RecordId<T> {
  fn default() -> Self { Self::new() }
}
