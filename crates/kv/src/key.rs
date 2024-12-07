//! Key type for use with a store.

mod segment;

use std::{fmt, sync::LazyLock};

use slugger::StrictSlug;
use smallvec::SmallVec;
use starc::Starc;

pub use self::segment::Segment;

/// A key for use with a store, consisting of a collection of segments.
///
/// [`Key`] implements [`Display`](fmt::Display), where the key is displayed as
/// a string with segments separated by colons.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]

pub struct Key {
  first_segment: Segment,
  segments:      SmallVec<[Segment; 6]>,
}

use std::hash::{Hash, Hasher};

impl Hash for Key {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.first_segment.hash(state);
    for segment in self.segments.iter() {
      segment.hash(state);
    }
  }
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

  /// Add a new segment that's either strict or lax.
  pub fn with_either(mut self, segment: impl Into<Segment>) -> Self {
    self.segments.push(segment.into());
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

impl TryFrom<Vec<u8>> for Key {
  type Error = std::str::Utf8Error;

  fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
    let string = std::str::from_utf8(&bytes)?;
    let mut segments = string.split(':').map(|s| s.to_string());
    let first_segment = segments.next().unwrap();
    let first_segment = Starc::new_owned(StrictSlug::new(first_segment));
    let mut key = Key::new(first_segment);
    for segment in segments {
      key.push(Starc::new_owned(StrictSlug::new(segment)));
    }
    Ok(key)
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
