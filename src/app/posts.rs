use crate::{app::Link, i18n::*};
use leptos::prelude::*;
use leptos_router::{hooks::use_params, params::Params};

#[component]
pub fn PostsList() -> impl IntoView {
    view! {
        <Link href="/posts/1">"post 1"</Link>
        <Link href="/posts/4">"post 4"</Link>
        <Link href="/posts/6">"post 6"</Link>
        <Link href="/posts/7">"post 7"</Link>
    }
}

type PostId = String;

#[derive(Params, Clone, PartialEq)]
struct PostContentParams {
    id: Option<PostId>,
}

#[server]
async fn load_post_content(post_id: PostId) -> Result<String, ServerFnError> {
    Ok(format!("hello post {post_id}!"))
}

#[component]
pub fn PostContent() -> impl IntoView {
    let i18n = use_i18n();
    let params = use_params::<PostContentParams>();

    let id = move || {
        params
            .read()
            .as_ref()
            .ok()
            .and_then(|params| params.id.clone())
            .unwrap_or_default()
    };

    let post = Resource::new(id, |id| async move { load_post_content(id).await });

    view! {
        <Suspense fallback=move || {
            view! { <p>"Loading..."</p> }
        }>
            {move || match post.get() {
                None => view! { <div>"None"</div> },
                Some(Err(e)) => view! { <div>{format!("{e:?}")}</div> },
                Some(Ok(post)) => view! { <div>{post}</div> },
            }}
        </Suspense>
    }
}
