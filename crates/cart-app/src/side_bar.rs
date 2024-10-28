use leptos::prelude::*;

#[component]
pub fn SideBar() -> impl IntoView {
  view! {
    <div class="flex-none self-stretch bg-backgroundSecondary w-60 p-6 pt-8 flex flex-col gap-2 border-r border-gray-6">
      <a href="/" class="link link-underline text-2xl font-semibold tracking-tight">
        "Cartographer"
      </a>
      <div class="flex flex-col gap-1 pl-2">
        <p class="text-content2">"Models"</p>
        <ul class="flex flex-col gap-1 pl-2">
          <li><a href="/model/cache" class="link link-underline">"Cache"</a></li>
          <li><a href="/model/entry" class="link link-underline">"Entry"</a></li>
          <li><a href="/model/store" class="link link-underline">"Store"</a></li>
          <li><a href="/model/token" class="link link-underline">"Token"</a></li>
        </ul>
      </div>
    </div>
  }
}
