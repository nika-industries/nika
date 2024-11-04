use slugger::StrictSlug;

/// A name for an entity.
///
/// This must be fit to be used in a URL. An entity should be able to be
/// referred to by its name.
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
pub struct EntityName(StrictSlug);

/// A nickname for an entity.
///
/// Just like a name, but it has no URL constraint. An entity should not be
/// queried programmatically by users by its nickname.
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

/// A human's name.
#[nutype::nutype(
  derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Hash,
    AsRef,
    Display
  ),
  validate(not_empty, len_char_max = 255)
)]
pub struct HumanName(String);
