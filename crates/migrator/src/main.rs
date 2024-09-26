//! Applies migrations to the database.

use db::Migratable;
use miette::Result;

#[tokio::main]
async fn main() -> Result<()> {
  tracing_subscriber::fmt::init();

  let db = db::TikvAdapter::new_from_env().await?;
  db.migrate().await?;

  tokio::time::sleep(std::time::Duration::from_secs(1)).await;

  Ok(())
}
