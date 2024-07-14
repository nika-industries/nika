use std::convert::Infallible;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct HealthCheckJob;

pub async fn execute_health_check_job(
  job: HealthCheckJob,
) -> Result<bool, Infallible> {
  Ok(true)
}
