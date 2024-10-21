//! Provides domain model types. Used by most crates in the workspace.
//!
//! This crate provides definitions for all the domain data models, and also
//! exposes all their constituent types, as defined by [`slugger`], [`dvf`], and
//! [`ulid`].
//!
//! The [`Model`] trait must be implemented for a type to be used as a domain
//! data model. It provides the table name, unique indices, and the ID of the
//! model.
//!
//! This crate is a very common dependency for the Nika workspace, and is
//! generally also the intended access point for depending on [`slugger`],
//! [`dvf`], or [`ulid`].

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

pub use dvf::*;
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

  /// The model's unique indices.
  ///
  /// An array of tuples containing the index name and a function that returns
  /// the index value. The produced value must be unique for each record.
  const UNIQUE_INDICES: &'static [(&'static str, SlugFieldGetter<Self>)];

  /// Returns the model's ID.
  fn id(&self) -> RecordId<Self>;
}
