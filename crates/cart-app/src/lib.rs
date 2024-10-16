//! The leptos app crate for the Cartographer app.

mod page_title;

use leptos::prelude::*;
use leptos_meta::{
  provide_meta_context, Link, MetaTags, Style, Stylesheet, Title,
};
use leptos_router::{
  components::{Route, Router, Routes},
  path,
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
              // <Route path=path!("/model/cache") view=CacheModelPage/>
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

#[component]
fn CacheModelPage() -> impl IntoView {
  view! {
    <div class="flex flex-col gap-4">
      <PageTitle title="Cache Model".to_string() level=1 />
      <p class="text-lg text-content2">"This page will display the cache model."</p>
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
