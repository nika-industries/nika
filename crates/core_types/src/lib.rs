//! Provides core platform-level types used by most crates in the workspace.

#[cfg(feature = "ssr")]
mod ssr;
mod store;

#[cfg(feature = "ssr")]
pub use ssr::*;
pub use store::*;
pub use ulid::Ulid;
