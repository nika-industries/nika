//! The leptos server crate for the Cartographer app.

use axum::Router;
use cart_app::*;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};

#[derive(Clone)]
struct AppState {}

#[tokio::main]
async fn main() {
  let conf = get_configuration(None).unwrap();
  let addr = conf.leptos_options.site_addr;
  let leptos_options = conf.leptos_options;
  // Generate the list of routes in your Leptos App
  let routes = generate_route_list(App);

  let app_state = AppState {};

  let app = Router::new()
    .leptos_routes_with_context(
      &leptos_options,
      routes,
      {
        let app_state = app_state.clone();
        move || {
          provide_context(app_state.clone());
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

  // run our app with hyper
  // `axum::Server` is a re-export of `hyper::Server`
  log!("listening on http://{}", &addr);
  let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
  axum::serve(listener, app.into_make_service())
    .await
    .unwrap();
}
