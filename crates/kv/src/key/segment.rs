use std::{
  fmt,
  hash::{Hash, Hasher},
};

use slugger::{EitherSlug, LaxSlug, StrictSlug};
use starc::Starc;

/// A segment; either a strict or lax slug.
///
/// The variant being used for holding the string data is considered an
/// implementation detail. For [`Hash`], [`PartialOrd`], [`Ord`], [`PartialEq`],
/// and [`Eq`] implementations, this type acts as a transparent container for
/// `&str`. If one segment is a [`StrictSlug`] and the other is a [`LaxSlug`]
/// but they [`.as_ref()`](AsRef::as_ref) to the same `&str`, they are
/// considered equal.
#[derive(Clone, Debug)]
pub enum Segment {
  /// A strict slug.
  Strict(Starc<StrictSlug>),
  /// A lax slug.
  Lax(Starc<LaxSlug>),
}

impl AsRef<str> for Segment {
  fn as_ref(&self) -> &str {
    match self {
      Self::Strict(slug) => slug.as_ref().as_ref(),
      Self::Lax(slug) => slug.as_ref().as_ref(),
    }
  }
}

impl PartialEq for Segment {
  fn eq(&self, other: &Self) -> bool { self.as_ref() == other.as_ref() }
}

impl Eq for Segment {}

impl PartialOrd for Segment {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for Segment {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.as_ref().cmp(other.as_ref())
  }
}

impl Hash for Segment {
  fn hash<H: Hasher>(&self, state: &mut H) { self.as_ref().hash(state); }
}

impl From<EitherSlug> for Segment {
  fn from(slug: EitherSlug) -> Self {
    match slug {
      EitherSlug::Strict(slug) => Self::Strict(Starc::new_owned(slug)),
      EitherSlug::Lax(slug) => Self::Lax(Starc::new_owned(slug)),
    }
  }
}

impl fmt::Display for Segment {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Strict(slug) => write!(f, "{}", slug),
      Self::Lax(slug) => write!(f, "{}", slug),
    }
  }
}
