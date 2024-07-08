mod fetcher_error;

use std::{ops::Deref, path::PathBuf, str::FromStr};

use axum::{
  body::Body,
  extract::{FromRef, Path, State},
  response::{IntoResponse, Response},
  routing::get,
  Router,
};
use storage::StorageClientGenerator;

use self::fetcher_error::FetcherError;

#[tracing::instrument(skip(db))]
async fn fetch_handler(
  State(db): State<db::DbConnection>,
  Path((store_name, path)): Path<(String, String)>,
) -> Response {
  async move {
    let store = db
      .fetch_store_by_name(&store_name)
      .await
      .map_err(FetcherError::SurrealDbStoreRetrievalError)?
      .ok_or_else(|| FetcherError::NoMatchingStore(store_name))?;

    let client = store.config.client().await;

    let response = fetch_path_from_client(&client, path).await?;
    Ok::<Response, FetcherError>(response)
  }
  .await
  .into_response()
}

#[tracing::instrument(skip(client))]
async fn fetch_path_from_client(
  client: impl Deref<Target = storage::DynStorageClient>,
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

  println!();
  for line in art::ascii_art!("../../media/ascii_logo.png").lines() {
    println!("{}", line);
  }
  println!();

  let app_state = AppState {
    db: db::DbConnection::new().await?,
  };
  let app = Router::new()
    .route("/:name/*path", get(fetch_handler))
    .with_state(app_state);

  let bind_address = "0.0.0.0:3000";
  let listener = tokio::net::TcpListener::bind(bind_address).await.unwrap();

  tracing::info!("listening on `{bind_address}`");
  axum::serve(listener, app).await.unwrap();

  Ok(())
}
