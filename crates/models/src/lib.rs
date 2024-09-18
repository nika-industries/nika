//! Provides db model types. Used by most crates in the workspace.

mod cache;
mod entry;
mod org;
mod perms;
mod record_id;
mod storage_creds;
mod store;
mod token;
mod user;

use std::fmt::Debug;

use serde::{de::DeserializeOwned, Serialize};
pub use slugger::*;
pub use ulid::Ulid;

pub use self::{
  cache::*, entry::*, org::*, perms::*, record_id::RecordId, storage_creds::*,
  store::*, token::*, user::*,
};

type SlugFieldGetter<T> = fn(&T) -> EitherSlug;

/// Represents a model in the database.
pub trait Model:
  Clone + Debug + PartialEq + Serialize + DeserializeOwned + Send + Sync + 'static
{
  /// The table name in the database.
  const TABLE_NAME: &'static str;

  /// The model's indices, an array of tuples containing the index name and a
  /// function that returns the index value.
  const INDICES: &'static [(&'static str, SlugFieldGetter<Self>)];

  /// Returns the model's ID.
  fn id(&self) -> RecordId<Self>;
}
