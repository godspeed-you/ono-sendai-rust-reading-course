//! Rendering of real Ono-Sendai snippets (spec §19–§20, §40, §48, §94).
//!
//! A snippet becomes a `<figure class="code-figure">` with a compact provenance line, an
//! expandable source-details block, a horizontally scrollable code region with real line numbers,
//! visible omission rows between segments, and (optionally) numbered annotations. See
//! `docs/frontend.md` for the markup contract.

use super::Site;
use crate::highlight;
use crate::html::escape;
use crate::model::{Annotation, LineSpec, Snippet};
use crate::snippet::source_lines;

/// Per-page rendering state: figure ids and code region labels must be unique on a page even
/// when a snippet is shown twice.
#[derive(Default)]
pub(crate) struct PageCtx {
    pub figures: usize,
    /// Accessible names of the code regions already on the page.
    pub region_labels: Vec<String>,
}

pub(crate) struct CodeSpec<'a> {
    pub snippet: &'a Snippet,
    pub highlight: &'a [LineSpec],
    pub annotations: &'a [Annotation],
    /// Markdown caption shown under the code.
    pub caption: Option<&'a str>,
}

enum Row {
    Line { n: u32, text: String, html: String },
    Omitted { from: u32, to: u32 },
}

/// Wrap the raw-text byte range `start..end` of one highlighted line in `<mark>` elements.
///
/// `html` is the escaped, highlighted HTML of the line (text, entities and `<span>` tags); the
/// range refers to the unescaped source text. The mark is closed before every tag and reopened
/// after it, so the result is always well nested no matter how the range crosses token spans.
/// `open_tag` is the complete opening tag to use, e.g. `<mark class="ann-token">`.
pub fn mark_range(html: &str, start: usize, end: usize, open_tag: &str) -> String {
    let mut out = String::with_capacity(html.len() + 32);
    let mut pos = 0usize; // position in raw text
    let mut open = false;
    let mut rest = html;
    while let Some(c) = rest.chars().next() {
        if c == '<' {
            let tag_end = rest.find('>').map_or(rest.len(), |e| e + 1);
            if open {
                out.push_str("</mark>");
                open = false;
            }
            out.push_str(&rest[..tag_end]);
            rest = &rest[tag_end..];
            continue;
        }
        let (unit, raw_len) = if c == '&' {
            match rest.find(';') {
                Some(e) if e <= 10 => {
                    let entity = &rest[..=e];
                    (entity, decoded_len(entity))
                }
                _ => (&rest[..1], 1),
            }
        } else {
            (&rest[..c.len_utf8()], c.len_utf8())
        };
        let inside = pos >= start && pos < end;
        if inside && !open {
            out.push_str(open_tag);
            open = true;
        } else if !inside && open {
            out.push_str("</mark>");
            open = false;
        }
        out.push_str(unit);
        pos += raw_len;
        rest = &rest[unit.len()..];
    }
    if open {
        out.push_str("</mark>");
    }
    out
}

/// Byte length of the character an HTML entity stands for.
fn decoded_len(entity: &str) -> usize {
    let body = &entity[1..entity.len() - 1];
    let code = if let Some(hex) = body.strip_prefix("#x").or_else(|| body.strip_prefix("#X")) {
        u32::from_str_radix(hex, 16).ok()
    } else if let Some(dec) = body.strip_prefix('#') {
        dec.parse().ok()
    } else {
        None
    };
    code.and_then(char::from_u32).map_or(1, char::len_utf8)
}

fn rows(snippet: &Snippet) -> Result<Vec<Row>, String> {
    let mut rows = Vec::new();
    let mut prev_end: Option<u32> = None;
    for seg in &snippet.segments {
        if let Some(pe) = prev_end {
            if seg.start_line > pe + 1 {
                rows.push(Row::Omitted {
                    from: pe + 1,
                    to: seg.start_line - 1,
                });
            }
        }
        let texts = source_lines(&seg.code);
        let htmls = highlight::highlight_rust(&seg.code);
        if texts.len() != htmls.len() || texts.len() as u32 != seg.end_line - seg.start_line + 1 {
            return Err(format!(
                "snippet `{}`: segment {}-{} does not contain the expected number of lines",
                snippet.id, seg.start_line, seg.end_line
            ));
        }
        for (i, (t, h)) in texts.iter().zip(htmls).enumerate() {
            rows.push(Row::Line {
                n: seg.start_line + i as u32,
                text: (*t).to_string(),
                html: h,
            });
        }
        prev_end = Some(seg.end_line);
    }
    Ok(rows)
}

/// "40–95" or "40–60, 96–120": the displayed line ranges of a snippet.
fn ranges_text(snippet: &Snippet) -> String {
    snippet
        .segments
        .iter()
        .map(|s| {
            LineSpec {
                start: s.start_line,
                end: s.end_line,
            }
            .to_string()
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn plural_lines(n: u32) -> &'static str {
    if n == 1 {
        "line"
    } else {
        "lines"
    }
}

/// The compact provenance line: `Ono-Sendai v0.6.2 · crates/x.rs:40–95 · commit cc61329`.
pub(crate) fn provenance_line(site: &Site, snippet: &Snippet) -> String {
    let span = LineSpec {
        start: snippet.first_line(),
        end: snippet.last_line(),
    };
    let commit = &snippet.source.commit;
    let short = &commit[..commit.len().min(7)];
    let shortened = if snippet.is_shortened() {
        " <span class=\"badge badge-shortened\">Shortened</span>"
    } else {
        ""
    };
    format!(
        "<p class=\"provenance\"><span class=\"prov-ono\">{ono}</span> · <span class=\"prov-file\"><code>{file}</code>:{span}</span> · <span class=\"prov-commit\">commit <code>{short}</code></span>{shortened}</p>",
        ono = escape(&site.ono_label()),
        file = escape(&snippet.source.file),
        short = escape(short),
    )
}

fn source_details(site: &Site, snippet: &Snippet) -> String {
    let mut out = String::from("<details class=\"source-details\"><summary>Source details</summary>\n<dl class=\"source-meta\">\n");
    out.push_str(&format!(
        "<dt>File</dt><dd><code>{}</code></dd>\n",
        escape(&snippet.source.file)
    ));
    out.push_str(&format!(
        "<dt>Lines shown</dt><dd>{}</dd>\n",
        ranges_text(snippet)
    ));
    out.push_str(&format!(
        "<dt>Commit</dt><dd><code class=\"hash\">{}</code></dd>\n",
        escape(&snippet.source.commit)
    ));
    if let Some(v) = site
        .course
        .lock
        .ono_sendai
        .version
        .as_deref()
        .filter(|v| !v.trim().is_empty())
    {
        out.push_str(&format!(
            "<dt>Ono-Sendai version</dt><dd>{}</dd>\n",
            escape(v)
        ));
    }
    out.push_str(&format!(
        "<dt>Snippet id</dt><dd><code>{}</code></dd>\n",
        escape(&snippet.id)
    ));
    out.push_str("<dt>Segments</dt><dd><ul class=\"segments\">");
    for seg in &snippet.segments {
        out.push_str(&format!(
            "<li>Lines {range} · anchor <code>{anchor}</code> · <code class=\"hash\">{hash}</code></li>",
            range = LineSpec { start: seg.start_line, end: seg.end_line },
            anchor = escape(&seg.anchor),
            hash = escape(&seg.content_hash),
        ));
    }
    out.push_str("</ul></dd>\n</dl>\n");
    if snippet.is_shortened() {
        let gaps: Vec<String> = snippet
            .segments
            .windows(2)
            .filter(|w| w[1].start_line > w[0].end_line + 1)
            .map(|w| {
                LineSpec {
                    start: w[0].end_line + 1,
                    end: w[1].start_line - 1,
                }
                .to_string()
            })
            .collect();
        let omitted: u32 = snippet
            .segments
            .windows(2)
            .map(|w| w[1].start_line.saturating_sub(w[0].end_line + 1))
            .sum();
        out.push_str(&format!(
            "<p class=\"shortened-note\"><strong>Shortened.</strong> {omitted} {} of the original file {} not shown ({}). Each omission is marked in the code; every line that is shown is exact.</p>\n",
            plural_lines(omitted),
            if omitted == 1 { "is" } else { "are" },
            gaps.join(", "),
        ));
    }
    out.push_str("<p class=\"exact-note\">The code is quoted exactly from this commit. Line numbers are the real line numbers in the file.</p>\n</details>");
    out
}

/// Render one snippet as a code figure.
pub(crate) fn code_figure(
    site: &Site,
    ctx: &mut PageCtx,
    root: &str,
    spec: &CodeSpec,
) -> Result<String, String> {
    ctx.figures += 1;
    let fig = format!("f{}", ctx.figures);
    let snippet = spec.snippet;
    let rows = rows(snippet)?;
    let anns = spec.annotations;

    // Annotation markers: how many annotations start on each line (for the gutter width).
    let max_markers = (snippet.first_line()..=snippet.last_line())
        .map(|l| anns.iter().filter(|a| a.lines.start == l).count())
        .max()
        .unwrap_or(0);

    let mut code = String::new();
    for row in &rows {
        match row {
            Row::Line { n, text, html } => {
                let n = *n;
                let hl = spec.highlight.iter().any(|h| h.contains(n));
                let covering: Vec<String> = anns
                    .iter()
                    .enumerate()
                    .filter(|(_, a)| a.lines.contains(n))
                    .map(|(i, _)| (i + 1).to_string())
                    .collect();
                let mut class = String::from("line");
                if hl {
                    class.push_str(" hl");
                }
                if !covering.is_empty() {
                    class.push_str(" ann");
                }
                let data_ann = if covering.is_empty() {
                    String::new()
                } else {
                    format!(" data-ann=\"{}\"", covering.join(" "))
                };
                // Token marks for annotations starting on this line.
                let mut line_html = html.clone();
                for (i, a) in anns.iter().enumerate() {
                    if a.lines.start != n {
                        continue;
                    }
                    if let Some(tok) = a.token.as_deref().filter(|t| !t.is_empty()) {
                        if let Some(at) = text.find(tok) {
                            let open = format!("<mark class=\"ann-token\" data-ann=\"{}\">", i + 1);
                            line_html = mark_range(&line_html, at, at + tok.len(), &open);
                        }
                    }
                }
                let mut gutter = String::from("<span class=\"gut\">");
                if !anns.is_empty() {
                    gutter.push_str("<span class=\"marks\">");
                    for (i, a) in anns.iter().enumerate() {
                        if a.lines.start == n {
                            let k = i + 1;
                            gutter.push_str(&format!(
                                "<a class=\"ann-marker\" href=\"#{fig}-ann-{k}\" data-ann=\"{k}\" data-n=\"{k}\" aria-label=\"Annotation {k}: {kind}, {lines}\"></a>",
                                kind = escape(a.kind.label()),
                                lines = lines_label(&a.lines),
                            ));
                        }
                    }
                    gutter.push_str("</span>");
                }
                gutter.push_str(&format!(
                    "<span class=\"ln\" data-n=\"{n}\" aria-hidden=\"true\"></span></span>"
                ));
                code.push_str(&format!(
                    "<span class=\"{class}\" data-line=\"{n}\"{data_ann}>{gutter}<span class=\"lc\">{line_html}</span></span>"
                ));
            }
            Row::Omitted { from, to } => {
                let count = to - from + 1;
                let marks = if anns.is_empty() {
                    ""
                } else {
                    "<span class=\"marks\"></span>"
                };
                code.push_str(&format!(
                    "<span class=\"line omitted\" data-omitted=\"{from}-{to}\"><span class=\"gut\">{marks}<span class=\"ln\" data-n=\"⋯\" aria-hidden=\"true\"></span></span><span class=\"lc\">⋯ {count} {} omitted ({})</span></span>",
                    plural_lines(count),
                    LineSpec { start: *from, end: *to },
                ));
            }
        }
    }

    let mut classes = String::from("code-figure");
    if !anns.is_empty() {
        classes.push_str(" has-ann");
    }
    if snippet.is_shortened() {
        classes.push_str(" is-shortened");
    }
    // Gutter sizing: digits of the largest line number, and marker columns when several
    // annotations start on the same line.
    let digits = snippet.last_line().max(1).to_string().len();
    let style = if max_markers > 1 {
        format!(" style=\"--ln-digits:{digits};--ann-cols:{max_markers}\"")
    } else {
        format!(" style=\"--ln-digits:{digits}\"")
    };
    let mut out = format!(
        "<figure class=\"{classes}\" id=\"{fig}\" data-snippet=\"{sid}\"{style}>\n<div class=\"code-head\">\n{prov}\n{details}\n</div>\n",
        sid = escape(&snippet.id),
        prov = provenance_line(site, snippet),
        details = source_details(site, snippet),
    );
    if !spec.highlight.is_empty() {
        let list: Vec<String> = spec.highlight.iter().map(ToString::to_string).collect();
        out.push_str(&format!(
            "<p class=\"hl-note\"><span class=\"hl-swatch\" aria-hidden=\"true\"></span>Highlighted {}: {}</p>\n",
            if spec.highlight.len() == 1 && spec.highlight[0].start == spec.highlight[0].end { "line" } else { "lines" },
            list.join(", ")
        ));
    }
    out.push_str("<div class=\"code-tools\" hidden><button type=\"button\" class=\"btn btn-small code-wrap\" aria-pressed=\"false\">Wrap long lines</button>");
    if !anns.is_empty() {
        out.push_str("<button type=\"button\" class=\"btn btn-small ann-all\" data-state=\"collapsed\">Expand all annotations</button>");
    }
    out.push_str("</div>\n");
    let span = LineSpec {
        start: snippet.first_line(),
        end: snippet.last_line(),
    };
    let mut label = format!(
        "Source code: {file}, {lines}{short}",
        file = snippet.source.file,
        lines = lines_label(&span),
        short = if snippet.is_shortened() {
            ", shortened"
        } else {
            ""
        },
    );
    // Landmarks need unique names: a snippet shown again on the same page says so.
    let seen = ctx
        .region_labels
        .iter()
        .filter(|l| l.as_str() == label)
        .count();
    ctx.region_labels.push(label.clone());
    if seen > 0 {
        label.push_str(&format!(" (view {})", seen + 1));
    }
    out.push_str(&format!(
        "<div class=\"code-scroll\" tabindex=\"0\" role=\"region\" aria-label=\"{}\">",
        escape(&label)
    ));
    out.push_str("<pre class=\"source\" translate=\"no\"><code>");
    out.push_str(&code);
    out.push_str("</code></pre></div>\n");

    if !anns.is_empty() {
        out.push_str(&format!(
            "<div class=\"ann-panel\">\n<p class=\"ann-heading\" id=\"{fig}-ann-h\">Annotations</p>\n<ol class=\"annotations\" aria-labelledby=\"{fig}-ann-h\">\n"
        ));
        for (i, a) in anns.iter().enumerate() {
            let k = i + 1;
            let body = site.md(
                &a.body,
                root,
                &format!("snippet `{}` annotation {k}", snippet.id),
            )?;
            let token = match a.token.as_deref().filter(|t| !t.is_empty()) {
                Some(t) => format!(" <code class=\"ann-token-label\">{}</code>", escape(t)),
                None => String::new(),
            };
            out.push_str(&format!(
                "<li><details class=\"annotation ann-kind-{slug}\" id=\"{fig}-ann-{k}\" data-ann=\"{k}\" data-lines=\"{s}-{e}\"><summary><span class=\"ann-num\">{k}</span><span class=\"ann-meta\"><span class=\"ann-kind\">{kind}</span> <span class=\"ann-lines\">{lines}</span>{token}</span></summary>\n<div class=\"ann-body\">\n{body}</div>\n</details></li>\n",
                slug = a.kind.slug(),
                s = a.lines.start,
                e = a.lines.end,
                kind = escape(a.kind.label()),
                lines = capitalize(&lines_label(&a.lines)),
            ));
        }
        out.push_str("</ol>\n</div>\n");
    }
    if let Some(c) = spec.caption {
        let caption = site.md(c, root, &format!("snippet `{}` caption", snippet.id))?;
        out.push_str(&format!(
            "<figcaption class=\"code-caption\">\n{caption}</figcaption>\n"
        ));
    }
    out.push_str("</figure>");
    Ok(out)
}

/// "line 12" or "lines 12–14".
fn lines_label(l: &LineSpec) -> String {
    if l.start == l.end {
        format!("line {}", l.start)
    } else {
        format!("lines {l}")
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const M: &str = "<mark>";

    #[test]
    fn marks_plain_text() {
        assert_eq!(mark_range("let x = y;", 4, 5, M), "let <mark>x</mark> = y;");
    }

    #[test]
    fn marks_inside_a_span() {
        let html = "<span class=\"tok-kw\">fn</span> run(<span class=\"tok-kw\">self</span>)";
        assert_eq!(
            mark_range(html, 7, 11, M),
            "<span class=\"tok-kw\">fn</span> run(<span class=\"tok-kw\"><mark>self</mark></span>)"
        );
    }

    #[test]
    fn marks_across_spans_stay_well_nested() {
        // `&self` = plain "&" (escaped) followed by a keyword span.
        let src = "fn f(&self) {}";
        let html = highlight::highlight_rust(src).remove(0);
        let at = src.find("&self").unwrap();
        let out = mark_range(&html, at, at + 5, M);
        assert!(
            out.contains("<mark>&amp;</mark><span class=\"tok-kw\"><mark>self</mark></span>"),
            "{out}"
        );
        assert_eq!(
            out.matches("<mark>").count(),
            out.matches("</mark>").count()
        );
    }

    #[test]
    fn entities_count_as_one_character() {
        let src = "a < b && c";
        let html = crate::html::escape(src);
        let at = src.find("&&").unwrap();
        assert_eq!(
            mark_range(&html, at, at + 2, M),
            "a &lt; b <mark>&amp;&amp;</mark> c"
        );
    }

    #[test]
    fn multibyte_text_positions_are_bytes() {
        let src = "// ⋯ ünïcode token";
        let at = src.find("token").unwrap();
        let html = highlight::highlight_rust(src).remove(0);
        let out = mark_range(&html, at, at + 5, M);
        assert!(out.contains("<mark>token</mark>"), "{out}");
    }

    #[test]
    fn marking_text_is_lossless() {
        let src = "    pub fn get<'a>(&'a self, key: &str) -> Option<&'a Value> { \"<x>\" }";
        let html = highlight::highlight_rust(src).remove(0);
        for (s, e) in [(0, 3), (4, 20), (10, src.len()), (src.len() - 3, src.len())] {
            let out = mark_range(&html, s, e, M);
            assert_eq!(out.replace("<mark>", "").replace("</mark>", ""), html);
        }
    }
}
