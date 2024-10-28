use leptos::prelude::*;

fn level_to_title_size_class(level: i32) -> &'static str {
  match level {
    1 => "text-4xl",
    2 => "text-3xl",
    3 => "text-2xl",
    4 => "text-xl",
    _ => "text-lg",
  }
}

#[component]
pub fn PageTitle(
  children: Children,
  #[prop(default = 1)] level: i32,
) -> impl IntoView {
  let size_class = level_to_title_size_class(level);
  let class = format!("font-semibold tracking-tight {size_class}");

  view! {
    <p class=class>{ children() }</p>
  }
}

#[component]
pub fn BoxHighlight(children: Children) -> impl IntoView {
  view! {
    <span class="text-content2 py-0.5 px-1 bg-gray-3 rounded border border-gray-6">
      { children() }
    </span>
  }
}

#[component]
pub fn CodeHighlight(children: Children) -> impl IntoView {
  view! {
    <span class="font-mono">
      { children() }
    </span>
  }
}

#[component]
pub fn ItemList(
  items: impl IntoIterator<Item = impl IntoView>,
) -> impl IntoView {
  view! {
    <ul class="flex flex-col gap-2">
      { items.into_iter().map(|item| view! {
        <li>
          { item }
        </li>
      }).collect_view() }
    </ul>
  }
}

#[component]
pub fn KeyValue(
  key: impl AsRef<str> + Send + Sync,
  children: Children,
) -> impl IntoView {
  let key = key.as_ref().to_string();

  view! {
    <span>
      <BoxHighlight> { key } </BoxHighlight>
      " "
      <CodeHighlight> { children() } </CodeHighlight>
    </span>
  }
}

#[component]
pub fn CodeBlock(children: Children) -> impl IntoView {
  let class = "bg-gray-2 p-4 rounded-md border border-gray-6 text-sm \
               text-content2 overflow-auto max-w-full whitespace-pre-wrap";

  view! {
    <pre class=class>
      <code>{ children() }</code>
    </pre>
  }
}

#[component]
pub fn PropList(children: Children) -> impl IntoView {
  view! {
    <div class="flex flex-row gap-x-2 flex-wrap items-center">
      { children() }
    </div>
  }
}

#[component]
pub fn Link(
  #[prop(into)] href: MaybeSignal<String>,
  children: Children,
) -> impl IntoView {
  view! {
    <a href={href} class="link link-underline">
      { children() }
    </a>
  }
}

#[component]
pub fn TitleLink(
  #[prop(into)] href: MaybeSignal<String>,
  #[prop(default = 1)] level: i32,
  children: Children,
) -> impl IntoView {
  let size_class = level_to_title_size_class(level);
  let class =
    format!("font-semibold tracking-tight {size_class} link link-underline");

  view! {
    <a href={href} class=class>
      { children() }
    </a>
  }
}

#[component]
pub fn TitleRow(children: Children) -> impl IntoView {
  view! {
    <div class="flex flex-row gap-4 items-center">
      { children() }
    </div>
  }
}

#[component]
pub fn SuccessDot() -> impl IntoView {
  view! {
    <span class="dot dot-success" />
  }
}

#[component]
pub fn Card(children: Children) -> impl IntoView {
  let class = "w-full max-w-3xl min-h-32 p-4 flex flex-col gap-2 bg-gray-2 \
               border border-gray-6 rounded-lg shadow animate-fade-in";
  view! {
    <div class=class>
      { children() }
    </div>
  }
}

// #[component]
// pub fn SkeletonCard() -> impl IntoView {
//   view! {
//     <div class="w-full max-w-3xl min-h-32 skeleton rounded-lg" />
//   }
// }

#[component]
pub fn PageWrapper(children: Children) -> impl IntoView {
  view! {
    <div class="flex flex-col gap-4 animate-fade-in">
      { children() }
    </div>
  }
}

macro_rules! id_component_and_link {
  ($component:ident, $component_link:ident, $component_title_link:ident, $record:ty, $route:expr) => {
    #[component]
    pub fn $component(#[prop(into)] id: MaybeSignal<$record>) -> impl IntoView {
      view! {
        { move || id().to_string() }
      }
    }

    #[component]
    pub fn $component_link(
      #[prop(into)] id: MaybeSignal<$record>,
    ) -> impl IntoView {
      let id = Signal::derive(id);
      let id_url =
        Signal::derive(move || format!("/model/{}/{}", $route, id()));

      view! {
        <Link href={id_url}>
          <$component id=id />
        </Link>
      }
    }

    #[component]
    pub fn $component_title_link(
      #[prop(into)] id: MaybeSignal<$record>,
      #[prop(default = 2)] level: i32,
    ) -> impl IntoView {
      let id = Signal::derive(id);
      let id_url =
        Signal::derive(move || format!("/model/{}/{}", $route, id()));

      view! {
        <TitleLink href={id_url} level=level>
          <$component id=id />
        </TitleLink>
      }
    }
  };
}

id_component_and_link!(
  StoreId,
  StoreIdLink,
  StoreIdTitleLink,
  models::StoreRecordId,
  "store"
);
id_component_and_link!(
  CacheId,
  CacheIdLink,
  CacheIdTitleLink,
  models::CacheRecordId,
  "cache"
);
id_component_and_link!(
  EntryId,
  EntryIdLink,
  EntryIdTitleLink,
  models::EntryRecordId,
  "entry"
);
id_component_and_link!(
  TokenId,
  TokenIdLink,
  TokenIdTitleLink,
  models::TokenRecordId,
  "token"
);

#[component]
pub fn Visibility(
  #[prop(into)] vis: MaybeSignal<models::Visibility>,
) -> impl IntoView {
  view! {
    { move || vis().to_string() }
  }
}

#[component]
pub fn EntityName(
  #[prop(into)] name: MaybeSignal<models::EntityName>,
) -> impl IntoView {
  view! {
    { move || name().to_string() }
  }
}

#[component]
pub fn EntityNickname(
  #[prop(into)] nickname: MaybeSignal<models::EntityNickname>,
) -> impl IntoView {
  view! {
    { move || nickname().to_string() }
  }
}
