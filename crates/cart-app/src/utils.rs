use leptos::prelude::*;

#[component]
pub fn PageTitle(
  children: Children,
  #[prop(default = 1)] level: i32,
) -> impl IntoView {
  let size_class = match level {
    1 => "text-4xl",
    2 => "text-3xl",
    3 => "text-2xl",
    4 => "text-xl",
    _ => "text-lg",
  };
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
  children: Children,
) -> impl IntoView {
  view! {
    <a href={href} class="font-semibold tracking-tight text-2xl link link-underline">
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
  view! {
    <div class="w-full max-w-3xl p-4 flex flex-col gap-2 bg-gray-2 border border-gray-6 rounded-lg shadow">
      { children() }
    </div>
  }
}
