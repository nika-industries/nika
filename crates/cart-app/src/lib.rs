//! The leptos app crate for the Cartographer app.

use leptos::prelude::*;
use leptos_meta::{
  provide_meta_context, Link, MetaTags, Style, Stylesheet, Title,
};
use leptos_router::{
  components::{Route, Router, Routes},
  StaticSegment,
};

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

    <div class="h-screen flex justify-items-start items-start gap-4">
      <div class="flex-none self-stretch bg-backgroundSecondary w-60 p-6 pt-8 flex flex-col gap-1 border-r border-gray-6">
        <a href="/" class="link text-2xl font-semibold tracking-tight">"Cartographer"</a>
        <div class="flex flex-col gap-1 pl-2">
          <p class="text-content2">"Models"</p>
          <div class="flex flex-col gap-1 pl-2">
            <a href="/model/cache" class="link">"Cache"</a>
            <a href="/model/entry" class="link">"Entry"</a>
            <a href="/model/store" class="link">"Store"</a>
            <a href="/model/token" class="link">"Token"</a>
            // <a href="/model/org" class="link">"Org"</a>
            // <a href="/model/user" class="link">"User"</a>
          </div>
        </div>
      </div>
      <div class="container py-8">
        <Router>
          <main>
            <Routes fallback=|| "Page not found.".into_view()>
              <Route path=StaticSegment("") view=HomePage/>
            </Routes>
          </main>
        </Router>
      </div>
    </div>
  }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
  view! {
    <div class="flex flex-col gap-4">
      <p class="text-4xl font-semibold tracking-tight">"Welcome to Cartographer"</p>
      <p class="text-lg text-content2">"Choose a page on the left to start exploring data."</p>
    </div>
  }
}
