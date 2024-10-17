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
    <span class="font-mono text-sm">
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
