//! Applies migrations to the database.

use miette::Result;

#[tokio::main]
async fn main() -> Result<()> {
  let db = db::TikvDb::new().await?;
  db.migrate().await?;

  Ok(())
}
