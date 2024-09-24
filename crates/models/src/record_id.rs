use std::{fmt, hash::Hash, marker::PhantomData, str::FromStr};

use serde::{Deserialize, Serialize};
pub use ulid::Ulid;

// Generally we have to implement these traits manually that we'd normally
// derive because of the `PhantomData` field; the derives assume that the `T`
// generic also has to implement the trait we're deriving.

/// A record ID for a [`Model`](crate::Model) implementer.
#[derive(Serialize, Deserialize)]
pub struct RecordId<T> {
  id: Ulid,
  #[serde(skip)]
  _m: PhantomData<T>,
}

impl<T> fmt::Debug for RecordId<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_tuple("RecordId").field(&self.id).finish()
  }
}

impl<T> fmt::Display for RecordId<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.id)
  }
}

impl<T> Clone for RecordId<T> {
  fn clone(&self) -> Self { *self }
}

impl<T> Copy for RecordId<T> {}

impl<T> PartialEq<Self> for RecordId<T> {
  fn eq(&self, other: &Self) -> bool { self.id.eq(&other.id) }
}

impl<T> Eq for RecordId<T> {}

impl<T> Hash for RecordId<T> {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.id.hash(state) }
}

impl<T> PartialOrd for RecordId<T> {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl<T> Ord for RecordId<T> {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering { self.id.cmp(&other.id) }
}

impl<T> RecordId<T> {
  /// Creates a new [`RecordId`].
  pub fn new() -> Self {
    Self {
      id: Ulid::new(),
      _m: PhantomData,
    }
  }
  /// Creates a new [`RecordId`] from a [`Ulid`].
  pub fn from_ulid(ulid: Ulid) -> Self {
    Self {
      id: ulid,
      _m: PhantomData,
    }
  }
}

impl<T> FromStr for RecordId<T> {
  type Err = ulid::DecodeError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self {
      id: Ulid::from_str(s)?,
      _m: PhantomData,
    })
  }
}

impl<T> From<RecordId<T>> for Ulid {
  fn from(id: RecordId<T>) -> Ulid { id.id }
}

impl<T> Default for RecordId<T> {
  fn default() -> Self { Self::new() }
}
