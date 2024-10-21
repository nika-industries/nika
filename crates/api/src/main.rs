//! HTTP server for the Nika API.
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

use std::sync::Arc;

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
  hex::health::{self, HealthAware},
  models,
  repos::TempStorageRepository,
  DynCacheService, DynEntryService, DynStoreService, DynTempStorageService,
  DynTokenService,
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
    .run((
      app_state.cache_service.clone(),
      app_state.store_service.clone(),
      app_state.token_service.clone(),
      app_state.entry_service.clone(),
    ))
    .await
    .map(Json)?,
  )
}

#[axum::debug_handler]
#[tracing::instrument(skip(app_state, payload))]
async fn naive_upload(
  State(app_state): State<AppState>,
  Path((cache_name, path)): Path<(String, String)>,
  payload: TempStoragePayload,
) -> impl IntoResponse {
  let payload_path = payload.upload().await.unwrap();
  tasks::NaiveUploadTask {
    cache_name:        models::StrictSlug::new(cache_name),
    path:              path.into(),
    temp_storage_path: payload_path,
  }
  .run((
    app_state.cache_service.clone(),
    app_state.store_service.clone(),
    app_state.entry_service.clone(),
    app_state.temp_storage_service.clone(),
  ))
  .await
  .unwrap();
}

#[derive(Clone, FromRef)]
struct AppState {
  cache_service:        DynCacheService,
  store_service:        DynStoreService,
  token_service:        DynTokenService,
  entry_service:        DynEntryService,
  temp_storage_service: DynTempStorageService,
}

impl AppState {
  async fn build(config: &RuntimeConfig) -> Result<Self> {
    let tikv_adapter =
      Arc::new(prime_domain::repos::db::TikvAdapter::new_from_env().await?);
    let cache_repo =
      prime_domain::repos::CacheRepositoryCanonical::new(tikv_adapter.clone());
    let store_repo =
      prime_domain::repos::StoreRepositoryCanonical::new(tikv_adapter.clone());
    let token_repo =
      prime_domain::repos::TokenRepositoryCanonical::new(tikv_adapter.clone());
    let entry_repo =
      prime_domain::repos::EntryRepositoryCanonical::new(tikv_adapter.clone());
    let temp_storage_repo: Box<dyn TempStorageRepository> = if config
      .mock_temp_storage
    {
      Box::new(prime_domain::repos::TempStorageRepositoryMock::new(
        std::path::PathBuf::from("/tmp/nika-temp-storage"),
      ))
    } else {
      let temp_storage_creds = storage::temp::TempStorageCreds::new_from_env()?;
      Box::new(
        prime_domain::repos::TempStorageRepositoryCanonical::new(
          temp_storage_creds,
        )
        .await?,
      )
    };

    let cache_service = prime_domain::CacheServiceCanonical::new(cache_repo);
    let store_service = prime_domain::StoreServiceCanonical::new(store_repo);
    let token_service = prime_domain::TokenServiceCanonical::new(token_repo);
    let entry_service = prime_domain::EntryServiceCanonical::new(entry_repo);
    let temp_storage_service =
      prime_domain::TempStorageServiceCanonical::new(temp_storage_repo);

    Ok(AppState {
      cache_service:        Arc::new(Box::new(cache_service)),
      store_service:        Arc::new(Box::new(store_service)),
      token_service:        Arc::new(Box::new(token_service)),
      entry_service:        Arc::new(Box::new(entry_service)),
      temp_storage_service: Arc::new(Box::new(temp_storage_service)),
    })
  }
}

#[prime_domain::hex::health::async_trait]
impl health::HealthReporter for AppState {
  fn name(&self) -> &'static str { stringify!(AppState) }
  async fn health_check(&self) -> health::ComponentHealth {
    health::AdditiveComponentHealth::from_iter(vec![
      self.cache_service.health_report().await,
      self.store_service.health_report().await,
      self.token_service.health_report().await,
      self.entry_service.health_report().await,
      self.temp_storage_service.health_report().await,
    ])
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
  tracing::info!(
    "service health: {}",
    serde_json::to_string(&state.health_report().await).unwrap()
  );

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
    .route("/naive-upload/:name/*path", post(naive_upload))
    .route("/fetch_payload", get(prepare_fetch_payload))
    .with_state(state);

  let bind_address = format!("{bind_address}:{bind_port}");
  let listener = tokio::net::TcpListener::bind(&bind_address).await.unwrap();

  tracing::info!("listening on `{bind_address}`");
  tokio::spawn(async move { axum::serve(listener, app).await });

  tokio::signal::ctrl_c().await.into_diagnostic()?;

  Ok(())
}
