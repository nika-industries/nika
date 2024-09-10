//! Applies migrations to the database.

use miette::Result;

#[tokio::main]
async fn main() -> Result<()> {
  tracing_subscriber::fmt::init();

  let db = db::TikvDb::new().await?;
  db.migrate().await?;

  tokio::time::sleep(std::time::Duration::from_secs(1)).await;

  Ok(())
}
