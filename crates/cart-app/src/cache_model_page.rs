use std::str::FromStr;

use leptos::{either::Either, prelude::*};

use crate::{fetchers::*, utils::*};

#[component]
fn Cache(#[prop(into)] cache: MaybeSignal<models::Cache>) -> impl IntoView {
  let cache = Signal::derive(move || cache.get());

  let cache_id = move || cache.with(|c| c.id.to_string());
  let cache_page_url =
    Signal::derive(move || format!("/model/cache/{}", cache_id()));

  let cache_name = move || cache.with(|c| c.name.to_string());
  let cache_visibility = move || cache.with(|c| c.visibility.to_string());
  let cache_store = move || cache.with(|c| c.store.to_string());
  let cache_store_url =
    Signal::derive(move || format!("/model/store/{}", cache_store()));

  view! {
    <Card>
      <TitleRow>
        <SuccessDot />
        <a href={cache_page_url} class="font-semibold tracking-tight text-2xl link link-underline">
          { cache_name }
        </a>
      </TitleRow>
      <PropList>
        <KeyValue key="ID:"> { cache_id } </KeyValue>
        <KeyValue key="Visibility:"> { cache_visibility } </KeyValue>
        <KeyValue key="Store:">
          <Link href={cache_store_url}>
            { cache_store }
          </Link>
        </KeyValue>
      </PropList>
    </Card>
  }
}

#[component]
pub fn CacheModelListPage() -> impl IntoView {
  let caches_resource = Resource::new_blocking(|| (), |_| fetch_all_caches());

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

#[component]
pub fn CacheModelSinglePage() -> impl IntoView {
  let params = leptos_router::hooks::use_params_map();
  let id_param = params().get("id").unwrap_or_default();

  let cache_id = match models::CacheRecordId::from_str(&id_param) {
    Ok(id) => id,
    Err(e) => {
      return Either::Left(view! {
        <div class="flex flex-col gap-4">
          <PageTitle level=1>"Cache: Invalid ID"</PageTitle>
          <p class="text-lg text-content2">"Invalid cache ID: " { e.to_string() }</p>
        </div>
      })
    }
  };

  let cache_resource = Resource::new_blocking(move || cache_id, fetch_cache);

  let fallback = move || "Loading...".into_view();

  let cache_reader = move || {
    Suspend::new(async move {
      match cache_resource.await {
        Ok(cache) => Either::Left(view! {
          <Cache cache=cache />
        }),
        Err(e) => Either::Right(view! {
          <p class="text-lg text-content2">"Error: "{format!("{e}")}</p>
        }),
      }
    })
  };

  Either::Right(view! {
    <div class="flex flex-col gap-4">
      <PageTitle level=1>
        "Cache: "
        <CodeHighlight>{ cache_id.to_string() }</CodeHighlight>
      </PageTitle>
      <Suspense fallback=fallback>
        { cache_reader }
      </Suspense>
    </div>
  })
}
