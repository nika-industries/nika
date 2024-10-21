//! Utilities for interacting directly with Nix & Co.
//!
//! This crate is kind of half-baked, but it's generally what we will use for
//! interacting directly with Nix primitives, the Nix daemon, and the Nix store.
//!
//! Expect more docs when this crate is more mature.

#[cfg(feature = "nar")]
pub mod nar;
