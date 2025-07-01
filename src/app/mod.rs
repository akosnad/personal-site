use crate::i18n::*;
use leptos::*;
use leptos_icons::Icon;
use leptos_meta::*;
use leptos_router::*;
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
    fn animation_enabled_fallback_value(&self) -> bool {
        self.is_screen_lg_or_larger.get() && !self.is_reduced_motion_preferred.get()
    }

    fn animation_enabled(&self) -> bool {
        if let Some(forced_value) = self.animation_user_override.get() {
            forced_value
        } else {
            self.animation_enabled_fallback_value()
        }
    }

    fn toggle_animation(&self) {
        self.animation_user_override_set.update(|w| {
            *w = match *w {
                Some(val) => Some(!val),
                None => Some(!self.animation_enabled_fallback_value()),
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
    let backdrop = use_context::<BackdropProvider>().expect("BackdropProvider should be provided");

    let backdrop2 = backdrop.clone();
    let anim_toggle_icon = move || {
        if backdrop2.animation_enabled() {
            view! { <Icon icon=icondata::MdiAnimationPlayOutline /> }
        } else {
            view! { <Icon icon=icondata::MdiAnimationOutline /> }
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
                class="ml-auto"
                on:click=on_anim_toggle
                title="Toggle visual animations"
                aria-hidden="true"
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
    let ctx: BackdropProvider = use_context().expect("BackdropProvider should be provided");

    let animation = move || {
        if ctx.animation_enabled() {
            view! {
                <div class="halftone">
                    <div id="background">
                        <div id="bg-breathe" />
                        <img src="/assets/bg.jpg" onload="this.style.opacity=1" class="easeload" />
                    </div>
                </div>
            }
        } else {
            view! { <div /> }
        }
    };

    view! { <div class="backdrop overflow-hidden">{animation} <div id="background-color" /></div> }
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
