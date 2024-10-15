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
    // injects a stylesheet into the document <head>
    // id=leptos means cargo-leptos will hot-reload this stylesheet
    <Stylesheet id="leptos" href="/pkg/cart.css"/>
    <Style>{include_str!("../style/fonts.css")}</Style>
    <Link rel="preload" href="/fonts/inter.ttf" as_="font" type_="font/ttf" crossorigin="anonymous" />

    // sets the document title
    <Title text="Welcome to Leptos"/>

    // content for this welcome page
    <Router>
      <main>
        <Routes fallback=|| "Page not found.".into_view()>
          <Route path=StaticSegment("") view=HomePage/>
        </Routes>
      </main>
    </Router>
  }
}

/// Renders the home page of your application.
#[island]
fn HomePage() -> impl IntoView {
  // Creates a reactive value to update the button
  let count = RwSignal::new(0);
  let on_click = move |_| *count.write() += 1;

  view! {
    <div class="h-screen flex justify-items-start items-start gap-4">
      <div class="flex-none self-stretch w-60 bg-backgroundSecondary">

      </div>
      <div class="container py-8">
        <div class="flex flex-col gap-4 items-start">
          <p class="text-2xl font-semibold tracking-tight">"Welcome to Leptos!"</p>
          <button class="btn" on:click=on_click>"Click Me: " {count}</button>
        </div>
      </div>
    </div>
  }
}
