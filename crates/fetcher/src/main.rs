mod fetcher_error;

use std::{fmt::Debug, ops::Deref, path::PathBuf, str::FromStr};

use axum::{
  body::Body,
  extract::{FromRef, Path, State},
  response::{IntoResponse, Response},
  routing::{get, post},
  Router,
};
use miette::IntoDiagnostic;
use mollusk::RenderApiError;
use storage::{DynStorageClient, StorageClientGenerator};
use tokio_stream::StreamExt;

use self::fetcher_error::FetcherError;

#[tracing::instrument(skip(db))]
async fn get_store_client(
  db: db::DbConnection,
  store_name: impl AsRef<str> + Debug,
) -> Result<DynStorageClient, FetcherError> {
  let creds = match store_name.as_ref() {
    "test-local" => {
      tracing::info!("using hard-coded store \"test-local\"");
      core_types::StorageCredentials::Local(
        core_types::LocalStorageCredentials(PathBuf::from("/tmp/nika")),
      )
    }
    "nika-temp" => {
      tracing::info!("using hard-coded store \"nika-temp\"");
      core_types::StorageCredentials::R2(
        core_types::R2StorageCredentials::Default {
          access_key:        std::env::var("R2_TEMP_ACCESS_KEY")
            .into_diagnostic()
            .map_err(FetcherError::StoreInitError)?,
          secret_access_key: std::env::var("R2_TEMP_SECRET_ACCESS_KEY")
            .into_diagnostic()
            .map_err(FetcherError::StoreInitError)?,
          endpoint:          std::env::var("R2_TEMP_ENDPOINT")
            .into_diagnostic()
            .map_err(FetcherError::StoreInitError)?,
          bucket:            std::env::var("R2_TEMP_BUCKET")
            .into_diagnostic()
            .map_err(FetcherError::StoreInitError)?,
        },
      )
    }
    store_name => {
      db.fetch_store_by_name(store_name.as_ref())
        .await
        .map_err(FetcherError::SurrealDbStoreRetrievalError)?
        .ok_or_else(|| FetcherError::NoMatchingStore(store_name.to_string()))?
        .config
    }
  };

  let client = creds.client().await.map_err(FetcherError::StoreInitError)?;

  Ok(client)
}

#[tracing::instrument(skip(db))]
async fn fetch_handler(
  State(db): State<db::DbConnection>,
  Path((store_name, path)): Path<(String, String)>,
) -> impl IntoResponse {
  async move {
    let client = get_store_client(db, store_name).await?;

    let response = fetch_path_from_client(&client, path).await?;
    Ok::<Response, FetcherError>(response)
  }
  .await
  .render_api_error()
}

#[axum::debug_handler]
#[tracing::instrument(skip(db, body))]
async fn test_upload(
  State(db): State<db::DbConnection>,
  Path((store_name, path)): Path<(String, String)>,
  body: axum::body::Body,
) -> impl IntoResponse {
  let client = get_store_client(db, store_name).await.unwrap();

  let stream = body.into_data_stream().map(|result| {
    result.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))
  });
  let reader = tokio_util::io::StreamReader::new(stream);

  client
    .write(
      PathBuf::from_str(&path).unwrap().as_path(),
      Box::new(reader),
    )
    .await
    .unwrap();
}

#[tracing::instrument(skip(client))]
async fn fetch_path_from_client(
  client: impl Deref<Target = DynStorageClient>,
  path: String,
) -> Result<Response, FetcherError> {
  // the error type here is `Infalliable`
  let path = PathBuf::from_str(&path).unwrap();

  let reader = client.read(&path).await?;
  let stream = tokio_util::io::ReaderStream::new(reader);

  tracing::info!("fetching path");
  Ok(Body::from_stream(stream).into_response())
}

#[derive(Clone, FromRef)]
struct AppState {
  db: db::DbConnection,
}

#[tokio::main]
async fn main() -> miette::Result<()> {
  tracing_subscriber::fmt::init();

  println!(art::ascii_art!("../../media/ascii_logo.png"));

  let app_state = AppState {
    db: db::DbConnection::new().await?,
  };
  let app = Router::new()
    .route("/test-upload/:name/*path", post(test_upload))
    .route("/:name/*path", get(fetch_handler))
    .with_state(app_state);

  let bind_address = "0.0.0.0:3000";
  let listener = tokio::net::TcpListener::bind(bind_address).await.unwrap();

  tracing::info!("listening on `{bind_address}`");
  axum::serve(listener, app).await.unwrap();

  Ok(())
}
