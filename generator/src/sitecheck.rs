//! Checks on the generated site: offline compliance (spec §33, §34), internal links and
//! anchors, required assets and basic HTML sanity (spec §64). Runs after every build and as
//! `scripts/course check-offline`.

use crate::diag::Diagnostics;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

pub const REQUIRED_FILES: &[&str] = &[
    "index.html",
    "about.html",
    "glossary.html",
    "learn-rust.html",
    "ono-sendai.html",
    "course-metadata.json",
    "assets/course.css",
    "assets/course.js",
    "assets/favicon.svg",
    "assets/ONO-SENDAI-LICENSE.txt",
];

/// Runtime APIs that would make a network request (or load code) from the page.
const FORBIDDEN_JS: &[&str] = &[
    "fetch(",
    "XMLHttpRequest",
    "WebSocket",
    "EventSource",
    "sendBeacon",
    "importScripts",
    "serviceWorker",
    "import(",
    "new Worker",
    "RTCPeerConnection",
];

const FORBIDDEN_TAGS: &[&str] = &["<iframe", "<object", "<embed", "<base", "<form action"];

const URL_ATTRS: &[&str] = &[
    "href",
    "src",
    "action",
    "poster",
    "srcset",
    "data",
    "formaction",
    "xlink:href",
];

pub struct SiteCheckOptions<'a> {
    /// The one external URL that may appear as displayed text (the Ono-Sendai repository).
    pub allowed_text_url: &'a str,
}

fn walk(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

fn rel(root: &Path, p: &Path) -> String {
    p.strip_prefix(root)
        .unwrap_or(p)
        .to_string_lossy()
        .replace('\\', "/")
}

/// All `attr="value"` pairs for URL-bearing attributes in an HTML document.
pub fn url_attributes(html: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let bytes = html.as_bytes();
    for attr in URL_ATTRS {
        let pat = format!("{attr}=");
        let mut from = 0;
        while let Some(pos) = html[from..].find(&pat) {
            let start = from + pos;
            from = start + pat.len();
            // Must be a whole attribute name: preceded by whitespace.
            if start == 0 || !bytes[start - 1].is_ascii_whitespace() {
                continue;
            }
            let q = bytes.get(from).copied();
            let value = match q {
                Some(b'"') | Some(b'\'') => {
                    let q = q.unwrap() as char;
                    let rest = &html[from + 1..];
                    rest.find(q).map(|e| rest[..e].to_string())
                }
                _ => {
                    let rest = &html[from..];
                    let e = rest
                        .find(|c: char| c.is_ascii_whitespace() || c == '>')
                        .unwrap_or(rest.len());
                    Some(rest[..e].to_string())
                }
            };
            if let Some(v) = value {
                out.push((attr.to_string(), v));
            }
        }
    }
    out
}

/// All `id="..."` values in a document.
pub fn ids(html: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(pos) = html[from..].find(" id=\"") {
        let s = from + pos + 5;
        if let Some(e) = html[s..].find('"') {
            out.push(html[s..s + e].to_string());
            from = s + e;
        } else {
            break;
        }
    }
    out
}

/// Byte ranges of displayed Ono-Sendai source (`<pre class="source ...">…</pre>`). A URL inside
/// real source text (a comment, a string literal) is displayed text that no browser fetches.
fn source_ranges(text: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(pos) = text[from..].find("<pre class=\"source") {
        let s = from + pos;
        let e = text[s..].find("</pre>").map_or(text.len(), |e| s + e);
        out.push((s, e));
        from = e;
    }
    out
}

/// XML namespace names that may appear in `xmlns` attributes of local SVG files.
const SVG_NAMESPACES: &[&str] = &["http://www.w3.org/2000/svg", "http://www.w3.org/1999/xlink"];

/// Report every occurrence of an absolute URL. Allowed are only the pinned repository URL as
/// displayed text, and URLs that are part of displayed Ono-Sendai source text.
fn scan_urls(file: &str, text: &str, allowed: &str, diags: &mut Diagnostics) {
    let is_html = file.ends_with(".html");
    let sources = if is_html {
        source_ranges(text)
    } else {
        Vec::new()
    };
    for scheme in ["http://", "https://"] {
        let mut from = 0;
        while let Some(pos) = text[from..].find(scheme) {
            let start = from + pos;
            from = start + scheme.len();
            let url_end = text[start..]
                .find(|c: char| {
                    c.is_whitespace() || matches!(c, '"' | '\'' | '<' | '>' | ')' | '`')
                })
                .map_or(text.len(), |e| start + e);
            let url = &text[start..url_end];
            let mut b = start.saturating_sub(12);
            while !text.is_char_boundary(b) {
                b -= 1;
            }
            let before = &text[b..start];
            // An XML namespace name in a local SVG file identifies a vocabulary; nothing fetches it.
            if file.ends_with(".svg")
                && before.ends_with("xmlns=\"")
                && SVG_NAMESPACES.contains(&url)
            {
                continue;
            }
            let allowed_url =
                !allowed.is_empty() && url.trim_end_matches(['.', ',', ';']) == allowed;
            // The metadata file records the pinned repository as a JSON string value (data, not a link).
            let json_value = file.ends_with(".json") && before.ends_with('"') && allowed_url;
            let in_attr_or_css = !json_value
                && (before.ends_with('"')
                    || before.ends_with('\'')
                    || before.ends_with('=')
                    || before.ends_with("url("));
            let in_source = sources.iter().any(|(s, e)| (*s..*e).contains(&start));
            if in_attr_or_css || !(allowed_url || in_source) {
                diags.error(file, format!("external URL `{url}` found; the course must not reference external resources"));
            }
        }
    }
}

pub fn check_site(dist: &Path, opts: &SiteCheckOptions) -> Diagnostics {
    let mut diags = Diagnostics::default();
    if !dist.is_dir() {
        diags.error(
            dist.display().to_string(),
            "output directory does not exist; run `scripts/course build` first",
        );
        return diags;
    }
    for f in REQUIRED_FILES {
        if !dist.join(f).is_file() {
            diags.error(*f, "required file is missing from the generated site");
        }
    }

    let files = walk(dist);
    let mut page_ids: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut pages: Vec<(String, String)> = Vec::new();
    for p in &files {
        let name = rel(dist, p);
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !matches!(ext, "html" | "css" | "js" | "json" | "svg" | "txt") {
            diags.error(
                &name,
                format!("unexpected file type `.{ext}` in the generated site"),
            );
            continue;
        }
        let Ok(text) = fs::read_to_string(p) else {
            diags.error(&name, "file is not valid UTF-8");
            continue;
        };
        scan_urls(&name, &text, opts.allowed_text_url, &mut diags);
        if text.contains("//cdn")
            || text.contains("fonts.googleapis")
            || text.contains("googletagmanager")
        {
            diags.error(&name, "reference to a CDN, remote font or analytics host");
        }
        if ext == "html" {
            check_inline_code(&name, &text, &mut diags);
        }
        if ext == "js" {
            for api in FORBIDDEN_JS {
                if text.contains(api) {
                    diags.error(
                        &name,
                        format!("network-capable API `{api}` must not be used at runtime"),
                    );
                }
            }
        }
        if ext == "css" {
            if text.contains("@import") {
                diags.error(&name, "CSS @import is not allowed");
            }
            let mut from = 0;
            while let Some(pos) = text[from..].find("url(") {
                let s = from + pos + 4;
                let v = text[s..].trim_start_matches(['"', '\'']);
                if !v.starts_with("data:") {
                    diags.error(&name, "CSS url() must only use inline data: URIs");
                }
                from = s;
            }
        }
        if ext == "html" {
            for t in FORBIDDEN_TAGS {
                if text.contains(t) {
                    diags.error(&name, format!("forbidden element `{t}>`"));
                }
            }
            let lower = text.to_ascii_lowercase();
            if !lower.starts_with("<!doctype html>") {
                diags.error(&name, "page must start with <!doctype html>");
            }
            if !text.contains("<html lang=\"") {
                diags.error(&name, "page must declare its language on <html>");
            }
            if text.matches("<h1").count() != 1 {
                diags.error(&name, "page must have exactly one <h1>");
            }
            if !text.contains("<title>") {
                diags.error(&name, "page must have a <title>");
            }
            if text.contains("user-scalable=no") || text.contains("maximum-scale=1") {
                diags.error(&name, "viewport must not block pinch zoom");
            }
            let all_ids = ids(&text);
            let mut set = BTreeSet::new();
            for id in &all_ids {
                if !set.insert(id.clone()) {
                    diags.error(&name, format!("duplicate id `{id}`"));
                }
            }
            page_ids.insert(name.clone(), set);
            pages.push((name, text));
        }
    }

    // Internal links: every relative href/src must resolve to a file, and fragments to an id.
    for (name, text) in &pages {
        let dir = Path::new(name).parent().unwrap_or(Path::new(""));
        for (attr, value) in url_attributes(text) {
            let v = value.trim();
            if v.is_empty() {
                diags.error(name, format!("empty {attr} attribute"));
                continue;
            }
            if let Some(scheme_end) = v.find(':') {
                let scheme = &v[..scheme_end];
                if scheme.chars().all(|c| c.is_ascii_alphabetic()) {
                    diags.error(name, format!("{attr}=\"{v}\" uses scheme `{scheme}:`; only relative links are allowed"));
                    continue;
                }
            }
            if v.starts_with("//") || v.starts_with('/') {
                diags.error(name, format!("{attr}=\"{v}\" is not a relative link"));
                continue;
            }
            let (path_part, frag) = match v.split_once('#') {
                Some((p, f)) => (p, Some(f)),
                None => (v, None),
            };
            let target = if path_part.is_empty() {
                name.clone()
            } else {
                normalize(&dir.join(path_part))
            };
            if !dist.join(&target).is_file() {
                diags.error(
                    name,
                    format!("broken link {attr}=\"{v}\": `{target}` does not exist"),
                );
                continue;
            }
            if let Some(f) = frag {
                if !f.is_empty() && target.ends_with(".html") {
                    let found = page_ids.get(&target).is_some_and(|s| s.contains(f));
                    if !found {
                        diags.error(
                            name,
                            format!("broken link {attr}=\"{v}\": no id `{f}` in `{target}`"),
                        );
                    }
                }
            }
            if path_part.ends_with('/') {
                diags.error(
                    name,
                    format!("link `{v}` points at a directory; file:// needs explicit .html files"),
                );
            }
        }
    }
    diags
}

/// HTML may only load local script files: inline `<script>` bodies and inline event handler
/// attributes are rejected, so every line of runtime JavaScript lives in a `.js` file where the
/// network-API scan applies. Text inside displayed source (`<pre class="source">`) is data.
fn check_inline_code(file: &str, text: &str, diags: &mut Diagnostics) {
    let sources = source_ranges(text);
    let outside = |pos: usize| !sources.iter().any(|(s, e)| (*s..*e).contains(&pos));
    let mut from = 0;
    while let Some(pos) = text[from..].find("<script") {
        let start = from + pos;
        from = start + 7;
        if !outside(start) {
            continue;
        }
        let tag_end = text[start..].find('>').map_or(text.len(), |e| start + e);
        let tag = &text[start..tag_end];
        let body_end = text[tag_end..]
            .find("</script>")
            .map_or(text.len(), |e| tag_end + e);
        if !tag.contains(" src=") {
            diags.error(
                file,
                "inline <script> is not allowed; runtime JavaScript must live in assets/",
            );
        } else if text[(tag_end + 1).min(body_end)..body_end].trim() != "" {
            diags.error(file, "a <script src> element must not have an inline body");
        }
    }
    let bytes = text.as_bytes();
    let mut i = 0;
    while let Some(pos) = text[i..].find(" on") {
        let at = i + pos;
        i = at + 3;
        let name_end = text[at + 1..]
            .find(|c: char| !c.is_ascii_alphabetic())
            .map_or(text.len(), |e| at + 1 + e);
        let is_handler = name_end > at + 3
            && bytes.get(name_end) == Some(&b'=')
            && text[..at].rfind('<') > text[..at].rfind('>');
        if is_handler && outside(at) {
            diags.error(
                file,
                format!(
                    "inline event handler `{}` is not allowed",
                    &text[at + 1..name_end]
                ),
            );
        }
    }
}

/// Resolve `.` and `..` in a relative path, returning a `/`-separated string.
fn normalize(p: &Path) -> String {
    let mut parts: Vec<String> = Vec::new();
    for c in p.components() {
        match c {
            std::path::Component::ParentDir => {
                parts.pop();
            }
            std::path::Component::CurDir => {}
            other => parts.push(other.as_os_str().to_string_lossy().to_string()),
        }
    }
    parts.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn site(files: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for (name, text) in files {
            let p = dir.path().join(name);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, text).unwrap();
        }
        dir
    }

    fn page(body: &str) -> String {
        format!("<!doctype html><html lang=\"en\"><head><title>t</title></head><body><h1>x</h1>{body}</body></html>")
    }

    fn check(files: &[(&str, &str)]) -> Diagnostics {
        let d = site(files);
        check_site(
            d.path(),
            &SiteCheckOptions {
                allowed_text_url: "https://github.com/godspeed-you/ono-sendai",
            },
        )
    }

    fn minimal() -> Vec<(&'static str, String)> {
        let mut v: Vec<(&str, String)> = REQUIRED_FILES
            .iter()
            .map(|f| {
                (
                    *f,
                    if f.ends_with(".html") {
                        page("")
                    } else {
                        String::new()
                    },
                )
            })
            .collect();
        v.push((
            "lessons/a.html",
            page("<p id=\"x\">a</p><a href=\"../index.html\">home</a>"),
        ));
        v
    }

    fn with(extra: &[(&'static str, String)]) -> Diagnostics {
        let mut v = minimal();
        for (k, t) in extra {
            v.retain(|(n, _)| n != k);
            v.push((k, t.clone()));
        }
        let refs: Vec<(&str, &str)> = v.iter().map(|(a, b)| (*a, b.as_str())).collect();
        check(&refs)
    }

    #[test]
    fn clean_site_passes() {
        let d = with(&[]);
        assert!(!d.has_errors(), "{}", d.report());
    }

    #[test]
    fn detects_missing_required_asset() {
        let v: Vec<(&str, String)> = minimal()
            .into_iter()
            .filter(|(n, _)| *n != "assets/course.js")
            .collect();
        let refs: Vec<(&str, &str)> = v.iter().map(|(a, b)| (*a, b.as_str())).collect();
        assert!(check(&refs).has_error_containing("required file is missing"));
    }

    #[test]
    fn detects_external_script_stylesheet_font_image() {
        for body in [
            "<script src=\"https://cdn.example.com/x.js\"></script>",
            "<link rel=\"stylesheet\" href=\"http://example.com/a.css\">",
            "<img src=\"//example.com/a.png\" alt=\"\">",
            "<link href=\"https://fonts.googleapis.com/css\" rel=\"stylesheet\">",
        ] {
            let d = with(&[("index.html", page(body))]);
            assert!(d.has_errors(), "not detected: {body}");
        }
    }

    #[test]
    fn rejects_inline_scripts_and_handlers_but_not_displayed_code() {
        assert!(with(&[("index.html", page("<script>fetch('x')</script>"))])
            .has_error_containing("inline <script>"));
        assert!(
            with(&[("index.html", page("<button onclick=\"x()\">b</button>"))])
                .has_error_containing("inline event handler")
        );
        let shown = page("<pre class=\"source\"><code>let r = fetch(url); // onload=x</code></pre><p>Read the online docs</p>");
        assert!(!with(&[("lessons/a.html", shown)]).has_errors());
    }

    #[test]
    fn detects_network_apis_in_js() {
        for js in [
            "fetch('x')",
            "new XMLHttpRequest()",
            "navigator.sendBeacon('a')",
            "new WebSocket('ws://x')",
        ] {
            let d = with(&[("assets/course.js", js.to_string())]);
            assert!(
                d.has_error_containing("network-capable API"),
                "not detected: {js}"
            );
        }
    }

    #[test]
    fn detects_css_remote_urls_and_imports() {
        assert!(with(&[("assets/course.css", "@import 'x.css';".into())]).has_errors());
        assert!(with(&[(
            "assets/course.css",
            "a{background:url(https://x/y.png)}".into()
        )])
        .has_errors());
        assert!(!with(&[(
            "assets/course.css",
            "a{background:url(\"data:image/svg+xml,%3Csvg%3E\")}".into()
        )])
        .has_errors());
    }

    #[test]
    fn allows_repository_url_only_as_text() {
        let ok = page("<p>Source: https://github.com/godspeed-you/ono-sendai</p>");
        assert!(!with(&[("about.html", ok)]).has_errors());
        let link = page("<a href=\"https://github.com/godspeed-you/ono-sendai\">repo</a>");
        assert!(with(&[("about.html", link)]).has_errors());
        let other = page("<p>https://example.com</p>");
        assert!(with(&[("about.html", other)]).has_errors());
    }

    #[test]
    fn allows_svg_namespace_and_repository_in_metadata_only() {
        let svg =
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1 1\"></svg>".to_string();
        assert!(!with(&[("assets/favicon.svg", svg)]).has_errors());
        let remote = "<svg xmlns=\"http://www.w3.org/2000/svg\"><image href=\"https://example.com/x.png\"/></svg>";
        assert!(with(&[("assets/favicon.svg", remote.to_string())]).has_errors());
        let json = "{\"repository\": \"https://github.com/godspeed-you/ono-sendai\"}".to_string();
        assert!(!with(&[("course-metadata.json", json)]).has_errors());
        let other = "{\"x\": \"https://example.com\"}".to_string();
        assert!(with(&[("course-metadata.json", other)]).has_errors());
        let ns_in_html = page("<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>");
        assert!(with(&[("index.html", ns_in_html)]).has_errors());
    }

    #[test]
    fn multibyte_text_before_a_url_does_not_panic() {
        let body = page("<p>⋯⋯⋯⋯ https://github.com/godspeed-you/ono-sendai</p>");
        assert!(!with(&[("about.html", body)]).has_errors());
    }

    #[test]
    fn allows_urls_inside_displayed_source_text_only() {
        let src = page("<pre class=\"source\"><code>// see https://example.com/spec</code></pre>");
        assert!(!with(&[("lessons/a.html", src)]).has_errors());
        let attr = page("<pre class=\"source\"><a href=\"https://example.com\">x</a></pre>");
        assert!(with(&[("lessons/a.html", attr)]).has_errors());
    }

    #[test]
    fn detects_broken_links_and_anchors() {
        let d = with(&[("index.html", page("<a href=\"lessons/missing.html\">x</a>"))]);
        assert!(d.has_error_containing("does not exist"));
        let d = with(&[("index.html", page("<a href=\"lessons/a.html#nope\">x</a>"))]);
        assert!(d.has_error_containing("no id `nope`"));
        let d = with(&[("index.html", page("<a href=\"lessons/a.html#x\">x</a>"))]);
        assert!(!d.has_errors(), "{}", d.report());
    }

    #[test]
    fn detects_duplicate_ids_and_missing_h1_and_zoom_block() {
        assert!(
            with(&[("index.html", page("<p id=\"a\"></p><p id=\"a\"></p>"))])
                .has_error_containing("duplicate id")
        );
        let no_h1 = "<!doctype html><html lang=\"en\"><title>t</title></html>".to_string();
        assert!(with(&[("index.html", no_h1)]).has_error_containing("exactly one <h1>"));
        let zoom =
            page("<meta name=\"viewport\" content=\"width=device-width, user-scalable=no\">");
        assert!(with(&[("index.html", zoom)]).has_error_containing("pinch zoom"));
    }

    #[test]
    fn detects_unexpected_file_types() {
        let d = with(&[("assets/font.woff2", String::new())]);
        assert!(d.has_error_containing("unexpected file type"));
    }
}
