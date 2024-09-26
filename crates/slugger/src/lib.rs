//! Provides a type-safe slug.

pub mod lax;
pub mod strict;

use std::fmt;

use serde::{Deserialize, Serialize};

use self::{lax::lax_slugify, strict::strict_slugify};

/// A strict slug, meant for user-supplied data used in URLs and entity names.
#[nutype::nutype(
  sanitize(with = |s: String| strict_slugify(&s)),
  derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
    AsRef, Display
  ),
)]
pub struct StrictSlug(String);

impl StrictSlug {
  /// Creates a new slug, and asserts that no edits are needed.
  ///
  /// This is intended for use with slugs based off string literals, to make
  /// sure that the string literal and the produced slug match exactly.
  pub fn confident(s: &'static str) -> StrictSlug {
    let slug = strict_slugify(s);
    assert_eq!(slug, s, "provided string is not already a valid slug");
    StrictSlug::new(slug)
  }
}

/// A lax slug, meant to accomodate Nix store paths.
#[nutype::nutype(
  sanitize(with = |s: String| lax_slugify(&s)),
  derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
    AsRef, Display
  ),
)]
pub struct LaxSlug(String);

impl LaxSlug {
  /// Creates a new slug, and asserts that no edits are needed.
  ///
  /// This is intended for use with slugs based off string literals, to make
  /// sure that the string literal and the produced slug match exactly.
  pub fn confident(s: &'static str) -> LaxSlug {
    let slug = lax_slugify(s);
    assert_eq!(slug, s, "provided string is not already a valid slug");
    LaxSlug::new(slug)
  }
}

/// A slug that can be either strict or lax.
#[derive(
  Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash,
)]
pub enum EitherSlug {
  /// A strict slug.
  Strict(StrictSlug),
  /// A lax slug.
  Lax(LaxSlug),
}

impl From<StrictSlug> for EitherSlug {
  fn from(slug: StrictSlug) -> Self { Self::Strict(slug) }
}

impl From<LaxSlug> for EitherSlug {
  fn from(slug: LaxSlug) -> Self { Self::Lax(slug) }
}

impl fmt::Display for EitherSlug {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Strict(slug) => write!(f, "{}", slug),
      Self::Lax(slug) => write!(f, "{}", slug),
    }
  }
}
