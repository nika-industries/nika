//! Binary for the main archive fetch route.

mod fetcher_error;

use std::{ops::Deref, path::PathBuf, str::FromStr};

use axum::{
  body::Body,
  extract::Path,
  http::HeaderMap,
  response::{IntoResponse, Response},
  routing::get,
  Router,
};
use mollusk::ExternalApiError;
use serde::Deserialize;
use storage::{DynStorageClient, StorageClientGenerator};

use self::fetcher_error::FetcherError;

#[derive(Deserialize)]
#[serde(untagged)]
enum UntaggedResult<T, E> {
  Ok(T),
  Err(E),
}

impl<T, E> UntaggedResult<T, E> {
  fn into_result(self) -> Result<T, E> {
    match self {
      UntaggedResult::Ok(value) => Ok(value),
      UntaggedResult::Err(error) => Err(error),
    }
  }
}

async fn get_fetch_payload(
  store_name: String,
  path: String,
  token_id: Option<String>,
  token_secret: Option<String>,
) -> Result<dvf::StorageCredentials, mollusk::PrepareFetchPayloadError> {
  let client = reqwest::Client::new();
  let response = client
    .get("http://localhost:3000/fetch_payload".to_string())
    .json(&(store_name, path, token_id, token_secret))
    .send()
    .await
    .unwrap()
    .json::<UntaggedResult<
      dvf::StorageCredentials,
      mollusk::PrepareFetchPayloadError,
    >>()
    .await
    .unwrap()
    .into_result()?;
  Ok(response)
}

#[tracing::instrument(skip(headers))]
async fn fetch_handler(
  Path((store_name, path)): Path<(String, String)>,
  headers: HeaderMap,
) -> Result<Response, ExternalApiError> {
  let token_id_secret_pair = headers
    .get("authorization")
    .and_then(|value| value.to_str().ok())
    .map(|value| value.to_string());
  let token_id = token_id_secret_pair
    .clone()
    .and_then(|pair| pair.split(':').next().map(|s| s.to_string()));
  let token_secret = token_id_secret_pair
    .and_then(|pair| pair.split(':').nth(1).map(|s| s.to_string()));

  let creds =
    get_fetch_payload(store_name, path.clone(), token_id, token_secret).await?;
  let client = creds.client().await.map_err(FetcherError::StoreInitError)?;

  let response = fetch_path_from_client(&client, path).await?;
  Ok(response)
}

#[tracing::instrument(skip(client))]
async fn fetch_path_from_client(
  client: impl Deref<Target = DynStorageClient>,
  path: String,
) -> Result<Response, FetcherError> {
  // the error type here is `Infalliable`
  let path = PathBuf::from_str(&path).unwrap();

  let stream = client.read(&path).await?;

  tracing::info!("fetching path");
  Ok(Body::from_stream(stream).into_response())
}

#[tokio::main]
async fn main() -> miette::Result<()> {
  let filter = tracing_subscriber::EnvFilter::try_from_default_env()
    .unwrap_or(tracing_subscriber::EnvFilter::new("info"));
  tracing_subscriber::fmt().with_env_filter(filter).init();

  art::ascii_art!("../../media/ascii_logo.png");

  let app = Router::new().route("/:name/*path", get(fetch_handler));

  let bind_address = "0.0.0.0:4000";
  let listener = tokio::net::TcpListener::bind(bind_address).await.unwrap();

  tracing::info!("listening on `{bind_address}`");
  axum::serve(listener, app).await.unwrap();

  Ok(())
}
