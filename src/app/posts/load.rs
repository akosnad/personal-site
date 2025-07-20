use super::id::{PostId, PostIdError};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PostLoadError {
    InvalidId(PostIdError),
    MarkdownParseFailed(String),
    SyntaxHighlightFailed(String),
    RenderMathFailed(String),
    NotFound,
    NoMetadata,
    MetadataParseFailed(String),
    FootnoteDefNotReferenced(String),
    MultipleFootnoteDefinitions(String),
    Unknown,
}
impl std::fmt::Display for PostLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidId(e) => write!(f, "Invalid post ID: {e}"),
            Self::NotFound => write!(f, "Post doesn't exist (yet!)"),
            Self::MarkdownParseFailed(s) => write!(f, "Parsing markdown failed: {s}"),
            Self::SyntaxHighlightFailed(s) => write!(f, "Code syntax highlighting failed: {s}"),
            Self::RenderMathFailed(s) => write!(f, "Rendering math failed: {s}"),
            Self::NoMetadata => write!(f, "Post file has no metadata"),
            Self::MetadataParseFailed(s) => write!(f, "Post metadata parsing failed: {s}"),
            Self::FootnoteDefNotReferenced(s) => {
                write!(f, "No references to footnote definition: {s}")
            }
            Self::MultipleFootnoteDefinitions(s) => {
                write!(f, "Multiple footnote definitions present for {s}")
            }
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
        if let Some(s) = s.strip_prefix("Rendering math failed: ") {
            return Ok(Self::RenderMathFailed(String::from(s)));
        }
        if let Some(s) = s.strip_prefix("Post metadata parsing failed: ") {
            return Ok(Self::MetadataParseFailed(String::from(s)));
        }
        if let Some(s) = s.strip_prefix("No references to footnote definition: ") {
            return Ok(Self::FootnoteDefNotReferenced(String::from(s)));
        }
        if let Some(s) = s.strip_prefix("Multiple footnote definitions present for ") {
            return Ok(Self::MultipleFootnoteDefinitions(String::from(s)));
        }
        match s {
            "Post doesn't exist (yet!)" => Ok(Self::NotFound),
            "Post file has no metadata" => Ok(Self::NoMetadata),
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
pub async fn load_post_content(post_id: PostId) -> Result<Post, ServerFnError<PostLoadError>> {
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
                ..markdown::Constructs::gfm()
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
            let mut metadata = None;
            let mut footnotes = std::collections::HashMap::new();
            let Some(preprocessed_mdast) =
                preprocess(mdast, &mut metadata, &mut footnotes, &md_options)
                    .map_err(|e| ServerFnError::from(e))?
            else {
                return Err(PostLoadError::MarkdownParseFailed(
                    "preprocess returned no root element".to_string(),
                )
                .into());
            };
            let Some(metadata) = metadata else {
                return Err(PostLoadError::NoMetadata.into());
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

            post_cache.ids.insert(post_id.number, post_id.clone());
            post_cache.entries.insert(
                post_id.number,
                CachedPost {
                    content: html.clone(),
                    last_update: Instant::now(),
                },
            );
            Ok(Post { html, metadata })
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
#[derive(Debug, Clone)]
struct PreprocessedPost {
    root: markdown::mdast::Node,
    metadata: Option<PostMetadata>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PostMetadata {
    pub title: String,
    pub date: chrono::NaiveDate,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Post {
    pub html: String,
    pub metadata: PostMetadata,
}

#[cfg(feature = "ssr")]
fn preprocess(
    mut content: markdown::mdast::Node,
    metadata: &mut Option<PostMetadata>,
    footnotes: &mut std::collections::HashMap<String, (usize, bool)>,
    md_options: &markdown::Options,
) -> Result<Option<markdown::mdast::Node>, PostLoadError> {
    use markdown::mdast::{
        Code, Delete, FootnoteDefinition, FootnoteReference, Html, InlineMath, Math, Node, Text,
        Toml, Yaml,
    };

    if let Some(children) = content.children_mut() {
        // preprocess children
        let new_children: Result<Vec<Option<Node>>, _> = children
            .iter()
            .map(|c| preprocess(c.clone(), metadata, footnotes, md_options))
            .collect();
        // remove `None` children
        let new_children: Vec<Node> = new_children?.iter().cloned().flatten().collect();
        let _ = std::mem::replace(children, new_children);
    }

    Ok(match content {
        Node::Code(Code {
            value,
            position,
            lang,
            ..
        }) if lang.is_some() => {
            let lang = lang.unwrap();
            Some(Node::Html(Html {
                value: syntax_highlight(&value, &lang)?,
                position,
            }))
        }
        Node::Math(Math {
            value, position, ..
        }) => Some(Node::Html(Html {
            value: render_math(&value, false)?,
            position,
        })),
        Node::InlineMath(InlineMath { value, position }) => Some(Node::Html(Html {
            value: render_math(&value, true)?,
            position,
        })),
        Node::Yaml(Yaml { value, .. }) => {
            let new_metadata: PostMetadata = serde_yaml::from_str(&value)
                .map_err(|e| PostLoadError::MetadataParseFailed(format!("{e:?}")))?;
            let _ = metadata.insert(new_metadata);
            None
        }
        Node::Toml(Toml { value, .. }) => {
            let new_metadata: PostMetadata = toml::from_str(&value)
                .map_err(|e| PostLoadError::MetadataParseFailed(format!("{e:?}")))?;
            let _ = metadata.insert(new_metadata);
            None
        }
        Node::FootnoteReference(FootnoteReference {
            position,
            identifier,
            label,
        }) => {
            let num = if let Some((prev_num, _)) = footnotes.get(&identifier) {
                *prev_num
            } else {
                let new_num = footnotes.len() + 1;
                footnotes.insert(identifier, (new_num, false));
                new_num
            };
            Some(Node::Html(Html {
                value: format!("<a class=\"footnote-ref\" href=\"#footnote-{num}\">{num}</a>"),
                position,
            }))
        }
        Node::FootnoteDefinition(def) => match footnotes.get(&def.identifier) {
            Some((num, false)) => {
                let num = num.clone();
                footnotes.insert(def.identifier.clone(), (num, true));
                let html = render_footnote_definition(&def, md_options)?;
                Some(Node::Html(Html {
                    value: format!(
                        "<div class=\"footnote-def\" id=\"footnote-{num}\"><p>[{num}]: </p>{html}</div>"
                    ),
                    position: def.position,
                }))
            }
            Some((_, true)) => {
                return Err(PostLoadError::MultipleFootnoteDefinitions(format!(
                    "{}",
                    def.identifier
                )));
            }
            None => {
                return Err(PostLoadError::FootnoteDefNotReferenced(def.identifier));
            }
        },
        c => Some(c),
    })
}

#[cfg(feature = "ssr")]
fn render_footnote_definition(
    node: &markdown::mdast::FootnoteDefinition,
    md_options: &markdown::Options,
) -> Result<String, PostLoadError> {
    let md =
        mdast_util_to_markdown::to_markdown(&markdown::mdast::Node::Root(markdown::mdast::Root {
            children: node.children.clone(),
            position: None,
        }))
        .map_err(|e| {
            eprintln!("render_footnote_definition to_markdown() failed: {e:?}");
            PostLoadError::MarkdownParseFailed(e.reason)
        })?;
    let html = markdown::to_html_with_options(&md, md_options).map_err(|e| {
        eprintln!("render_footnote_definition to_html_with_options() failed: {e:?}");
        PostLoadError::MarkdownParseFailed(e.reason)
    })?;
    Ok(html)
}

#[cfg(feature = "ssr")]
fn render_math(src: &str, inline: bool) -> Result<String, PostLoadError> {
    let opts = katex::Opts::builder()
        .display_mode(!inline)
        .build()
        .map_err(|e| PostLoadError::RenderMathFailed(format!("{e:?}")))?;
    let html = katex::render_with_opts(src, &opts)
        .map_err(|e| PostLoadError::RenderMathFailed(format!("{e:?}")))?;
    if inline {
        Ok(html)
    } else {
        Ok(format!("<pre>{html}</pre>"))
    }
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
            return Err(PostLoadError::SyntaxHighlightFailed(format!(
                "{lang} not implemented :("
            )));
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
                    "<span class=\"highlight {}\">",
                    highlight_names[s.0]
                ));
            }
            HighlightEvent::HighlightEnd => {
                highlighted_src.push_str("</span>");
            }
        }
    }
    Ok(format!(
        "<pre><code class=\"language-{lang}\">{highlighted_src}</code></pre>"
    ))
}
