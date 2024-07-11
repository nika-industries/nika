use std::convert::Infallible;

use apalis::prelude::Job;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct HealthCheckJob;

impl Job for HealthCheckJob {
  const NAME: &'static str = "jobs::HealthCheckJob";
}

async fn execute_health_check_job() -> Result<bool, Infallible> { Ok(true) }
