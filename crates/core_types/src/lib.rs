//! Provides core platform-level types used by most crates in the workspace.

mod org;
mod slug;
#[cfg(feature = "ssr")]
mod ssr;
#[cfg(feature = "ssr")]
mod storage_creds;
mod store;

pub use ulid::Ulid;

pub use self::{org::*, slug::*, store::*};
#[cfg(feature = "ssr")]
pub use self::{ssr::*, storage_creds::*};
