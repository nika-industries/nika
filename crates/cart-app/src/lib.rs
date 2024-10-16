//! The leptos app crate for the Cartographer app.

mod page_title;

use leptos::{either::Either, prelude::*};
use leptos_meta::{
  provide_meta_context, Link, MetaTags, Style, Stylesheet, Title,
};
use leptos_router::{
  components::{Route, Router, Routes},
  path, SsrMode,
};

use self::page_title::PageTitle;

/// Builds the HTML shell for the application.
pub fn shell(options: LeptosOptions) -> impl IntoView {
  view! {
    <!DOCTYPE html>
    <html lang="en">
      <head>
        <meta charset="utf-8"/>
        <meta name="viewport" content="width=device-width, initial-scale=1"/>
        <AutoReload options=options.clone() />
        <HydrationScripts options/>
        <MetaTags/>
      </head>
      <body>
        <App/>
      </body>
    </html>
  }
}

/// The main application component.
#[component]
pub fn App() -> impl IntoView {
  // Provides context that manages stylesheets, titles, meta tags, etc.
  provide_meta_context();

  view! {
    <Stylesheet id="leptos" href="/pkg/cart.css"/>
    <Style>{include_str!("../style/fonts.css")}</Style>
    <Link rel="preload" href="/fonts/inter.ttf" as_="font" type_="font/ttf" crossorigin="anonymous" />

    <Title text="Welcome to Leptos"/>

    <Router>
      <div class="h-screen flex justify-items-start items-start gap-4">
        <SideBar/>
        <div class="container py-8">
          <main>
            <Routes fallback=|| "Page not found.".into_view()>
              <Route path=path!("") view=HomePage/>
              <Route path=path!("/model/cache") view=CacheModelPage ssr=SsrMode::OutOfOrder/>
              <Route path=path!("/model/cache/*id") view=CacheModelPage/>
            </Routes>
          </main>
        </div>
      </div>
    </Router>
  }
}

#[component]
fn HomePage() -> impl IntoView {
  view! {
    <div class="flex flex-col gap-4">
      <PageTitle title="Welcome to Cartographer".to_string() level=1 />
      <p class="text-lg text-content2">"Choose a page on the left to start exploring data."</p>
    </div>
  }
}

#[server]
async fn enumerate_cache_ids(
) -> Result<Vec<models::CacheRecordId>, ServerFnError> {
  let cache_service: Option<_> = use_context();
  let cache_service: prime_domain::DynCacheService = cache_service
    .ok_or(ServerFnError::new("Cache service is not available."))?;

  let ids = cache_service.enumerate_models().await.map_err(|e| {
    ServerFnError::new(format!("Failed to enumerate cache models: {}", e))
  })?;
  Ok(ids)
}

#[component]
fn CacheModelPage() -> impl IntoView {
  let cache_ids_resource = Resource::new(|| (), |_| enumerate_cache_ids());

  let fallback = move || "Loading...".into_view();

  let cache_ids_reader = move || {
    Suspend::new(async move {
      match cache_ids_resource.await {
        Ok(ids) => Either::Left(view! {
          <span>"Available models: "{ ids.len() }</span>
        }),
        Err(e) => Either::Right(view! {
          <span>"Error: "{format!("{e}")}</span>
        }),
      }
    })
  };

  view! {
    <div class="flex flex-col gap-4">
      <PageTitle title="Cache Model".to_string() level=1 />
      <div class="text-lg text-content2">
        <p>"This page will display the cache model."</p>
        <p>
          <Suspense fallback=fallback>
            { cache_ids_reader }
          </Suspense>
        </p>
      </div>
    </div>
  }
}

#[component]
fn SideBar() -> impl IntoView {
  view! {
    <div class="flex-none self-stretch bg-backgroundSecondary w-60 p-6 pt-8 flex flex-col gap-2 border-r border-gray-6">
      <a href="/" class="link text-2xl font-semibold tracking-tight">"Cartographer"</a>
      <div class="flex flex-col gap-1 pl-2">
        <p class="text-content2">"Models"</p>
        <ul class="flex flex-col gap-1 pl-2">
          <li><a href="/model/cache" class="link link-underline">"Cache"</a></li>
        </ul>
      </div>
    </div>
  }
}
