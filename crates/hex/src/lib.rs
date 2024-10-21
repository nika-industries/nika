//! Hexagonal architecture primitives.
//!
//! This is the starting point for anything defined to be within the Glorious
//! Hexagon of Business Logic (or `GHBLi` for short). Every hexagonal trait must
//! require [`Hexagonal`].
//!
//! Right now, the [`Hexagonal`] trait only requires that the implementing type
//! is [`HealthAware`]` + `[`Send`]` + `[`Sync`]`+ 'static`, but this may
//! change in the future. [`Hexagonal`] is a blanket implemented.

pub use health;
use health::HealthAware;

/// The `Hexagonal` trait is the main trait that all hexagonal architecture
/// components must implement.
pub trait Hexagonal: HealthAware + Send + Sync + 'static {}

impl<T> Hexagonal for T where T: HealthAware + Send + Sync + 'static {}
