use leptos::prelude::*;

#[component]
fn CodeBlock(children: Children) -> impl IntoView {
  let class = "bg-gray-2 p-4 rounded-md border border-gray-6 text-sm \
               text-content2 overflow-auto max-w-full whitespace-pre-wrap";

  view! {
    <pre class=class>
      <code>{ children() }</code>
    </pre>
  }
}
