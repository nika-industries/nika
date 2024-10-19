use std::str::FromStr;

use leptos::{either::Either, prelude::*};

use crate::{fetchers::*, utils::*};

#[component]
fn Store(#[prop(into)] store: MaybeSignal<models::Store>) -> impl IntoView {
  let store = Signal::derive(move || store.get());

  let store_id = move || store.with(|c| c.id.to_string());
  let store_page_url = move || format!("/model/store/{}", store_id());
  let store_config = move || store.with(|c| format!("{:#?}", c.config));

  let store_nickname = move || store.with(|c| c.nickname.to_string());

  view! {
    <Card>
      <TitleRow>
        <SuccessDot />
        <a href={store_page_url} class="font-semibold tracking-tight text-2xl link link-underline">
          { store_nickname }
        </a>
      </TitleRow>
      <PropList>
        <KeyValue key="ID:"> { store_id } </KeyValue>
      </PropList>
      <div class="flex flex-row gap-2 items-start">
        <BoxHighlight> "Config:" </BoxHighlight>
        <CodeBlock> { store_config } </CodeBlock>
      </div>
    </Card>
  }
}

#[component]
pub fn StoreModelListPage() -> impl IntoView {
  let stores_resource = Resource::new(|| (), |_| fetch_all_stores());

  let fallback = move || "Loading...".into_view();

  let stores_reader = move || {
    Suspend::new(async move {
      match stores_resource.await {
        Ok(stores) => {
          let stores = stores.into_iter().map(|c| view! { <Store store=c /> });
          Either::Left(view! {
            <ItemList items=stores />
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
      <PageTitle level=1>"Store Model"</PageTitle>
      <p class="text-lg text-content2">"See the stores present in the database below."</p>
      <Suspense fallback=fallback>
        { stores_reader }
      </Suspense>
    </div>
  }
}

#[component]
pub fn StoreModelSinglePage() -> impl IntoView {
  let params = leptos_router::hooks::use_params_map();
  let id_param = params().get("id").unwrap_or_default();

  let store_id = match models::StoreRecordId::from_str(&id_param) {
    Ok(id) => id,
    Err(e) => {
      return Either::Left(view! {
        <div class="flex flex-col gap-4">
          <PageTitle level=1>"Store: Invalid ID"</PageTitle>
          <p class="text-lg text-content2">"Invalid store ID: " { e.to_string() }</p>
        </div>
      })
    }
  };

  let store_resource = Resource::new(move || store_id, fetch_store);

  let fallback = move || "Loading...".into_view();

  let store_reader = move || {
    Suspend::new(async move {
      match store_resource.await {
        Ok(store) => Either::Left(view! {
          <Store store=store />
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
        "Store: "
        <CodeHighlight>{ store_id.to_string() }</CodeHighlight>
      </PageTitle>
      <Suspense fallback=fallback>
        { store_reader }
      </Suspense>
    </div>
  })
}
