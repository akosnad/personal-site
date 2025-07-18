use super::id::{PostId, PostIdError};
use leptos::prelude::*;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum PostLoadError {
    InvalidId(PostIdError),
    MarkdownParseFailed(String),
    SyntaxHighlightFailed(String),
    NotFound,
    Unknown,
}
impl std::fmt::Display for PostLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidId(e) => write!(f, "Invalid post ID: {e}"),
            Self::NotFound => write!(f, "Post doesn't exist (yet!)"),
            Self::MarkdownParseFailed(s) => write!(f, "Parsing markdown failed: {s}"),
            Self::SyntaxHighlightFailed(s) => write!(f, "Code syntax highlighting failed: {s}"),
            Self::Unknown => write!(f, "Unknown error"),
        }
    }
}
impl std::error::Error for PostLoadError {}
impl std::str::FromStr for PostLoadError {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(s) = s.strip_prefix("Invalid post ID: ") {
            return Ok(Self::InvalidId(PostIdError::from_str(s)?));
        }
        if let Some(s) = s.strip_prefix("Parsing markdown failed: ") {
            return Ok(Self::MarkdownParseFailed(String::from(s)));
        }
        if let Some(s) = s.strip_prefix("Code syntax highlighting failed: ") {
            return Ok(Self::SyntaxHighlightFailed(String::from(s)));
        }
        match s {
            "Post doesn't exist (yet!)" => Ok(Self::NotFound),
            _ => Ok(Self::Unknown),
        }
    }
}

#[cfg(feature = "ssr")]
#[derive(Default)]
struct PostCache {
    ids: std::collections::HashMap<usize, PostId>,
    entries: std::collections::HashMap<usize, CachedPost>,
}

#[cfg(feature = "ssr")]
struct CachedPost {
    content: String,
    last_update: std::time::Instant,
}

#[server]
pub async fn load_post_content(post_id: PostId) -> Result<String, ServerFnError<PostLoadError>> {
    use std::io::ErrorKind;
    use std::sync::{Arc, Mutex, OnceLock};
    use std::time::{Duration, Instant};

    static POST_CACHE: OnceLock<Arc<Mutex<PostCache>>> = OnceLock::new();
    let md_options = markdown::Options {
        parse: markdown::ParseOptions {
            constructs: markdown::Constructs {
                math_flow: true,
                math_text: true,
                frontmatter: true,
                ..Default::default()
            },
            ..Default::default()
        },
        compile: markdown::CompileOptions {
            allow_any_img_src: true,
            allow_dangerous_html: true,
            allow_dangerous_protocol: true,
            ..Default::default()
        },
    };

    let mut post_cache = POST_CACHE
        .get_or_init(|| Default::default())
        .lock()
        .unwrap();
    eprintln!(
        "cached: {},{}",
        post_cache.ids.len(),
        post_cache.entries.len()
    );

    #[cfg(not(debug_assertions))]
    if let Some(post) = post_cache.entries.get(&post_id.number) {
        if post.last_update.elapsed() < Duration::from_secs(12 * 60 * 60) {
            return Ok(post.content.clone());
        }
    }

    if let Some(cached_id) = post_cache.ids.get(&post_id.number) {}

    let resp = expect_context::<leptos_actix::ResponseOptions>();
    let config = leptos::config::get_configuration(None).unwrap();
    let site_root = &config.leptos_options.site_root;

    let post_path = format!("{}/posts/{}-{}.md", site_root, post_id.number, post_id.slug);
    eprintln!("{post_path}");
    match std::fs::read_to_string(post_path) {
        Ok(post_raw) => {
            // safe to unwrap because markdown doesn't have syntax errors
            let mdast = markdown::to_mdast(&post_raw, &md_options.parse).unwrap();
            let Some(preprocessed_mdast) = preprocess(mdast)? else {
                return Err(PostLoadError::MarkdownParseFailed(
                    "preprocess returned no root element".to_string(),
                )
                .into());
            };
            let preprocessed_md = mdast_util_to_markdown::to_markdown(&preprocessed_mdast)
                .map_err(|e| {
                    eprintln!("to_markdown() failed: {e:?}");
                    PostLoadError::MarkdownParseFailed(e.reason)
                })?;
            let html =
                markdown::to_html_with_options(&preprocessed_md, &md_options).map_err(|e| {
                    eprintln!("to_html_with_options() failed: {e:?}");
                    PostLoadError::MarkdownParseFailed(e.reason)
                })?;
            let postprocessed_html = postprocess(&html)?;

            post_cache.ids.insert(post_id.number, post_id.clone());
            post_cache.entries.insert(
                post_id.number,
                CachedPost {
                    content: postprocessed_html.clone(),
                    last_update: Instant::now(),
                },
            );
            Ok(postprocessed_html)
        }
        Err(e) => match e.kind() {
            ErrorKind::NotFound => Err(PostLoadError::NotFound.into()),
            _ => Err(ServerFnError::ServerError(
                "failed to read post file".to_string(),
            )),
        },
    }
}

#[cfg(feature = "ssr")]
fn preprocess(
    mut content: markdown::mdast::Node,
) -> Result<Option<markdown::mdast::Node>, PostLoadError> {
    use markdown::mdast::{Code, Node};

    if let Some(children) = content.children_mut() {
        // preprocess children
        let new_children: Result<Vec<Option<Node>>, _> =
            children.iter().map(|c| preprocess(c.clone())).collect();
        // remove `None` children
        let new_children: Vec<Node> = new_children?.iter().cloned().flatten().collect();
        let _ = std::mem::replace(children, new_children);
    }

    Ok(match content {
        Node::Code(Code {
            value,
            position,
            lang,
            meta,
        }) if lang.is_some() => {
            // syntax highlight code block with language specified

            let lang = lang.unwrap();
            Some(Node::Code(Code {
                value: syntax_highlight(&value, &lang)?,
                position,
                lang: Some(lang),
                meta,
            }))
        }
        // remove frontmatter
        // TODO: parse and return values as post metadata
        Node::Yaml(_) | Node::Toml(_) => None,

        // TODO: also render math blocks
        c => Some(c),
    })
}

#[cfg(feature = "ssr")]
fn syntax_highlight(src: &str, lang: &str) -> Result<String, PostLoadError> {
    // TODO: support all highlight types
    let highlight_names = [
        // "attribute",
        "comment", "constant",
        // "constant.builtin",
        // "constructor",
        // "embedded",
        "function", // "function.builtin",
        "keyword",  // "module",
        "number",
        // "operator",
        // "property",
        // "property.builtin",
        // "punctuation",
        // "punctuation.bracket",
        // "punctuation.delimiter",
        // "punctuation.special",
        "string", // "string.special",
        "tag", "type",
        // "type.builtin",
        "variable",
        // "variable.builtin",
        // "variable.parameter",
    ];
    use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};
    let mut highlighter = Highlighter::new();

    let mut config = match lang {
        "rust" => {
            let rust_lang = tree_sitter_rust::LANGUAGE;
            HighlightConfiguration::new(
                rust_lang.into(),
                "rust",
                tree_sitter_rust::HIGHLIGHTS_QUERY,
                tree_sitter_rust::INJECTIONS_QUERY,
                tree_sitter_rust::TAGS_QUERY,
            )
            .map_err(|e| {
                PostLoadError::SyntaxHighlightFailed(format!(
                    "failed to initialize tree-sitter-rust: {e:?}"
                ))
            })?
        }
        "html" => {
            let html_lang = tree_sitter_html::LANGUAGE;
            HighlightConfiguration::new(
                html_lang.into(),
                "html",
                tree_sitter_html::HIGHLIGHTS_QUERY,
                tree_sitter_html::INJECTIONS_QUERY,
                "",
            )
            .map_err(|e| {
                PostLoadError::SyntaxHighlightFailed(format!(
                    "failed to initialize tree-sitter-html: {e:?}"
                ))
            })?
        }
        _ => {
            return Err(PostLoadError::SyntaxHighlightFailed(
                "language highlight not implemented".to_string(),
            ));
        }
    };

    config.configure(&highlight_names);

    let highlights = highlighter
        .highlight(&config, src.as_bytes(), None, |_| None)
        .map_err(|e| PostLoadError::SyntaxHighlightFailed(format!("{e:?}")))?;

    let mut highlighted_src = String::new();
    for event in highlights {
        match event.map_err(|e| PostLoadError::SyntaxHighlightFailed(format!("{e:?}")))? {
            HighlightEvent::Source { start, end } => {
                highlighted_src.push_str(&src[start..end]);
            }
            HighlightEvent::HighlightStart(s) => {
                highlighted_src.push_str(&format!(
                    "!~#<span class=\"highlight {}\">#~!",
                    highlight_names[s.0]
                ));
            }
            HighlightEvent::HighlightEnd => {
                highlighted_src.push_str("!~#</span>#~!");
            }
        }
    }
    Ok(highlighted_src)
}

#[cfg(feature = "ssr")]
fn postprocess(html: &str) -> Result<String, PostLoadError> {
    use std::collections::VecDeque;

    let re = regex::Regex::new(r"\!\~\#.*?\#\~\!").map_err(|e| {
        PostLoadError::SyntaxHighlightFailed(format!("failed to parse postprocessing regex: {e:?}"))
    })?;
    let mut replacement_strings: VecDeque<((usize, usize), String)> = re
        .find_iter(html)
        .into_iter()
        .map(|m| {
            let inner = &html[m.start()..m.end()];
            let decoded = html_escape::decode_html_entities(inner);
            ((m.start(), m.end()), String::from(decoded))
        })
        .collect();

    if replacement_strings.is_empty() {
        return Ok(String::from(html));
    }

    let mut postprocessed = String::new();
    let mut last_end = 0;
    while let Some(((start, end), replacement)) = replacement_strings.pop_front() {
        postprocessed.push_str(&html[last_end..start]);
        postprocessed.push_str(&replacement[3..replacement.len() - 3]);
        last_end = end;
    }
    if last_end < html.len() {
        postprocessed.push_str(&html[last_end..html.len()]);
    }

    Ok(postprocessed)
}
