pub use slug::slugify;

/// A slug, as determined by the [`slug`] library.
#[nutype::nutype(
  sanitize(with = |s: String| slugify(&s)),
  derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash
  ),
)]
pub struct Slug(String);
