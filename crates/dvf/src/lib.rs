//! Data Validation Fundamentals.

pub use slugger;
use slugger::StrictSlug;

/// A nickname for an entity.
///
/// There is no hard requirement that this is unique or that it works in a URL.
#[nutype::nutype(derive(
  Debug,
  Clone,
  Serialize,
  Deserialize,
  PartialEq,
  Eq,
  Hash,
  AsRef,
  Display
))]
pub struct EntityNickname(StrictSlug);
