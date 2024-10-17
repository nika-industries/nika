use leptos::{either::Either, prelude::*};

use crate::page_title::PageTitle;

#[server]
async fn fetch_caches() -> Result<Vec<models::Cache>, ServerFnError> {
  let cache_service: Option<_> = use_context();
  let cache_service: prime_domain::DynCacheService = cache_service
    .ok_or(ServerFnError::new("Cache service is not available."))?;

  let ids = cache_service.enumerate_models().await.map_err(|e| {
    ServerFnError::new(format!("Failed to enumerate cache models: {}", e))
  })?;
  Ok(ids)
}

#[component]
fn CacheList(caches: impl IntoIterator<Item = models::Cache>) -> impl IntoView {
  view! {
    <ul class="flex flex-col gap-2">
      { caches.into_iter().map(|cache| view! {
        <li>
          <Cache cache=cache/>
        </li>
      }).collect_view() }
    </ul>
  }
}

#[component]
fn Cache(#[prop(into)] cache: MaybeSignal<models::Cache>) -> impl IntoView {
  let cache = Signal::derive(move || cache.get());
  let cache_name = move || cache.with(|c| c.name.to_string());
  let cache_id = move || cache.with(|c| c.id.to_string());
  let cache_page_url = move || format!("/model/cache/{}", cache_id());
  let cache_visibility = move || cache.with(|c| c.visibility.to_string());
  let cache_store = move || cache.with(|c| c.store.to_string());
  let cache_store_url = move || format!("/model/store/{}", cache_store());

  view! {
    <div class="w-full max-w-3xl p-4 flex flex-col gap-2 bg-gray-2 border border-gray-6 rounded-lg shadow">
      <a href={cache_page_url} class="font-semibold tracking-tight text-2xl link link-underline">
        { cache_name }
      </a>
      <div class="flex flex-row gap-x-2 flex-wrap">
        <span><span class="text-content2">"ID: "</span>{ cache_id }</span>
        <span><span class="text-content2">"Visibility: "</span>{ cache_visibility }</span>
        <span><span class="text-content2">"Store: "</span><a href={cache_store_url} class="link link-underline">{ cache_store }</a></span>
      </div>
    </div>
  }
}

#[component]
pub fn CacheModelListPage() -> impl IntoView {
  let caches_resource = Resource::new(|| (), |_| fetch_caches());

  let fallback = move || "Loading...".into_view();

  let caches_reader = move || {
    Suspend::new(async move {
      match caches_resource.await {
        Ok(caches) => Either::Left(view! {
          <CacheList caches=caches/>
        }),
        Err(e) => Either::Right(view! {
          <span>"Error: "{format!("{e}")}</span>
        }),
      }
    })
  };

  view! {
    <div class="flex flex-col gap-4">
      <PageTitle level=1>"Cache Model"</PageTitle>
      <p class="text-lg text-content2">"See the caches present in the database below."</p>
      <Suspense fallback=fallback>
        { caches_reader }
      </Suspense>
    </div>
  }
}
