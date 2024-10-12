//! Health checks and reporting.

use std::sync::Arc;

pub use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Describes a component that can be health checked.
#[async_trait::async_trait]
pub trait HealthReporter: Send + Sync + 'static {
  /// The name of this component.
  fn name(&self) -> &'static str;
  /// Perform a health check on this component.
  async fn health_check(&self) -> ComponentHealth;
}

#[async_trait::async_trait]
impl<T: HealthReporter + ?Sized> HealthReporter for Box<T> {
  fn name(&self) -> &'static str { T::name(self) }
  async fn health_check(&self) -> ComponentHealth {
    T::health_check(self).await
  }
}
#[async_trait::async_trait]
impl<T: HealthReporter + ?Sized> HealthReporter for Arc<T> {
  fn name(&self) -> &'static str { T::name(self) }
  async fn health_check(&self) -> ComponentHealth {
    T::health_check(self).await
  }
}

/// Describes a component that can provide a health report.
#[async_trait::async_trait]
pub trait HealthAware: HealthReporter + Send + Sync + 'static {
  /// Perform a health check on this component.
  async fn health_report(&self) -> ComponentHealthReport;
}

#[async_trait::async_trait]
impl<T: HealthReporter + ?Sized> HealthAware for T {
  async fn health_report(&self) -> ComponentHealthReport {
    ComponentHealthReport {
      name:   self.name().to_string(),
      health: self.health_check().await,
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

impl FromIterator<ComponentHealthReport> for AdditiveComponentHealth {
  fn from_iter<I: IntoIterator<Item = ComponentHealthReport>>(iter: I) -> Self {
    AdditiveComponentHealth {
      components: iter.into_iter().collect(),
    }
  }
}

impl AdditiveComponentHealth {
  /// Create a new `AdditiveComponentHealth`.
  pub fn start(
    component: impl Into<ComponentHealthReport>,
  ) -> AdditiveComponentHealth {
    AdditiveComponentHealth {
      components: vec![component.into()],
    }
  }
  /// Add a component to the health report.
  #[allow(clippy::should_implement_trait)]
  pub fn add(mut self, component: impl Into<ComponentHealthReport>) -> Self {
    self.components.push(component.into());
    self
  }
}

/// The health of a component which is fully tied to a single dependency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingularComponentHealth {
  /// The status of the component.
  status: HealthStatus,
}

impl SingularComponentHealth {
  /// Create a new `SingularComponentHealth`.
  pub fn new(status: HealthStatus) -> SingularComponentHealth {
    SingularComponentHealth { status }
  }
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
