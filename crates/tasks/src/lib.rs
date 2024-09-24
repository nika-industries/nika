//! Provides types and business logic for all platform tasks used with [`rope`].

mod naive_upload;
mod prepare_fetch_payload;

pub use rope::Task;

pub use self::{naive_upload::*, prepare_fetch_payload::*};
