//! Hexagonal architecture primitives.

pub use health;
use health::HealthAware;

/// The `Hexagonal` trait is the main trait that all hexagonal architecture
/// components must implement.
pub trait Hexagonal: HealthAware + Send + Sync + 'static {}

impl<T> Hexagonal for T where T: HealthAware + Send + Sync + 'static {}
