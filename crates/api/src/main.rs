//! HTTP server for the Rambit API.
//!
//! The API server itself runs on Axum, and serves to run tasks from the `tasks`
//! crate in response to HTTP requests. It also serves as the platform threshold
//! for authentication.
//!
//! # CLI
//! It has two CLI subcommands: `start` and `health`.
//! - `start` starts the API server in regular operation.
//! - `health` runs the health check, dumps it as JSON to `stdout`, and exits.
//!   This is generally for testing configuration in tests.
//!
//! See `api --help` for more information and other options.
//!
//! # Environment Variables
//! It has no extra required environment variables, outside of those required by
//! its services. If you're missing one, it will tell you. Your exact service
//! configuration depends on a number of other crates, in addition to which
//! things you're mocking.

mod cmd;
mod temp_storage_payload;

use std::{sync::Arc, time::Duration};

use axum::{
  extract::{FromRef, Path, State},
  response::IntoResponse,
  routing::{get, post},
  Json, Router,
};
use clap::Parser;
use cmd::Commands;
use miette::{IntoDiagnostic, Result};
use prime_domain::{
  hex::{
    health::{self, HealthAware},
    retryable::Retryable,
  },
  models,
  repos::TempStorageRepository,
  DynPrimeDomainService,
};
use tasks::Task;
use tracing_subscriber::prelude::*;

use self::{cmd::RuntimeConfig, temp_storage_payload::TempStoragePayload};

#[tracing::instrument(skip(app_state))]
async fn prepare_fetch_payload(
  State(app_state): State<AppState>,
  Json((cache_name, path, token_id, token_secret)): Json<(
    String,
    String,
    Option<String>,
    Option<String>,
  )>,
) -> Result<Json<models::StorageCredentials>, mollusk::InternalApiError> {
  Ok(
    tasks::PrepareFetchPayloadTask {
      cache_name:   models::StrictSlug::new(cache_name),
      token_id:     token_id
        .and_then(|s| models::TokenRecordId::try_from(s).ok()),
      token_secret: token_secret
        .map(|s| models::TokenSecret::new(models::StrictSlug::new(s))),
      path:         models::LaxSlug::new(path),
    }
    .run(app_state.prime_domain_service.clone())
    .await
    .map(Json)?,
  )
}

#[tracing::instrument(skip(app_state, payload))]
async fn naive_upload(
  State(app_state): State<AppState>,
  Path((cache_name, original_path)): Path<(String, String)>,
  payload: TempStoragePayload,
) -> Result<(), mollusk::ExternalApiError> {
  let path = models::LaxSlug::new(original_path.clone());
  if path.to_string() != original_path {
    return Err(
      mollusk::InvalidPathError {
        path: original_path,
      }
      .into(),
    );
  }

  let payload_path = payload.upload().await.unwrap();
  tasks::NaiveUploadTask {
    cache_name: models::StrictSlug::new(cache_name),
    path,
    temp_storage_path: payload_path,
  }
  .run(app_state.prime_domain_service.clone())
  .await
  .unwrap();
  Ok(())
}

async fn dummy_root_handler() -> impl IntoResponse {
  "You've reached the root endpoint of the Rambit API binary.\nYou probably \
   meant to go somewhere else."
}

#[tracing::instrument(skip(app_state))]
async fn health_handler(
  State(app_state): State<AppState>,
) -> impl IntoResponse {
  let report = app_state.health_report().await;
  let overall_status = report.overall_status();
  Json(serde_json::json!({
    "report": report,
    "overall_status": overall_status,
  }))
}

#[derive(Clone, FromRef)]
struct AppState {
  prime_domain_service: DynPrimeDomainService,
}

impl AppState {
  async fn build(config: &RuntimeConfig) -> Result<Self> {
    let tikv_store_init = move || async move {
      prime_domain::repos::db::kv::tikv::TikvClient::new_from_env().await
    };
    let retryable_tikv_store =
      Retryable::init(5, Duration::from_secs(2), tikv_store_init).await;
    let kv_db_adapter = Arc::new(
      prime_domain::repos::db::KvDatabaseAdapter::new(retryable_tikv_store),
    );
    let cache_repo =
      prime_domain::repos::CacheRepositoryCanonical::new(kv_db_adapter.clone());
    let store_repo =
      prime_domain::repos::StoreRepositoryCanonical::new(kv_db_adapter.clone());
    let token_repo =
      prime_domain::repos::TokenRepositoryCanonical::new(kv_db_adapter.clone());
    let entry_repo =
      prime_domain::repos::EntryRepositoryCanonical::new(kv_db_adapter.clone());
    let temp_storage_repo: Box<dyn TempStorageRepository> = if config
      .mock_temp_storage
    {
      Box::new(prime_domain::repos::TempStorageRepositoryMock::new(
        std::path::PathBuf::from("/tmp/rambit-temp-storage"),
      ))
    } else {
      let temp_storage_creds = prime_domain::TempStorageCreds::new_from_env()?;
      Box::new(
        prime_domain::repos::TempStorageRepositoryCanonical::new(
          temp_storage_creds,
        )
        .await?,
      )
    };
    let user_storage_repo =
      prime_domain::repos::UserStorageRepositoryCanonical::new();

    let prime_domain_service = prime_domain::PrimeDomainServiceCanonical::new(
      cache_repo,
      entry_repo,
      store_repo,
      token_repo,
      temp_storage_repo,
      user_storage_repo,
    );

    Ok(AppState {
      prime_domain_service: Arc::new(Box::new(prime_domain_service)),
    })
  }
}

#[prime_domain::hex::health::async_trait]
impl health::HealthReporter for AppState {
  fn name(&self) -> &'static str { stringify!(AppState) }
  async fn health_check(&self) -> health::ComponentHealth {
    health::AdditiveComponentHealth::from_futures(vec![self
      .prime_domain_service
      .health_report()])
    .await
    .into()
  }
}

#[tokio::main]
async fn main() -> Result<()> {
  let config = RuntimeConfig::parse();

  let filter_layer = tracing_subscriber::EnvFilter::try_from_default_env()
    .unwrap_or(tracing_subscriber::EnvFilter::new("info"));
  let fmt_layer = tracing_subscriber::fmt::layer()
    .with_target(false)
    .with_writer(std::io::stderr);
  let registry = tracing_subscriber::registry()
    .with(filter_layer)
    .with(fmt_layer);

  let use_chrome_tracing = match &config.command {
    Commands::Start { chrome_tracing, .. } => *chrome_tracing,
    Commands::Health => false,
  };
  let _guard = match use_chrome_tracing {
    true => {
      let (chrome_layer, guard) =
        tracing_chrome::ChromeLayerBuilder::new().build();
      registry.with(chrome_layer).init();
      Some(guard)
    }
    false => {
      registry.init();
      None
    }
  };

  art::ascii_art!("../../media/ascii_logo.png");

  tracing::info!("starting up");
  tracing::info!("config: {:?}", config);

  tracing::info!("initializing services");

  let state = AppState::build(&config).await?;

  tracing::info!("finished initializing services");
  let health_report = state.health_report().await;
  tracing::info!(
    "service health: {}",
    serde_json::to_string(&health_report).unwrap()
  );
  tracing::info!("overall health: {:#?}", health_report.overall_status());

  let (bind_address, bind_port) = match &config.command {
    Commands::Health => {
      let health_report = state.health_report().await;
      println!("{}", serde_json::to_string(&health_report).unwrap());
      return Ok(());
    }
    Commands::Start {
      bind_address,
      bind_port,
      ..
    } => (bind_address.clone(), *bind_port),
  };

  tracing::info!("starting server");

  let app = Router::new()
    .route("/health", get(health_handler))
    .route("/naive-upload/:name/*path", post(naive_upload))
    .route("/fetch_payload", get(prepare_fetch_payload))
    .route("/", get(dummy_root_handler))
    .with_state(state);

  let bind_address = format!("{bind_address}:{bind_port}");
  let listener = tokio::net::TcpListener::bind(&bind_address).await.unwrap();

  tracing::info!("listening on `{bind_address}`");
  tokio::spawn(async move { axum::serve(listener, app).await });

  tokio::signal::ctrl_c().await.into_diagnostic()?;

  Ok(())
}
