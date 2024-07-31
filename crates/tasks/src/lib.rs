//! Provides types and business logic for all platform tasks used with [`rope`].

mod fetch_store_creds;
mod health_check;

pub use rope::Task;

pub use self::{
  fetch_store_creds::FetchStoreCredsTask, health_check::HealthCheckTask,
};
