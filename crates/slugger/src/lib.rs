//! Provides a type-safe slug.

use slug::slugify;

/// A slug, as determined by the [`slug`] library.
#[nutype::nutype(
  sanitize(with = |s: String| slugify(&s)),
  derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
    AsRef, Display
  ),
)]
pub struct StrictSlug(String);

impl StrictSlug {
  /// Creates a new slug, and asserts that no edits are needed.
  ///
  /// This should not be used in production code. It is intended for use with
  /// slugs based off string literals, where constructing the slug with a string
  /// literal often creates a false assumption that the created slug is the same
  /// as what was provided.
  pub fn confident(s: String) -> StrictSlug {
    let slug = slugify(&s);
    assert_eq!(slug, s, "provided string is not already a valid slug");
    StrictSlug::new(slug)
  }
}
