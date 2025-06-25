use crate::i18n::*;
use leptos::*;

#[component]
pub fn Page() -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <h1 class="text-5xl">"akosnad.dev"</h1>
        <span class="text-xl italic">{t!(i18n, under_development)}</span>
        <p class="py-8">{t!(i18n, greeting)}</p>
    }
}
