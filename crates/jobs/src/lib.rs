use std::convert::Infallible;

use apalis::prelude::Job;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct HealthCheckJob;

impl Job for HealthCheckJob {
  const NAME: &'static str = "jobs::HealthCheckJob";
}

pub async fn execute_health_check_job(
  job: HealthCheckJob,
) -> Result<bool, Infallible> {
  Ok(true)
}
