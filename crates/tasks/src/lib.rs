use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct HealthCheckTask;

#[async_trait::async_trait]
impl rope::Task for HealthCheckTask {
  const NAME: &'static str = "HealthCheck";

  type Response = bool;
  type Error = ();

  async fn run(self) -> Result<Self::Response, Self::Error> {
    tokio::time::sleep(tokio::time::Duration::from_secs_f32(1.0)).await;
    Ok(true)
  }
}
