use crate::i18n::*;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_meta::{Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    hooks::use_location,
    path,
};
use leptos_use::{
    BreakpointsTailwind, breakpoints_tailwind, use_breakpoints, use_cookie,
    use_prefers_reduced_motion,
};

mod about;
mod home;
mod posts;

#[derive(Clone, Debug)]
struct BackdropProvider {
    is_screen_lg_or_larger: Signal<bool>,
    is_reduced_motion_preferred: Signal<bool>,
    animation_user_override: Signal<Option<bool>>,
    animation_user_override_set: WriteSignal<Option<bool>>,
}
impl Default for BackdropProvider {
    fn default() -> Self {
        let screen_width = use_breakpoints(breakpoints_tailwind());
        let is_screen_lg_or_larger = screen_width.ge(BreakpointsTailwind::Lg);

        let is_reduced_motion_preferred = use_prefers_reduced_motion();

        let (animation_user_override, animation_user_override_set) =
            use_cookie::<bool, codee::string::FromToStringCodec>("animations");

        Self {
            is_screen_lg_or_larger,
            is_reduced_motion_preferred,
            animation_user_override,
            animation_user_override_set,
        }
    }
}
impl BackdropProvider {
    fn animation_enabled(&self) -> bool {
        if self.is_reduced_motion_preferred.get() {
            return false;
        }

        if let Some(forced_value) = self.animation_user_override.get() {
            forced_value
        } else {
            self.is_screen_lg_or_larger.get()
        }
    }

    fn toggle_animation(&self) {
        self.animation_user_override_set.update(|w| {
            *w = match *w {
                Some(val) => Some(!val),
                None => Some(true),
            }
        });
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    provide_context(BackdropProvider::default());

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/personal-site.css" />

        <I18nContextProvider>
            <Title text="akosnad.dev" />
            <Backdrop />

            <Router>
                <NavBar />

                <main class="container mx-auto my-auto px-8 py-8 h-screen flex flex-col items-center justify-center">
                    <Routes fallback=move || "Not Found">
                        <Route path=path!("") view=home::Page />
                        <Route path=path!("/posts") view=posts::PostsList />
                        <Route
                            path=path!("/posts/:id")
                            view=posts::PostContent
                            ssr=leptos_router::SsrMode::Async
                        />
                        <Route path=path!("/about") view=about::Page />
                        <Route path=path!("any") view=NotFound />
                    </Routes>
                </main>
            </Router>
        </I18nContextProvider>
    }
}

#[component]
fn NavBar() -> impl IntoView {
    let i18n = use_i18n();
    let backdrop = use_context::<BackdropProvider>().expect("BackdropProvider should be provided");

    let backdrop2 = backdrop.clone();
    let anim_toggle_icon = move || {
        if backdrop2.animation_enabled() {
            view! { <Icon icon=icondata::MdiFilmstrip /> }
        } else {
            view! { <Icon icon=icondata::MdiFilmstripOff /> }
        }
    };

    let on_anim_toggle = move |_| {
        backdrop.toggle_animation();
    };

    view! {
        <nav class="fixed left-0 top-0 right-0 container mx-auto flex px-4 py-4 flex-row gap-6">
            <Link class="font-bold" href="/">
                "akosnad.dev"
            </Link>
            <Link href="/posts">{t!(i18n, posts)}</Link>
            <Link href="/about">{t!(i18n, about)}</Link>

            <button
                class="ml-auto motion-reduce:hidden"
                on:click=on_anim_toggle
                title="Toggle visual animations"
            >
                {anim_toggle_icon}
            </button>
        </nav>
    }
}

#[component]
fn Link(
    #[prop(into)] href: String,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let location = use_location();

    let pointed_path = href.clone();
    let class = move || {
        if location.pathname.get() == pointed_path {
            format!(
                "{} hover:-translate-y-0.5 underline hover:decoration-8 decoration-4 transition-all",
                class
            )
        } else {
            format!(
                "{} hover:-translate-y-0.5 underline hover:decoration-4 transition-all",
                class
            )
        }
    };
    view! {
        <a class=class href=href>
            {children()}
        </a>
    }
}

#[component]
fn Backdrop() -> impl IntoView {
    let ctx: BackdropProvider = use_context().expect("BackdropProvider should be provided");

    view! {
        <div class="backdrop overflow-hidden">
            <Show when=move || { ctx.animation_enabled() }>
                <div class="halftone">
                    <div id="background">
                        <div id="bg-breathe" />
                        <img src="/assets/bg.jpg" class="easeload" />
                    </div>
                </div>
            </Show>
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
