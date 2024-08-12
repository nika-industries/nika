//! Provides types and business logic for all platform tasks used with [`rope`].

mod fetch_store_creds;
mod naive_upload;

pub use rope::Task;

pub use self::{
  fetch_store_creds::FetchStoreCredsTask, naive_upload::NaiveUploadTask,
};
