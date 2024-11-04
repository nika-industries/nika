use std::str::FromStr;

use leptos::{either::Either, prelude::*};

use crate::{fetchers::*, utils::*};

#[component]
fn Entry(#[prop(into)] entry: MaybeSignal<models::Entry>) -> impl IntoView {
  let entry = Signal::derive(move || entry.get());

  let entry_id = Signal::derive(move || entry.with(|e| e.id));

  let entry_path = move || entry.with(|e| format!("{:?}", e.path.to_string()));
  let entry_c_status = move || entry.with(|e| format!("{:?}", e.c_status));
  let entry_cache = Signal::derive(move || entry.with(|e| e.cache));

  view! {
    <Card>
      <TitleRow>
        <SuccessDot />
        <EntryIdTitleLink id=entry_id />
      </TitleRow>
      <PropList>
        <KeyValue key="ID:"><EntryIdLink id=entry_id /></KeyValue>
        <KeyValue key="Path:"> { entry_path } </KeyValue>
        <KeyValue key="C-Status:"> { entry_c_status } </KeyValue>
        <KeyValue key="Cache:">
          <CacheIdLink id=entry_cache />
        </KeyValue>
      </PropList>
    </Card>
  }
}

#[component]
pub fn EntryModelListPage() -> impl IntoView {
  let entries_resource = Resource::new(|| (), |_| fetch_all_entries());

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
    <PageWrapper>
      <PageTitle level=1>"Entry Model"</PageTitle>
      <p class="text-lg text-content2">"See the entries present in the database below."</p>
      <Suspense fallback=crate::fallback>
        { entries_reader }
      </Suspense>
    </PageWrapper>
  }
}

#[component]
pub fn EntryModelSinglePage() -> impl IntoView {
  let params = leptos_router::hooks::use_params_map();
  let id_param = params().get("id").unwrap_or_default();

  let entry_id = match models::EntryRecordId::from_str(&id_param) {
    Ok(id) => id,
    Err(e) => {
      return Either::Left(view! {
        <div class="flex flex-col gap-4">
          <PageTitle level=1>"Entry: Invalid ID"</PageTitle>
          <p class="text-lg text-content2">"Invalid entry ID: " { e.to_string() }</p>
        </div>
      })
    }
  };

  let entry_resource = Resource::new(move || entry_id, fetch_entry);

  let entry_reader = move || {
    Suspend::new(async move {
      match entry_resource.await {
        Ok(entry) => Either::Left(view! {
          <Entry entry=entry />
        }),
        Err(e) => Either::Right(view! {
          <p class="text-lg text-content2">"Error: "{format!("{e}")}</p>
        }),
      }
    })
  };

  Either::Right(view! {
    <PageWrapper>
      <PageTitle level=1>
        "Entry: "
        <CodeHighlight>{ entry_id.to_string() }</CodeHighlight>
      </PageTitle>
      <Suspense fallback=crate::fallback>
        { entry_reader }
      </Suspense>
    </PageWrapper>
  })
}
