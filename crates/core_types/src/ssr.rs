use std::{fmt::Debug, hash::Hash};

use serde::{Deserialize, Serialize};
use surrealdb::{
  opt::{IntoResource, Resource},
  sql::{Id, Thing},
};

use crate::{Store, StoreRecordId, STORE_TABLE_NAME};

/// A [`Ulid`](ulid::Ulid) or a [`Thing`].
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum UlidOrThing {
  /// A [`Ulid`](ulid::Ulid).
  Ulid(ulid::Ulid),
  /// A [`Thing`].
  Thing(Thing),
}

impl From<UlidOrThing> for ulid::Ulid {
  fn from(u: UlidOrThing) -> ulid::Ulid {
    match u {
      UlidOrThing::Ulid(u) => u,
      UlidOrThing::Thing(t) => t.id.to_string().parse().unwrap(),
    }
  }
}

/// A type that can be stored in the database.
///
/// It must be serializable and deserializable, and it must have an associated
/// id type.
///
/// This trait is mainly meant for bounds and to be extended, hence it has no
/// methods.
pub trait CoreModel:
  Clone
  + Debug
  + Serialize
  + for<'a> Deserialize<'a>
  + Sized
  + Send
  + Sync
  + 'static
{
  /// The id type for this model.
  type Id: CoreId<Model = Self> + IntoResource<Option<Self>> + Send;

  /// Get the ID for this model.
  fn id(&self) -> Self::Id;
}

/// A type that can be used as an id for a model.
///
/// Check the crate documentation for info on how to implement this trait.
pub trait CoreId:
  Copy
  + Clone
  + Send
  + Debug
  + PartialEq
  + Eq
  + Hash
  + Serialize
  + for<'a> Deserialize<'a>
  + From<UlidOrThing>
  + IntoResource<Option<Self::Model>>
  + 'static
{
  /// The database table name for this id.
  const TABLE: &'static str;
  /// The model type for this id.
  type Model: CoreModel<Id = Self>;

  /// Create a new id.
  fn new() -> Self;
  /// Create a new id with a specific inner id.
  fn new_with_id(inner_id: ulid::Ulid) -> Self;
  /// Convert to a surrealdb [`Thing`].
  fn to_thing(&self) -> Thing;
}

macro_rules! impl_table {
  ($id_type:ident, $model_type:ident, $table:ident) => {
    impl From<UlidOrThing> for $id_type {
      fn from(u: UlidOrThing) -> $id_type { $id_type(ulid::Ulid::from(u)) }
    }

    impl<R> IntoResource<Option<R>> for $id_type {
      fn into_resource(self) -> Result<Resource, surrealdb::Error> {
        Ok(Resource::RecordId(self.to_thing()))
      }
    }

    impl CoreId for $id_type {
      const TABLE: &'static str = $table;
      type Model = $model_type;

      fn new() -> Self { $id_type(ulid::Ulid::new()) }
      fn new_with_id(inner_id: ulid::Ulid) -> Self { $id_type(inner_id) }
      fn to_thing(&self) -> Thing {
        Thing::from((Self::TABLE.to_string(), Id::String(self.0.to_string())))
      }
    }

    impl CoreModel for $model_type {
      type Id = $id_type;
      fn id(&self) -> $id_type { self.id }
    }
  };
}

impl_table!(StoreRecordId, Store, STORE_TABLE_NAME);
