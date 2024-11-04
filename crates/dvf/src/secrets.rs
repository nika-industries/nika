use slugger::StrictSlug;

/// A token secret.
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
pub struct TokenSecret(StrictSlug);
