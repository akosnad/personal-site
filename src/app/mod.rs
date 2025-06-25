use crate::i18n::*;
use leptos::*;
use leptos_animation::*;
use leptos_meta::*;
use leptos_router::*;

mod about;
mod home;
mod posts;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    AnimationContext::provide();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/personal-site.css" />

        <I18nContextProvider>
            <Title text="akosnad.dev" />
            <Backdrop />

            <NavBar />

            <Router>
                <main class="container mx-auto my-auto px-8 py-8 h-screen flex flex-col items-center justify-center">
                    <Routes>
                        <Route path="" view=home::Page />
                        <Route path="/posts" view=posts::Page />
                        <Route path="/about" view=about::Page />
                        <Route path="/*any" view=NotFound />
                    </Routes>
                </main>
            </Router>
        </I18nContextProvider>
    }
}

#[component]
fn NavBar() -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <nav class="fixed top-0 left-0 right-0 container flex px-4 py-4 flex-row gap-6">
            <Link class="font-bold" href="/">
                "akosnad.dev"
            </Link>
            <Link href="/posts">{t!(i18n, posts)}</Link>
            <Link href="/about">{t!(i18n, about)}</Link>

            <button class="mx-auto">{}</button>
        </nav>
    }
}

#[component]
fn Link(
    #[prop(into)] href: String,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let class = format!(
        "{} hover:-translate-y-0.5 underline hover:decoration-4 transition-all",
        class
    );
    view! {
        <a class=class href=href>
            {children()}
        </a>
    }
}

#[component]
fn Backdrop() -> impl IntoView {
    view! {
        <div class="backdrop overflow-hidden">
            <div class="halftone">
                <div id="background">
                    <div id="bg-breathe" />
                    <img src="/assets/bg.jpg" onload="this.style.opacity=1" class="easeload" />
                </div>
            </div>
            <div id="background-color" />
        </div>
    }
}

/// 404 - Not Found
#[component]
fn NotFound() -> impl IntoView {
    // set an HTTP status code 404
    // this is feature gated because it can only be done during
    // initial server-side rendering
    // if you navigate to the 404 page subsequently, the status
    // code will not be set because there is not a new HTTP request
    // to the server
    #[cfg(feature = "ssr")]
    {
        // this can be done inline because it's synchronous
        // if it were async, we'd use a server function
        let resp = expect_context::<leptos_actix::ResponseOptions>();
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    view! {
        <h1 class="text-5xl">
            <span class="glitch" data-text="Not Found">
                "Not Found"
            </span>
        </h1>
    }
}
