//! Applies migrations to the database.

use db::Migratable;
use miette::Result;

#[tokio::main]
async fn main() -> Result<()> {
  let filter = tracing_subscriber::EnvFilter::try_from_default_env()
    .unwrap_or(tracing_subscriber::EnvFilter::new("info"));
  tracing_subscriber::fmt().with_env_filter(filter).init();

  let db = db::TikvAdapter::new_from_env().await?;
  db.migrate().await?;

  tokio::time::sleep(std::time::Duration::from_secs(1)).await;

  Ok(())
}
