//! Health checks and reporting.

use std::ops::Deref;

use serde::{Deserialize, Serialize};

/// Describes a component that can be health checked.
#[async_trait::async_trait]
pub trait HealthReporter: Send + Sync + 'static {
  /// The name of this component.
  const NAME: &'static str;

  /// The type of health report this component can produce.
  type HealthReport: Into<ComponentHealth>;

  /// Perform a health check on this component.
  async fn health_check(&self) -> Self::HealthReport;
}

#[async_trait::async_trait]
impl<T, I> HealthReporter for T
where
  T: Deref<Target = I> + Send + Sync + 'static,
  I: HealthReporter,
{
  const NAME: &'static str = I::NAME;
  type HealthReport = I::HealthReport;

  async fn health_check(&self) -> Self::HealthReport {
    self.deref().health_check().await
  }
}

/// Describes a component that can provide a health report.
#[async_trait::async_trait]
pub trait HealthAware: Send + Sync + 'static {
  /// Perform a health check on this component.
  async fn health_report(&self) -> ComponentHealthReport;
}

#[async_trait::async_trait]
impl<T> HealthAware for T
where
  T: HealthReporter,
{
  async fn health_report(&self) -> ComponentHealthReport {
    ComponentHealthReport {
      name:   T::NAME.to_string(),
      health: self.health_check().await.into(),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A component health report.
pub struct ComponentHealthReport {
  name:   String,
  health: ComponentHealth,
}

/// A description of the health of a component.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ComponentHealth {
  /// The component's health is its components plus a composite status.
  Composite(CompositeComponentHealth),
  /// The component's health is the sum of its components.
  Additive(AdditiveComponentHealth),
  /// The component's health is fully tied to a single dependency.
  Singular(SingularComponentHealth),
  /// The component is intrinsically up.
  IntrensicallyUp,
  /// The component is intrinsically down.
  IntrensicallyDown,
}

impl From<CompositeComponentHealth> for ComponentHealth {
  fn from(v: CompositeComponentHealth) -> Self { Self::Composite(v) }
}

impl From<AdditiveComponentHealth> for ComponentHealth {
  fn from(v: AdditiveComponentHealth) -> Self { Self::Additive(v) }
}

impl From<SingularComponentHealth> for ComponentHealth {
  fn from(v: SingularComponentHealth) -> Self { Self::Singular(v) }
}

impl From<IntrensicallyUp> for ComponentHealth {
  fn from(_: IntrensicallyUp) -> Self { Self::IntrensicallyUp }
}

impl From<IntrensicallyDown> for ComponentHealth {
  fn from(_: IntrensicallyDown) -> Self { Self::IntrensicallyDown }
}

/// The health of a component, described as a composition of itself and its
/// components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeComponentHealth {
  /// The status of the component, tested as a whole.
  composite_statuses: Vec<ComponentHealthReport>,
  /// The status of the component's constituents
  additive:           AdditiveComponentHealth,
}

/// The health of a component, described as the addition of its components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdditiveComponentHealth {
  /// The status of the component's constituents.
  components: Vec<ComponentHealthReport>,
}

/// The health of a component which is fully tied to a single dependency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingularComponentHealth {
  /// The status of the component.
  status: HealthStatus,
}

/// The health of a component which cannot statefully fail.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IntrensicallyUp;

/// The health of a component which is always down.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IntrensicallyDown;

/// The health status of a component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
  /// The component is up and running.
  Ok,
  /// The component is degraded, but still running.
  Degraded(Vec<DegredationMessage>),
  /// The component is down.
  Down(Vec<FailureMessage>),
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
