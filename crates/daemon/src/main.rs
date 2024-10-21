//! Binary for consuming and running tasks.
//!
//! This is useless and disabled for the moment because we have no long-running
//! tasks, but we will use it when we do.
//!
//! It builds up the same app state as the API binary, and then spawns tokio
//! tasks for each type of task that we want to listen for. It then executes
//! each task that lands in the redis queue using its state.

// use rope::Backend;

#[tokio::main]
async fn main() -> miette::Result<()> {
  let filter = tracing_subscriber::EnvFilter::try_from_default_env()
    .unwrap_or(tracing_subscriber::EnvFilter::new("info"));
  tracing_subscriber::fmt().with_env_filter(filter).init();

  art::ascii_art!("../../media/ascii_logo.png");

  // let backend = rope::RedisBackend::<tasks::HealthCheckTask>::new(()).await;
  // backend.consume().await;

  Ok(())
}
