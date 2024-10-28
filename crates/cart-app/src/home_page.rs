use leptos::prelude::*;

use crate::utils::*;

#[component]
pub fn HomePage() -> impl IntoView {
  view! {
    <PageWrapper>
      <PageTitle>"Welcome to Cartographer"</PageTitle>
      <p class="text-lg text-content2">"Choose a page on the left to start exploring data."</p>
    </PageWrapper>
  }
}
