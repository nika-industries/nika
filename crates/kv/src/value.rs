//! Value type for key-value store.

use serde::{de::DeserializeOwned, Serialize};

/// Represents a value in a key-value store.
#[derive(Debug, Clone, PartialEq)]
pub struct Value(Vec<u8>);

impl Value {
  /// Create a new value with the given bytes.
  pub fn new(value: Vec<u8>) -> Self { Self(value) }
  /// Get the inner bytes of the value.
  pub fn into_inner(self) -> Vec<u8> { self.0 }

  /// Serialize a value into a [`Value`], using MessagePack.
  pub fn serialize<T: Serialize>(
    value: &T,
  ) -> Result<Self, rmp_serde::encode::Error> {
    Ok(Self(rmp_serde::to_vec_named(value)?))
  }
  /// Deserialize a value from a [`Value`], using MessagePack.
  pub fn deserialize<T: DeserializeOwned>(
    self,
  ) -> Result<T, rmp_serde::decode::Error> {
    rmp_serde::from_read(self.0.as_slice())
  }
}

impl From<Value> for Vec<u8> {
  fn from(value: Value) -> Self { value.0 }
}

impl From<Vec<u8>> for Value {
  fn from(value: Vec<u8>) -> Self { Self(value) }
}
