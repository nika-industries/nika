//! Provides access to the database.

mod adapter;
mod migrate;
mod tikv;

pub use self::{
  adapter::{CreateModelError, DatabaseAdapter},
  migrate::Migratable,
  tikv::TikvAdapter,
};
