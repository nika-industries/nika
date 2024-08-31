//! Provides a type-safe slug.

pub mod lax;
pub mod strict;

/// A strict slug that only allows lowercase ASCII letters, numbers, and
/// hyphens.
#[nutype::nutype(
  sanitize(with = |s: String| strict::strict_slugify(&s)),
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
    let slug = strict::strict_slugify(s);
    assert_eq!(slug, s, "provided string is not already a valid slug");
    StrictSlug::new(slug)
  }
}

/// A lax slug that allows lowercase and uppercase ASCII letters, numbers,
/// hyphens, underscores, dots, and plus signs.
#[nutype::nutype(
  sanitize(with = |s: String| lax::lax_slugify(&s)),
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
    let slug = lax::lax_slugify(s);
    assert_eq!(slug, s, "provided string is not already a valid slug");
    LaxSlug::new(slug)
  }
}
