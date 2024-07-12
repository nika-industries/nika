use std::convert::Infallible;

use apalis::{prelude::Job, redis::RedisStorage};
use miette::IntoDiagnostic;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub type StorageError = redis::RedisError;

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

pub async fn get_storage<J: Job + Serialize + DeserializeOwned + 'static>(
) -> Result<RedisStorage<J>, miette::Report> {
  let conn =
    apalis::redis::connect(std::env::var("REDIS_URL").into_diagnostic()?)
      .await
      .into_diagnostic()?;
  let config = apalis::redis::Config::default();
  let storage = RedisStorage::new_with_config(conn, config);
  Ok(storage)
}
