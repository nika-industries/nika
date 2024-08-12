pub use slug::slugify;

#[nutype::nutype(
  sanitize(with = |s: String| slugify(&s)),
  derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash
  ),
)]
pub struct Slug(String);
