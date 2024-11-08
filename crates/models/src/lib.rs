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
//! This crate is a very common dependency for the Rambit workspace, and is
//! generally also the intended access point for depending on [`slugger`],
//! [`dvf`], or [`ulid`].

mod cache;
mod entry;
mod org;
mod perms;
mod store;
mod token;
mod user;

pub use dvf::*;
pub use model::*;
pub use slugger::*;

pub use self::{
  cache::*, entry::*, org::*, perms::*, store::*, token::*, user::*,
};
