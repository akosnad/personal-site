use crate::i18n::*;
use leptos::*;
use leptos_animation::*;
use leptos_meta::*;
use leptos_router::*;

mod home;

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
            // sets the document title
            <Title text="akosnad.dev" />
            <Backdrop />

            // content for this welcome page
            <Router>
                <main class="container mx-auto my-auto px-8 py-8 h-screen flex flex-col items-center justify-center">
                    <Routes>
                        <Route path="" view=HomePage />
                        <Route path="/home" view=home::Home />
                        <Route path="/*any" view=NotFound />
                    </Routes>
                </main>
            </Router>
        </I18nContextProvider>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <h1 class="text-5xl">"akosnad.dev"</h1>
        <span class="text-xl italic">{t!(i18n, under_development)}</span>
        <p class="py-8">{t!(i18n, greeting)}</p>
    }
}

#[component]
fn Backdrop() -> impl IntoView {
    view! {
        <div class="backdrop overflow-hidden">
            <div class="halftone">
                <div id="background">
                    <div id="bg-breathe" />
                    <img src="/assets/bg.jpg" onload="this.style.opacity=1" class="easeload"/>
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
        <h1 class="text-5xl">"Not Found"</h1>
        <a class="py-8" href="/">go back home</a>
    }
}
