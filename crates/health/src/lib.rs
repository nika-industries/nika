//! Health checks and reporting.
//!
//! This crate provides a simple interface for defining health checks on
//! hexagonal components.
//!
//! # Usage
//! Implement the [`HealthReporter`] trait for your component. That's it.
//!
//! There is a blanket
//! implementation of [`HealthAware`] for all types that implement
//! [`HealthReporter`], and there is a blanket implementation of
//! [`HealthReporter`] for any type that derefs to a type that implements
//! [`HealthReporter`] (e.g. any combination of smart pointer or dynamic
//! dispatch).

use std::future::Future;

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
impl<T, I> HealthReporter for T
where
  T: std::ops::Deref<Target = I> + Send + Sync + 'static,
  I: HealthReporter + ?Sized,
{
  fn name(&self) -> &'static str { self.deref().name() }
  async fn health_check(&self) -> ComponentHealth {
    self.deref().health_check().await
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

impl ComponentHealthReport {
  /// Calculate the overall status of this component.
  pub fn overall_status(&self) -> HealthStatus {
    self.health.recursive_status()
  }
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

impl ComponentHealth {
  /// Recursively calculate the [`HealthStatus`] of this [`ComponentHealth`].
  fn recursive_status(&self) -> HealthStatus {
    match self {
      ComponentHealth::Composite(health) => {
        let mut status = HealthStatus::Ok;
        for report in &health.composite_statuses {
          status = status.merge(&report.health.recursive_status());
        }
        status
      }
      ComponentHealth::Additive(health) => {
        let mut status = HealthStatus::Ok;
        for report in &health.components {
          status = status.merge(&report.health.recursive_status());
        }
        status
      }
      ComponentHealth::Singular(health) => health.status.clone(),
      ComponentHealth::IntrensicallyUp => HealthStatus::Ok,
      ComponentHealth::IntrensicallyDown => HealthStatus::Down(vec![]),
    }
  }
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

impl AdditiveComponentHealth {
  /// Create a new `AdditiveComponentHealth` from a collection of futures.
  pub async fn from_futures<
    I: IntoIterator<Item = impl Future<Output = ComponentHealthReport>>,
  >(
    iter: I,
  ) -> Self {
    AdditiveComponentHealth {
      components: futures::future::join_all(iter).await,
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

impl HealthStatus {
  /// Merge two health statuses.
  pub fn merge(&self, other: &HealthStatus) -> HealthStatus {
    match (self, other) {
      (HealthStatus::Ok, HealthStatus::Ok) => HealthStatus::Ok,
      (HealthStatus::Degraded(a), HealthStatus::Degraded(b)) => {
        HealthStatus::Degraded(a.iter().chain(b.iter()).cloned().collect())
      }
      (HealthStatus::Down(a), HealthStatus::Down(b)) => {
        HealthStatus::Down(a.iter().chain(b.iter()).cloned().collect())
      }
      (HealthStatus::Ok, HealthStatus::Degraded(b)) => {
        HealthStatus::Degraded(b.clone())
      }
      (HealthStatus::Ok, HealthStatus::Down(b)) => {
        HealthStatus::Down(b.clone())
      }
      (HealthStatus::Degraded(a), HealthStatus::Ok) => {
        HealthStatus::Degraded(a.clone())
      }
      (HealthStatus::Degraded(a), HealthStatus::Down(b)) => HealthStatus::Down(
        a.iter()
          .map(DegredationMessage::as_failure_message)
          .chain(b.iter().cloned())
          .collect(),
      ),
      (HealthStatus::Down(a), HealthStatus::Ok) => {
        HealthStatus::Down(a.clone())
      }
      (HealthStatus::Down(a), HealthStatus::Degraded(b)) => HealthStatus::Down(
        a.iter()
          .cloned()
          .chain(b.iter().map(DegredationMessage::as_failure_message))
          .collect(),
      ),
    }
  }
}

/// A message describing why a component is degraded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegredationMessage(String);

impl DegredationMessage {
  /// Create a new `DegredationMessage`.
  pub fn new(message: &str) -> DegredationMessage {
    DegredationMessage(message.to_string())
  }
  /// Convert this `DegredationMessage` into a `FailureMessage`.
  pub(crate) fn as_failure_message(&self) -> FailureMessage {
    FailureMessage(self.0.clone())
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
