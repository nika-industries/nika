//! **D**ata **V**alidation **F**undamentals.
//!
//! This crate provides the fundamental types used for data validation all over
//! Nika. Types from this crate are meant to be used in the domain layer, and
//! should replace all bare or primitive types used in domain models, e.g.
//! [`EntityName`] over `String`.
//!
//! These types should also be used to maintain the validation barrier inside
//! all business logic, e.g. returning a [`FileSize`] from the
//! `storage::StorageClient::write()` method instead of a `u64`.

mod compression;
mod creds;
mod files;
mod names;
mod secrets;

pub use slugger;

pub use self::{creds::*, files::*, names::*, secrets::*};
