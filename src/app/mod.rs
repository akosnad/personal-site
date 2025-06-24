use std::f64::EPSILON;

use crate::i18n::*;
use leptos::*;
use leptos_animation::*;
use leptos_meta::*;
use leptos_router::*;
use leptos_use::{UseWindowSizeReturn, use_window_size};

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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd)]
enum Direction {
    #[default]
    Positive,
    Negative,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
struct Position {
    x: f64,
    y: f64,
    dir_x: Direction,
    dir_y: Direction,
}
impl std::ops::Sub for Position {
    type Output = Position;

    fn sub(self, rhs: Self) -> Self::Output {
        Position {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            ..self
        }
    }
}
impl std::ops::Add for Position {
    type Output = Position;

    fn add(self, rhs: Self) -> Self::Output {
        Position {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            ..self
        }
    }
}
impl std::ops::Mul<f64> for Position {
    type Output = Position;

    fn mul(self, rhs: f64) -> Self::Output {
        Position {
            x: self.x * rhs,
            y: self.y * rhs,
            ..self
        }
    }
}

#[component]
fn Backdrop() -> impl IntoView {
    let UseWindowSizeReturn { width, height } = use_window_size();
    let (pos, set_pos) = create_signal(Position {
        x: 120.0,
        y: 220.0,
        ..Default::default()
    });
    let pos_anim = create_animated_signal(move || pos.get().into(), tween_default);

    let (breathe_cycle, set_breathe_cycle) = create_signal(0.0);
    let breathe_cycle_anim =
        create_animated_signal(move || breathe_cycle.get().into(), tween_default);

    const R: f64 = 100.0;
    let bg_img_style = move || {
        let pos = pos_anim.get();
        let cx = pos.x as f64 - R;
        let cy = pos.y as f64 - R;

        format!("left: {}px; top: {}px;", cx, cy)
    };

    let center_color = move || {
        set_pos.update(|pos| {
            // collide
            if pos.x + R >= width.get() {
                pos.dir_x = Direction::Negative;
            }
            if pos.x - R <= EPSILON {
                pos.dir_x = Direction::Positive;
            }

            if pos.y + R >= height.get() {
                pos.dir_y = Direction::Negative;
            }
            if pos.y - R <= EPSILON {
                pos.dir_y = Direction::Positive;
            }

            // apply movement
            const SPEED: f64 = 3.0; // pixels/frame
            pos.x += match pos.dir_x {
                Direction::Positive => SPEED,
                Direction::Negative => -SPEED,
            };
            pos.y += match pos.dir_y {
                Direction::Positive => SPEED,
                Direction::Negative => -SPEED,
            };
        });

        const BREATHE_STEP: f64 = 0.02;
        const BREATHE_INTENSITY: f64 = 0.04;
        const BREATHE_OFFSET: f64 = 0.3;
        let breathe_cycle = breathe_cycle_anim.get();
        set_breathe_cycle.set(match breathe_cycle {
            0.0..0.2 => 0.0,
            b => b - BREATHE_STEP,
        });
        let alpha = BREATHE_INTENSITY * breathe_cycle + BREATHE_OFFSET;

        format!("rgba(0.0,0.0,0.0,{alpha})")
    };

    view! {
        <div class="backdrop overflow-hidden">
            <div class="halftone">
                <div id="background">
                    <svg class="absolute" witdth="200" height="200" style=bg_img_style>
                        <defs>
                            <radialGradient id="bw">
                                <stop offset="0%" stop-color=center_color />
                                <stop offset="100%" stop-color="rgba(0.0,0.0,0.0,0.0)" />
                            </radialGradient>
                        </defs>
                        <circle cx="100" cy="100" r="100" fill="url(#bw)" />
                    </svg>
                    <img src="/assets/bg.jpg" />
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

    view! { <h1>"Not Found"</h1> }
}
