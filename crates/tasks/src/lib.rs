//! Provides types and business logic for all platform tasks used with [`rope`].

use serde::{Deserialize, Serialize};

/// The health check task.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthCheckTask;

#[async_trait::async_trait]
impl rope::Task for HealthCheckTask {
  const NAME: &'static str = "HealthCheck";

  type Response = bool;
  type Error = ();

  async fn run(
    self,
    _state: Self::State,
  ) -> Result<Self::Response, Self::Error> {
    Ok(true)
  }
}
