//! Binary for consuming and running tasks.

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
