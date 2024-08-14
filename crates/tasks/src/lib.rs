//! Provides types and business logic for all platform tasks used with [`rope`].

mod db;
mod fetch_store_creds;
mod naive_upload;
mod prepare_fetch_payload;

pub use rope::Task;

pub use self::{db::*, fetch_store_creds::*, naive_upload::*};
