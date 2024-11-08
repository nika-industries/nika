//! Provides the [`Model`] trait.
//!
//! The [`Model`] trait must be implemented for a type to be used as a domain
//! data model. It provides the table name, unique indices, and the ID of the
//! model.

use std::fmt::Debug;

use dvf::slugger::EitherSlug;
use serde::{de::DeserializeOwned, Serialize};

/// A function that returns a slug field value.
pub type SlugFieldGetter<T> = fn(&T) -> EitherSlug;

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
  fn id(&self) -> dvf::RecordId<Self>;
}
