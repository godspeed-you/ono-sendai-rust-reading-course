//! Markdown rendering for course prose.
//!
//! Prose is CommonMark with tables and strikethrough. The renderer is deliberately strict:
//! raw HTML, images, headings (they would break the page's heading hierarchy) and any link that
//! is not a course-internal reference are errors, so the generated course cannot acquire an
//! external dependency or a broken link through prose.

use crate::highlight;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, LinkType, Options, Parser, Tag, TagEnd};

/// Resolves course-internal link targets such as `lesson:ownership-02` to relative hrefs.
pub trait LinkResolver {
    /// Returns the href for a `scheme:id` target relative to a page whose path to the site root
    /// is `root` (`""` or `"../"`), or `None` if the target does not exist.
    fn resolve(&self, scheme: &str, id: &str, root: &str) -> Option<String>;
}

pub const LINK_SCHEMES: &[&str] = &["lesson", "chapter", "concept", "topic", "glossary"];

/// Render Markdown to HTML. `root` is the relative path from the current page to the site root.
pub fn render(md: &str, resolver: &dyn LinkResolver, root: &str) -> Result<String, Vec<String>> {
    let mut errors = Vec::new();
    let opts = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH;
    let mut events: Vec<Event> = Vec::new();
    let mut code_block: Option<(String, String)> = None; // (lang, text)

    for ev in Parser::new_ext(md, opts) {
        if let Some((lang, text)) = code_block.as_mut() {
            match ev {
                Event::Text(t) => {
                    text.push_str(&t);
                    continue;
                }
                Event::End(TagEnd::CodeBlock) => {
                    events.push(Event::Html(CowStr::from(illustration(lang, text))));
                    code_block = None;
                    continue;
                }
                _ => continue,
            }
        }
        match ev {
            Event::Html(h) | Event::InlineHtml(h) => {
                errors.push(format!(
                    "raw HTML is not allowed in course prose: `{}`",
                    h.trim()
                ));
            }
            Event::Start(Tag::Image { dest_url, .. }) => {
                errors.push(format!("images are not supported in prose: `{dest_url}`"));
            }
            Event::Start(Tag::Heading { .. }) => {
                errors.push(
                    "headings are not allowed in prose; use a section `title` instead".to_string(),
                );
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                let lang = match kind {
                    CodeBlockKind::Fenced(l) => l.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                code_block = Some((lang, String::new()));
            }
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            }) => {
                let href = match link_type {
                    LinkType::Autolink | LinkType::Email => {
                        errors.push(format!("external links are not allowed: `{dest_url}`"));
                        String::from("#")
                    }
                    _ => match resolve_target(&dest_url, resolver, root) {
                        Ok(h) => h,
                        Err(e) => {
                            errors.push(e);
                            String::from("#")
                        }
                    },
                };
                events.push(Event::Start(Tag::Link {
                    link_type,
                    dest_url: CowStr::from(href),
                    title,
                    id,
                }));
            }
            other => events.push(other),
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, events.into_iter());
    Ok(html)
}

/// Render Markdown that must fit on one line (titles, choices): the outer `<p>` is removed.
pub fn render_inline(
    md: &str,
    resolver: &dyn LinkResolver,
    root: &str,
) -> Result<String, Vec<String>> {
    let html = render(md, resolver, root)?;
    let trimmed = html.trim();
    let inner = trimmed
        .strip_prefix("<p>")
        .and_then(|s| s.strip_suffix("</p>"))
        .filter(|s| !s.contains("<p>"));
    Ok(inner
        .map(str::to_string)
        .unwrap_or_else(|| trimmed.to_string()))
}

fn resolve_target(dest: &str, resolver: &dyn LinkResolver, root: &str) -> Result<String, String> {
    let Some((scheme, id)) = dest.split_once(':') else {
        return Err(format!(
            "link target `{dest}` is not a course reference (use lesson:, chapter:, concept:, topic: or glossary:)"
        ));
    };
    if !LINK_SCHEMES.contains(&scheme) {
        return Err(format!(
            "link target `{dest}` is not allowed; only course-internal references are (external URLs would break offline use)"
        ));
    }
    resolver
        .resolve(scheme, id, root)
        .ok_or_else(|| format!("broken internal link `{dest}`: no {scheme} with id `{id}`"))
}

/// An artificial example inside prose. It is visibly labelled so it is never mistaken for
/// Ono-Sendai source (spec §2).
fn illustration(lang: &str, text: &str) -> String {
    let body: String = if lang == "rust" {
        highlight::highlight_rust(text).join("\n")
    } else {
        crate::html::escape(text.trim_end_matches('\n'))
    };
    let label = if lang == "rust" {
        "Illustration — not Ono-Sendai source"
    } else {
        "Illustration"
    };
    format!(
        "<figure class=\"illustration\"><figcaption>{label}</figcaption><pre tabindex=\"0\"><code>{body}</code></pre></figure>\n"
    )
}

/// Collect every `scheme:id` link target in a Markdown string, for reference checks.
pub fn link_targets(md: &str) -> Vec<String> {
    Parser::new_ext(md, Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH)
        .filter_map(|ev| match ev {
            Event::Start(Tag::Link { dest_url, .. }) => Some(dest_url.to_string()),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct R;
    impl LinkResolver for R {
        fn resolve(&self, scheme: &str, id: &str, root: &str) -> Option<String> {
            (id == "ok").then(|| format!("{root}{scheme}/{id}.html"))
        }
    }

    #[test]
    fn renders_basic_markdown() {
        let html = render("Hello *world* and `code`.", &R, "").unwrap();
        assert_eq!(html, "<p>Hello <em>world</em> and <code>code</code>.</p>\n");
    }

    #[test]
    fn escapes_text() {
        let html = render("a < b & `Vec<T>`", &R, "").unwrap();
        assert!(
            html.contains("a &lt; b &amp; <code>Vec&lt;T&gt;</code>"),
            "{html}"
        );
    }

    #[test]
    fn resolves_internal_links_relative_to_root() {
        let html = render("[x](lesson:ok)", &R, "../").unwrap();
        assert!(html.contains("href=\"../lesson/ok.html\""), "{html}");
    }

    #[test]
    fn rejects_broken_and_external_links() {
        assert!(
            render("[x](lesson:missing)", &R, "").unwrap_err()[0].contains("broken internal link")
        );
        assert!(render("[x](https://example.com)", &R, "").unwrap_err()[0].contains("not allowed"));
        assert!(render("<https://example.com>", &R, "").is_err());
        assert!(render("[x](other.html)", &R, "").is_err());
    }

    #[test]
    fn rejects_raw_html_images_and_headings() {
        assert!(render("<script>x</script>", &R, "").is_err());
        assert!(render("a <b>bold</b>", &R, "").is_err());
        assert!(render("![alt](img.png)", &R, "").is_err());
        assert!(render("# Heading", &R, "").is_err());
    }

    #[test]
    fn labels_illustrative_code() {
        let html = render("```rust\nlet x = 1;\n```", &R, "").unwrap();
        assert!(html.contains("not Ono-Sendai source"), "{html}");
        assert!(html.contains("<span class=\"tok-kw\">let</span>"), "{html}");
    }

    #[test]
    fn inline_strips_paragraph() {
        assert_eq!(render_inline("*a*", &R, "").unwrap(), "<em>a</em>");
    }
}
