use rope::Backend;

#[tokio::main]
async fn main() -> miette::Result<()> {
  tracing_subscriber::fmt::init();

  println!(art::ascii_art!("../../media/ascii_logo.png"));

  let backend = rope::RedisBackend::<tasks::HealthCheckTask>::new(()).await;

  backend.consume().await;

  Ok(())
}
