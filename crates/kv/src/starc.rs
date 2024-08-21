//! Provides a smart pointer that can be either static or owned and ref-counted.

use std::{
  borrow::Borrow,
  fmt::{Debug, Display},
  hash::Hash,
  ops::Deref,
  sync::Arc,
};

/// A smart pointer that can be either static or owned and ref-counted.
pub enum Starc<T: ?Sized + 'static> {
  /// A static reference to a value.
  Static(&'static T),
  /// An owned reference-counted value.
  Owned(Arc<T>),
}

impl<T> Starc<T> {
  /// Create a new `Starc` from a static value.
  pub const fn new_static(value: &'static T) -> Self { Self::Static(value) }
  /// Create a new `Starc` from an owned value.
  pub fn new_owned(value: T) -> Self { Self::Owned(Arc::new(value)) }
  /// Create a new `Starc` from a `LazyLock`.
  pub fn new_lazy(value: &'static std::sync::LazyLock<T>) -> Self {
    Self::new_static(value)
  }
}

impl<T: ?Sized> Deref for Starc<T> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &Self::Target {
    match self {
      Starc::Static(v) => v,
      Starc::Owned(v) => v,
    }
  }
}

impl<T: ?Sized> Borrow<T> for Starc<T> {
  #[inline]
  fn borrow(&self) -> &T { self }
}

impl<T: ?Sized> AsRef<T> for Starc<T> {
  #[inline]
  fn as_ref(&self) -> &T { self }
}

impl<T: ?Sized> Clone for Starc<T> {
  #[inline]
  fn clone(&self) -> Self {
    match self {
      Self::Static(value) => Self::Static(value),
      Self::Owned(value) => Self::Owned(value.clone()),
    }
  }
}

impl<T: PartialEq + ?Sized> PartialEq for Starc<T> {
  #[inline]
  fn eq(&self, other: &Self) -> bool { self.deref().eq(other.deref()) }
}

impl<T: PartialEq + ?Sized> Eq for Starc<T> {}

impl<T: Hash + ?Sized> Hash for Starc<T> {
  #[inline]
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.deref().hash(state);
  }
}

impl<T: Debug + ?Sized> Debug for Starc<T> {
  #[inline]
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    Debug::fmt(self.deref(), f)
  }
}

impl<T: Display + ?Sized> Display for Starc<T> {
  #[inline]
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    Display::fmt(self.deref(), f)
  }
}

impl<T: PartialOrd + ?Sized> PartialOrd for Starc<T> {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    self.deref().partial_cmp(other.deref())
  }
}

impl Default for Starc<str> {
  fn default() -> Self { Starc::Static(Default::default()) }
}

impl<T: Ord + ?Sized> Ord for Starc<T> {
  #[inline]
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.deref().cmp(other.deref())
  }
}

impl From<String> for Starc<str> {
  #[inline]
  fn from(value: String) -> Self { Starc::Owned(value.into()) }
}

impl From<slugger::Slug> for Starc<slugger::Slug> {
  #[inline]
  fn from(value: slugger::Slug) -> Self { Starc::Owned(value.into()) }
}

impl<T: ?Sized> From<&'static T> for Starc<T> {
  #[inline]
  fn from(value: &'static T) -> Self { Starc::Static(value) }
}
