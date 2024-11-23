use std::str::FromStr;

use leptos::{either::Either, prelude::*};

use crate::{fetchers::*, utils::*};

#[component]
fn Token(#[prop(into)] token: Signal<models::Token>) -> impl IntoView {
  let token_id = Signal::derive(move || token.with(|t| t.id));

  let token_nickname =
    Signal::derive(move || token.with(|t| t.nickname.clone()));
  let token_secret = move || token.with(|t| t.secret.to_string());
  let token_perms = move || token.with(|t| format!("{:#?}", t.perms));

  view! {
    <Card>
      <TitleRow>
        <SuccessDot />
        <TokenIdTitleLink id=token_id />
      </TitleRow>
      <PropList>
        <KeyValue key="ID:">
          <TokenIdLink id=token_id />
        </KeyValue>
        <KeyValue key="Nickname:">
          <EntityNickname nickname=token_nickname />
        </KeyValue>
        <KeyValue key="Secret:"> { token_secret } </KeyValue>
      </PropList>
      <div class="flex flex-row gap-2 items-start">
        <BoxHighlight> "Config:" </BoxHighlight>
        <CodeBlock> { token_perms } </CodeBlock>
      </div>
    </Card>
  }
}

#[component]
pub fn TokenModelListPage() -> impl IntoView {
  let tokens_resource = Resource::new(|| (), |_| fetch_all_tokens());

  let tokens_reader = move || {
    Suspend::new(async move {
      match tokens_resource.await {
        Ok(tokens) => {
          let tokens = tokens.into_iter().map(|c| view! { <Token token=c /> });
          Either::Left(view! {
            <ItemList items=tokens />
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
      <PageTitle level=1>"Token Model"</PageTitle>
      <p class="text-lg text-content2">"See the tokens present in the database below."</p>
      <Suspense fallback=crate::fallback>
        { tokens_reader }
      </Suspense>
    </PageWrapper>
  }
}

#[component]
pub fn TokenModelSinglePage() -> impl IntoView {
  let params = leptos_router::hooks::use_params_map();
  let id_param = params().get("id").unwrap_or_default();

  let token_id = match models::TokenRecordId::from_str(&id_param) {
    Ok(id) => id,
    Err(e) => {
      return Either::Left(view! {
        <div class="flex flex-col gap-4">
          <PageTitle level=1>"Token: Invalid ID"</PageTitle>
          <p class="text-lg text-content2">"Invalid token ID: " { e.to_string() }</p>
        </div>
      })
    }
  };

  let token_resource = Resource::new(move || token_id, fetch_token);

  let token_reader = move || {
    Suspend::new(async move {
      match token_resource.await {
        Ok(token) => Either::Left(view! {
          <Token token=token />
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
        "Token: "
        <CodeHighlight>{ token_id.to_string() }</CodeHighlight>
      </PageTitle>
      <Suspense fallback=crate::fallback>
        { token_reader }
      </Suspense>
    </PageWrapper>
  })
}
