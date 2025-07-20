use crate::app::Link;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::hooks::{use_location, use_params_map};
use leptos_use::use_window;
use std::str::FromStr as _;

mod id;
use id::*;

mod load;
use load::*;

#[component]
pub fn PostsList() -> impl IntoView {
    view! {
        <Link href="/posts/1-hello">"post 1"</Link>
        <Link href="/posts/4">"post 4"</Link>
        <Link href="/posts/6">"post 6"</Link>
        <Link href="/posts/7">"post 7"</Link>
        <Link href="/posts/42-haha">"post 42"</Link>
    }
}

#[component]
pub fn PostContent() -> impl IntoView {
    let params = use_params_map();

    let post_id = move || {
        let result = params
            .read()
            .get("id")
            .ok_or(PostIdError::Missing)
            .and_then(|s| PostId::from_str(&s));
        #[cfg(feature = "ssr")]
        {
            if let Err(_) = result {
                let resp = expect_context::<leptos_actix::ResponseOptions>();
                resp.set_status(actix_web::http::StatusCode::BAD_REQUEST);
            }
        }
        result
    };

    let post = Resource::new(post_id, move |post_id| async move {
        match post_id {
            Ok(id) => load_post_content(id).await,
            Err(e) => Err(PostLoadError::InvalidId(e).into()),
        }
    });

    view! {
        <ErrorBoundary fallback=|errors| {
            view! {
                <div class="error">
                    <ul>
                        {move || {
                            errors
                                .get()
                                .into_iter()
                                .map(|(_, e)| view! { <li>{e.to_string()}</li> })
                                .collect::<Vec<_>>()
                        }}
                    </ul>
                </div>
            }
        }>
            <Suspense fallback=move || {
                view! { <p>"Loading..."</p> }
            }>
                {move || {
                    post.read()
                        .as_ref()
                        .cloned()
                        .map(|res| { res.map(|p| view! { <PostBody post=p /> }) })
                }}
            </Suspense>
        </ErrorBoundary>
    }
}

#[component]
fn PostBody(post: Post) -> impl IntoView {
    view! {
        <div id="post-metadata">
            <h1 id="post-title" class="text-4xl font-extrabold">
                {post.metadata.title}
            </h1>
            <p id="post-date">{post.metadata.date.to_string()}</p>
            <hr />
        </div>
        <div id="post-body" inner_html=post.html />
    }
}
