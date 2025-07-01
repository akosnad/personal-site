use crate::i18n::*;
use leptos::prelude::*;

#[component]
pub fn Page() -> impl IntoView {
    let i18n = use_i18n();

    view! { <p>{t!(i18n, soon)}</p> }
}
