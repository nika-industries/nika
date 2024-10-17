//! The leptos app crate for the Cartographer app.

mod cache_model_page;
mod code_block;
mod home_page;
mod page_title;
mod side_bar;

use leptos::prelude::*;
use leptos_meta::{
  provide_meta_context, Link, MetaTags, Style, Stylesheet, Title,
};
use leptos_router::{
  components::{Route, Router, Routes},
  path, SsrMode,
};

use self::{
  cache_model_page::CacheModelListPage, home_page::HomePage, side_bar::SideBar,
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

    <Router>
      <div class="h-screen flex justify-items-start items-start gap-4">
        <SideBar/>
        <div class="container mx-4 py-8">
          <main>
            <Routes fallback=|| "Page not found.".into_view()>
              <Route path=path!("") view=HomePage/>
              <Route path=path!("/model/cache") view=CacheModelListPage ssr=SsrMode::OutOfOrder/>
              <Route path=path!("/model/cache/:id") view=CacheModelListPage/>
            </Routes>
          </main>
        </div>
      </div>
    </Router>
  }
}
