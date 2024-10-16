//! The leptos server crate for the Cartographer app.

use std::sync::Arc;

use axum::{extract::FromRef, Router};
use cart_app::*;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use prime_domain::DynCacheService;

#[derive(Clone, FromRef)]
struct AppState {
  cache_service: DynCacheService,
}

#[tokio::main]
async fn main() -> miette::Result<()> {
  let conf = get_configuration(None).unwrap();
  let addr = conf.leptos_options.site_addr;
  let leptos_options = conf.leptos_options;
  let routes = generate_route_list(App);

  let tikv_adapter =
    Arc::new(prime_domain::repos::db::TikvAdapter::new_from_env().await?);
  let cache_repo =
    prime_domain::repos::CacheRepositoryCanonical::new(tikv_adapter.clone());
  let cache_service = prime_domain::CacheServiceCanonical::new(cache_repo);

  let app_state = AppState {
    cache_service: Arc::new(Box::new(cache_service)),
  };

  let app = Router::new()
    .leptos_routes_with_context(
      &leptos_options,
      routes,
      {
        let app_state = app_state.clone();
        move || {
          provide_context(app_state.cache_service.clone());
        }
      },
      {
        let leptos_options = leptos_options.clone();
        move || shell(leptos_options.clone())
      },
    )
    .fallback(leptos_axum::file_and_error_handler(shell))
    .layer(tower_http::compression::CompressionLayer::new())
    .with_state(leptos_options);

  log!("listening on http://{}", &addr);
  let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
  axum::serve(listener, app.into_make_service())
    .await
    .unwrap();

  Ok(())
}
