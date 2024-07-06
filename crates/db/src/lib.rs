mod store;

use std::sync::Arc;

use include_dir::{include_dir, Dir};
use miette::{miette, IntoDiagnostic, Result, WrapErr};
use serde::Deserialize;
use surrealdb::{
  engine::remote::ws::{Client as WsClient, Ws},
  opt::auth::Root,
  Result as SurrealResult, Surreal,
};
use tracing::instrument;

const MIGRATIONS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/migrations");

#[derive(Deserialize)]
pub struct Count {
  pub count: usize,
}

#[derive(Clone, Debug)]
pub struct DbConnection(Arc<Surreal<WsClient>>);

impl DbConnection {
  #[instrument]
  pub async fn new() -> Result<Self> {
    let client = Surreal::new::<Ws>(
      std::env::var("SURREAL_WS_URL")
        .into_diagnostic()
        .wrap_err("could not find env var \"SURREAL_WS_URL\"")?,
    )
    .await
    .into_diagnostic()
    .wrap_err_with(|| {
      format!(
        "Could not connect to SurrealDB endpoint: `{}`\n\tNB: don't include \
         the ws:// or wss:// prefix, e.g. `example.com:8080` instead of \
         `wss://example.com:8080`",
        std::env::var("SURREAL_WS_URL").unwrap()
      )
    })?;

    client
      .signin(Root {
        username: &std::env::var("SURREAL_USER")
          .into_diagnostic()
          .wrap_err("could not find env var \"SURREAL_USER\"")?,
        password: &std::env::var("SURREAL_PASS")
          .into_diagnostic()
          .wrap_err("could not find env var \"SURREAL_PASS\"")?,
      })
      .await
      .into_diagnostic()
      .wrap_err("failed to sign in to SurrealDB as root")?;

    Ok(Self(Arc::new(client)))
  }

  #[instrument(skip(self))]
  pub async fn into_inner(self) -> SurrealResult<Surreal<WsClient>> {
    let client = self.use_main().await?;
    Ok(Arc::unwrap_or_clone(client.clone()))
  }

  #[instrument(skip(self))]
  async fn use_main(&self) -> SurrealResult<&Arc<Surreal<WsClient>>> {
    self.0.use_ns("main").use_db("main").await?;
    Ok(&self.0)
  }

  #[instrument(skip(self))]
  pub async fn run_migrations(&self) -> Result<()> {
    let db = self.use_main().await.into_diagnostic()?;

    surrealdb_migrations::MigrationRunner::new(db)
      .load_files(&MIGRATIONS_DIR)
      .up()
      .await
      .map_err(|e| miette!("failed to run surrealdb migrations: {e}"))?;

    Ok(())
  }
}
