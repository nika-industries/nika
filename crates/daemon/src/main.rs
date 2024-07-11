use std::time::Duration;

use apalis::{
  layers::tracing::TraceLayer,
  prelude::{Monitor, WorkerBuilder, WorkerFactoryFn},
  redis::RedisStorage,
  utils::TokioExecutor,
};
use miette::IntoDiagnostic;

#[tokio::main]
async fn main() -> miette::Result<()> {
  tracing_subscriber::fmt::init();

  println!();
  for line in art::ascii_art!("../../media/ascii_logo.png").lines() {
    println!("{}", line);
  }
  println!();

  tracing::info!("connecting to job store...");
  let conn =
    apalis::redis::connect(std::env::var("REDIS_URL").into_diagnostic()?)
      .await
      .into_diagnostic()?;
  let config = apalis::redis::Config::default();
  let storage = RedisStorage::new_with_config(conn, config);
  tracing::info!("connected to job store");

  let monitor = Monitor::<TokioExecutor>::new()
    .register_with_count(2, {
      WorkerBuilder::new(names::name())
        .layer(TraceLayer::new())
        .with_storage(storage)
        .build_fn(jobs::execute_health_check_job)
    })
    .shutdown_timeout(Duration::from_secs(5));

  tracing::info!("listening for jobs");
  monitor
    .run_with_signal(tokio::signal::ctrl_c()) // This will wait for ctrl+c then gracefully shutdown
    .await
    .into_diagnostic()?;

  Ok(())
}
