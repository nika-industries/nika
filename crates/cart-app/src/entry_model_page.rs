use leptos::{either::Either, prelude::*};

use crate::utils::{ItemList, KeyValue, PageTitle};

#[server]
async fn fetch_entries() -> Result<Vec<models::Entry>, ServerFnError> {
  let entry_service: Option<_> = use_context();
  let entry_service: prime_domain::DynEntryService = entry_service
    .ok_or(ServerFnError::new("Entry service is not available."))?;

  let ids = entry_service.enumerate_models().await.map_err(|e| {
    ServerFnError::new(format!("Failed to enumerate entry models: {}", e))
  })?;
  Ok(ids)
}

#[component]
fn Entry(#[prop(into)] entry: MaybeSignal<models::Entry>) -> impl IntoView {
  let entry = Signal::derive(move || entry.get());

  let entry_id = move || entry.with(|e| e.id.to_string());
  let entry_page_url = move || format!("/model/entry/{}", entry_id());

  let entry_path = move || entry.with(|e| e.path.to_string());
  let entry_size = move || entry.with(|e| e.size.to_string());
  let entry_cache = move || entry.with(|e| e.cache.to_string());
  let entry_cache_url = move || format!("/model/cache/{}", entry_cache());

  view! {
    <div class="w-full max-w-3xl p-4 flex flex-col gap-2 bg-gray-2 border border-gray-6 rounded-lg shadow">
      <div class="flex flex-row gap-4 items-center">
        <span class="dot dot-success" />
        <a href={entry_page_url} class="font-semibold tracking-tight text-2xl link link-underline">
          { entry_path }
        </a>
      </div>
      <div class="flex flex-row gap-x-2 flex-wrap items-center">
        <KeyValue key="ID:"> { entry_id } </KeyValue>
        <KeyValue key="Path:"> { entry_path } </KeyValue>
        <KeyValue key="Size:"> { entry_size } </KeyValue>
        <KeyValue key="Cache:">
          <a href={entry_cache_url} class="text-sm link link-underline">
            { entry_cache }
          </a>
        </KeyValue>
      </div>
    </div>
  }
}

#[component]
pub fn EntryModelListPage() -> impl IntoView {
  let entries_resource = Resource::new(|| (), |_| fetch_entries());

  let fallback = move || "Loading...".into_view();

  let entries_reader = move || {
    Suspend::new(async move {
      match entries_resource.await {
        Ok(entries) => {
          let entries =
            entries.into_iter().map(|c| view! { <Entry entry=c /> });
          Either::Left(view! {
            <ItemList items=entries />
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
      <PageTitle level=1>"Entry Model"</PageTitle>
      <p class="text-lg text-content2">"See the entries present in the database below."</p>
      <Suspense fallback=fallback>
        { entries_reader }
      </Suspense>
    </div>
  }
}
