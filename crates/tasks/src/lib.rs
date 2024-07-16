use serde::{Deserialize, Serialize};

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