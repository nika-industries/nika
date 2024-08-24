//! Key type for use with a store.

use std::{fmt, sync::LazyLock};

use slugger::Slug;
use smallvec::SmallVec;
use starc::Starc;

/// A key for use with a store, consisting of a collection of segments.
///
/// Invariants enforced by this type:
/// - The first segment is always present.
/// - All segments are [`Slug`]s.
///
/// Invariants enforced by the [`Slug`] type:
/// - Consists of only a-z, 0-9, and ‘-’.
/// - Never contains more than one ‘-’ in a row.
/// - Will never start or end with ‘-’.
///
/// [`Key`] implements [`Display`](fmt::Display), where the key is displayed as
/// a string with segments separated by colons.

#[derive(Clone, Debug)]
pub struct Key {
  first_segment: Starc<Slug>,
  segments:      SmallVec<[Starc<Slug>; 6]>,
}

impl Key {
  /// Create a new key with the given segment.
  pub fn new(segment: Starc<Slug>) -> Self {
    Self {
      first_segment: segment,
      segments:      SmallVec::new(),
    }
  }
  /// Create a new key with the given `LazyLock` segment.
  pub fn new_lazy(segment: &'static LazyLock<Slug>) -> Self {
    Self {
      first_segment: Starc::new_lazy(segment),
      segments:      SmallVec::new(),
    }
  }

  /// Add a new segment onto the key with method chaining.
  pub fn with(mut self, segment: Starc<Slug>) -> Self {
    self.push(segment);
    self
  }

  /// Push a new segment onto the key.
  pub fn push(&mut self, segment: Starc<Slug>) { self.segments.push(segment); }

  /// Create a new key by pushing a segment onto the given key.
  pub fn push_new(&self, segment: Starc<Slug>) -> Self {
    let mut new_key = self.clone();
    new_key.push(segment);
    new_key
  }

  /// Get the segment at the given index, if it exists.
  pub fn get(&self, index: usize) -> Option<&Starc<Slug>> {
    match index {
      0 => Some(&self.first_segment),
      i => self.segments.get(i - 1),
    }
  }
}

impl fmt::Display for Key {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.first_segment)?;
    for segment in self.segments.iter() {
      write!(f, ":{}", segment)?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::sync::LazyLock;

  use super::*;

  static A: LazyLock<Slug> = LazyLock::new(|| Slug::new("a"));
  static B: LazyLock<Slug> = LazyLock::new(|| Slug::new("b"));
  static C: LazyLock<Slug> = LazyLock::new(|| Slug::new("c"));

  #[test]
  fn key_display() {
    let key = Key::new(Starc::new_static(&A));
    assert_eq!(key.to_string(), "a");

    let mut key = Key::new(Starc::new_lazy(&A));
    key.push(Starc::new_lazy(&B));
    assert_eq!(key.to_string(), "a:b");

    let mut key = Key::new(Starc::new_lazy(&A));
    key.push(Starc::new_lazy(&B));
    key.push(Starc::new_lazy(&C));
    assert_eq!(key.to_string(), "a:b:c");
  }

  #[test]
  fn key_push() {
    let mut key = Key::new(Starc::new_lazy(&A));
    key.push(Starc::new_lazy(&B));
    assert_eq!(key.to_string(), "a:b");

    key.push(Starc::new_lazy(&C));
    assert_eq!(key.to_string(), "a:b:c");
  }

  #[test]
  fn key_push_new() {
    let key = Key::new(Starc::new_lazy(&A));
    let new_key = key.push_new(Starc::new_lazy(&B));
    assert_eq!(new_key.to_string(), "a:b");

    let new_key = new_key.push_new(Starc::new_lazy(&C));
    assert_eq!(new_key.to_string(), "a:b:c");
  }

  #[test]
  fn key_get() {
    let key = Key::new(Starc::new_lazy(&A));
    assert_eq!(key.get(0), Some(&Starc::new_lazy(&A)));
    assert_eq!(key.get(1), None);

    let mut key = Key::new(Starc::new_lazy(&A));
    key.push(Starc::new_lazy(&B));
    assert_eq!(key.get(0), Some(&Starc::new_lazy(&A)));
    assert_eq!(key.get(1), Some(&Starc::new_lazy(&B)));
    assert_eq!(key.get(2), None);
  }
}
