//! Provides core platform-level types. Used by most crates in the workspace.

mod org;
mod perms;
mod storage_creds;
mod store;
mod token;
mod user;

use std::{fmt::Debug, hash::Hash};

use serde::{de::DeserializeOwned, Serialize};
use slugger::Slug;
pub use ulid::Ulid;

pub use self::{
  org::*, perms::*, storage_creds::*, store::*, token::*, user::*,
};

type SlugFieldGetter<T> = fn(&T) -> Slug;

/// Represents a model in the database.
pub trait Model:
  Clone + Debug + PartialEq + Serialize + DeserializeOwned + 'static
{
  /// The model's ID type
  type Id: Clone + Debug + PartialEq + Eq + Hash + Serialize + DeserializeOwned;
  /// The table name in the database.
  const TABLE_NAME: &'static str;

  /// The model's indices, an array of tuples containing the index name and a
  /// function that returns the index value.
  const INDICES: &'static [(&'static str, SlugFieldGetter<Self>)];

  /// Returns the model's ID.
  fn id(&self) -> Self::Id;
}
