//! Health checks and reporting.

use serde::{Deserialize, Serialize};

/// The health of a component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
  /// The name of the component.
  name:            String,
  /// The status of the component, tested as a whole.
  holistic_status: HealthStatus,
  /// The status of the component's constituents.
  components:      Vec<ComponentHealth>,
}

/// The health status of a component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
  /// The component is up and running.
  Ok,
  /// The component is degraded, but still running.
  Degraded(Vec<DegredationMessage>),
  /// The component is down.
  Down(Vec<FailureMessage>),
  /// The component cannot statefully fail (i.e. FS access)
  IntrinsicallyUp,
  /// The component's health is fully dependent upon its constituents.
  Additive,
}

/// A message describing why a component is degraded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegredationMessage(String);

impl DegredationMessage {
  /// Create a new `DegredationMessage`.
  pub fn new(message: &str) -> DegredationMessage {
    DegredationMessage(message.to_string())
  }
}

/// A message describing why a component is down.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureMessage(String);

impl FailureMessage {
  /// Create a new `FailureMessage`.
  pub fn new(message: &str) -> FailureMessage {
    FailureMessage(message.to_string())
  }
}
