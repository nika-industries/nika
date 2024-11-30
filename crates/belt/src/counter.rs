use std::sync::{
  atomic::{AtomicU64, Ordering},
  Arc,
};

/// A tracking counter for the total number of bytes read from a
/// [`Belt`](crate::Belt).
#[derive(Debug)]
pub struct Counter(Arc<AtomicU64>);

impl Counter {
  /// Get the current count of bytes read.
  pub fn current(&self) -> u64 { self.0.load(Ordering::Acquire) }

  pub(crate) fn new(count: Arc<AtomicU64>) -> Self { Self(count) }
}
