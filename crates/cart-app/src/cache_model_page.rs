use leptos::{either::Either, prelude::*};

use crate::utils::{ItemList, KeyValue, PageTitle};

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
fn Cache(#[prop(into)] cache: MaybeSignal<models::Cache>) -> impl IntoView {
  let cache = Signal::derive(move || cache.get());

  let cache_id = move || cache.with(|c| c.id.to_string());
  let cache_page_url = move || format!("/model/cache/{}", cache_id());

  let cache_name = move || cache.with(|c| c.name.to_string());
  let cache_visibility = move || cache.with(|c| c.visibility.to_string());
  let cache_store = move || cache.with(|c| c.store.to_string());
  let cache_store_url = move || format!("/model/store/{}", cache_store());

  view! {
    <div class="w-full max-w-3xl p-4 flex flex-col gap-2 bg-gray-2 border border-gray-6 rounded-lg shadow">
      <div class="flex flex-row gap-4 items-center">
        <span class="dot dot-success" />
        <a href={cache_page_url} class="font-semibold tracking-tight text-2xl link link-underline">
          { cache_name }
        </a>
      </div>
      <div class="flex flex-row gap-x-2 flex-wrap items-center">
        <KeyValue key="ID:"> { cache_id } </KeyValue>
        <KeyValue key="Visibility:"> { cache_visibility } </KeyValue>
        <KeyValue key="Store:">
          <a href={cache_store_url} class="text-sm link link-underline">
            { cache_store }
          </a>
        </KeyValue>
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
        Ok(caches) => {
          let caches = caches.into_iter().map(|c| view! { <Cache cache=c /> });
          Either::Left(view! {
            <ItemList items=caches />
          })
        }
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
