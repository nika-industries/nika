//! Provides core platform-level types used by most crates in the workspace.

mod slug;
#[cfg(feature = "ssr")]
mod ssr;
#[cfg(feature = "ssr")]
mod storage_creds;
mod store;

pub use ulid::Ulid;

#[cfg(feature = "ssr")]
pub use self::ssr::*;
#[cfg(feature = "ssr")]
pub use self::storage_creds::*;
pub use self::{slug::*, store::*};
