use leptos::prelude::*;

use crate::utils::PageTitle;

#[component]
pub fn HomePage() -> impl IntoView {
  view! {
    <div class="flex flex-col gap-4">
      <PageTitle>"Welcome to Cartographer"</PageTitle>
      <p class="text-lg text-content2">"Choose a page on the left to start exploring data."</p>
    </div>
  }
}
