//! Key type for use with a store.

use std::{fmt, sync::LazyLock};

use slugger::{LaxSlug, StrictSlug};
use smallvec::SmallVec;
use starc::Starc;

/// A segment; either a strict or lax slug.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Segment {
  /// A strict slug.
  Strict(Starc<StrictSlug>),
  /// A lax slug.
  Lax(Starc<LaxSlug>),
}

impl fmt::Display for Segment {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Strict(slug) => write!(f, "{}", slug),
      Self::Lax(slug) => write!(f, "{}", slug),
    }
  }
}

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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Key {
  first_segment: Segment,
  segments:      SmallVec<[Segment; 6]>,
}

impl Key {
  /// Create a new key with the given segment.
  pub fn new(segment: impl Into<Starc<StrictSlug>>) -> Self {
    Self {
      first_segment: Segment::Strict(segment.into()),
      segments:      SmallVec::new(),
    }
  }
  /// Create a new key with the given `LazyLock` segment.
  pub fn new_lazy(segment: &'static LazyLock<StrictSlug>) -> Self {
    Self {
      first_segment: Segment::Strict(Starc::new_lazy(segment)),
      segments:      SmallVec::new(),
    }
  }

  /// Add a new segment onto the key with method chaining.
  pub fn with(mut self, segment: impl Into<Starc<StrictSlug>>) -> Self {
    self.push(segment.into());
    self
  }

  /// Push a new segment onto the key.
  pub fn push(&mut self, segment: impl Into<Starc<StrictSlug>>) {
    self.segments.push(Segment::Strict(segment.into()));
  }

  /// Create a new key by pushing a segment onto the given key.
  pub fn push_new(&self, segment: impl Into<Starc<StrictSlug>>) -> Self {
    let mut new_key = self.clone();
    new_key.push(segment);
    new_key
  }

  /// Get the segment at the given index, if it exists.
  pub fn get(&self, index: usize) -> Option<&Segment> {
    match index {
      0 => Some(&self.first_segment),
      i => self.segments.get(i - 1),
    }
  }

  /// Get an iterator over the segments of the key.
  pub fn segments(&self) -> impl Iterator<Item = &Segment> {
    std::iter::once(&self.first_segment).chain(self.segments.iter())
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

  static A: LazyLock<StrictSlug> = LazyLock::new(|| StrictSlug::new("a"));
  static B: LazyLock<StrictSlug> = LazyLock::new(|| StrictSlug::new("b"));
  static C: LazyLock<StrictSlug> = LazyLock::new(|| StrictSlug::new("c"));

  #[test]
  fn key_display() {
    let key = Key::new(&A);
    assert_eq!(key.to_string(), "a");

    let mut key = Key::new(&A);
    key.push(&B);
    assert_eq!(key.to_string(), "a:b");

    let mut key = Key::new(&A);
    key.push(&B);
    key.push(&C);
    assert_eq!(key.to_string(), "a:b:c");
  }

  #[test]
  fn key_push() {
    let mut key = Key::new(&A);
    key.push(&B);
    assert_eq!(key.to_string(), "a:b");

    key.push(&C);
    assert_eq!(key.to_string(), "a:b:c");
  }

  #[test]
  fn key_push_new() {
    let key = Key::new(&A);
    let new_key = key.push_new(&B);
    assert_eq!(new_key.to_string(), "a:b");

    let new_key = new_key.push_new(&C);
    assert_eq!(new_key.to_string(), "a:b:c");
  }

  #[test]
  fn key_get() {
    let key = Key::new(&A);
    assert_eq!(key.get(0), Some(&Segment::Strict(Starc::new_lazy(&A))));
    assert_eq!(key.get(1), None);

    let mut key = Key::new(&A);
    key.push(&B);
    assert_eq!(key.get(0), Some(&Segment::Strict(Starc::new_lazy(&A))));
    assert_eq!(key.get(1), Some(&Segment::Strict(Starc::new_lazy(&B))));
    assert_eq!(key.get(2), None);
  }
}
