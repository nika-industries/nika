//! Hexagonal architecture primitives.

/// The `Hexagonal` trait is the main trait that all hexagonal architecture
/// components must implement.
pub trait Hexagonal: Send + Sync + 'static {}
