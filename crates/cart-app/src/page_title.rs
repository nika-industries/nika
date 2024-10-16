use leptos::prelude::*;

#[component]
pub fn PageTitle(
  #[prop(into)] title: MaybeSignal<
    impl AsRef<str> + Clone + Send + Sync + 'static,
  >,
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

  let title_reader = move || title.with(|t| t.as_ref().to_string());

  view! {
    <p class=class>{ title_reader }</p>
  }
}
