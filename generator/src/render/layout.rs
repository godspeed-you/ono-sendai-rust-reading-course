//! The page skeleton shared by every page: head, header, course navigator and footer.

use super::{root_of, Site};
use crate::html::{escape, fill};
use crate::model::Stage;
use crate::paths;

const PAGE_TEMPLATE: &str = include_str!("../../templates/page.html");

/// What the navigator should mark as current.
#[derive(Clone, Copy, Default)]
pub(crate) struct Current<'a> {
    /// Output path of the page itself (for `aria-current` on site links).
    pub path: &'a str,
    pub chapter: Option<usize>,
    pub lesson: Option<&'a str>,
}

pub(crate) struct Page<'a> {
    /// Output path relative to the site root, e.g. `lessons/x.html`.
    pub path: &'a str,
    /// Plain-text page title (without the course name).
    pub title: &'a str,
    /// Plain-text description for `<meta name="description">`.
    pub description: &'a str,
    /// Page kind: `home`, `chapter`, `lesson`, `learn-rust`, `ono-sendai`, `glossary`, `about`.
    pub kind: &'static str,
    /// Extra `data-*` attributes for `<body>`: already escaped, each starting with a space.
    pub body_attrs: String,
    pub main: String,
    pub current: Current<'a>,
}

/// Site-wide links, shown in the header on wide screens and in the navigator on narrow ones.
pub(crate) const SITE_LINKS: &[(&str, &str)] = &[
    (paths::INDEX, "Course home"),
    (paths::LEARN_RUST, "Learn Rust"),
    (paths::ONO_SENDAI, "Understand Ono-Sendai"),
    (paths::GLOSSARY, "Glossary"),
    (paths::ABOUT, "About"),
];

fn site_links(root: &str, current: &str) -> String {
    let mut out = String::new();
    for (path, label) in SITE_LINKS {
        let cur = if *path == current {
            " aria-current=\"page\""
        } else {
            ""
        };
        out.push_str(&format!(
            "<li><a href=\"{root}{path}\"{cur}>{}</a></li>\n",
            escape(label)
        ));
    }
    out.trim_end().to_string()
}

/// A compact stage marker for lists: "S1" with the full name available to screen readers.
pub(crate) fn stage_chip(stage: Stage) -> String {
    format!(
        "<span class=\"stage-chip stage-{slug}\"><span class=\"vh\">Stage </span>{n}<span class=\"vh\">: {title}</span></span>",
        slug = stage.slug(),
        n = stage.number(),
        title = escape(stage.title()),
    )
}

/// A visible stage label: "Stage 1 · Guided reading".
pub(crate) fn stage_label(stage: Stage) -> String {
    format!(
        "<span class=\"stage-label stage-{}\">Stage {} · {}</span>",
        stage.slug(),
        stage.number(),
        escape(stage.title())
    )
}

/// "Stage 2" or "Stages 1–3" for a range.
pub(crate) fn stage_range_text(lo: Stage, hi: Stage) -> String {
    if lo == hi {
        format!("Stage {} · {}", lo.number(), lo.title())
    } else {
        format!(
            "Stages {}–{} · {} to {}",
            lo.number(),
            hi.number(),
            lo.title(),
            hi.title().to_lowercase()
        )
    }
}

/// The course navigator: site links (narrow screens) and the chapter/lesson tree.
fn course_nav(site: &Site, root: &str, current: Current) -> String {
    let mut out = String::new();
    out.push_str("<nav id=\"course-nav\" class=\"course-nav\" aria-label=\"Course contents\">\n");
    out.push_str("<div class=\"nav-head\"><p class=\"nav-title\">Course contents</p><button type=\"button\" class=\"btn btn-small nav-close\" hidden>Close menu</button></div>\n");
    out.push_str("<ul class=\"nav-site\">\n");
    out.push_str(&site_links(root, current.path));
    out.push_str("\n</ul>\n<ol class=\"nav-chapters\">\n");
    for (ci, chapter) in site.chapters().enumerate() {
        let is_current = current.chapter == Some(ci);
        let open = if is_current { " open" } else { "" };
        let chapter_path = paths::chapter(chapter.number, &chapter.id);
        let overview_cur = if current.path == chapter_path {
            " aria-current=\"page\""
        } else {
            ""
        };
        out.push_str(&format!(
            "<li><details class=\"nav-chapter\"{open}><summary><span class=\"nav-ch-num\">{n}</span> <span class=\"nav-ch-title\">{title}</span></summary>\n<ol class=\"nav-lessons\">\n<li class=\"nav-overview\"><a href=\"{root}{chapter_path}\"{overview_cur}>Chapter overview</a></li>\n",
            n = chapter.number,
            title = escape(&chapter.title),
        ));
        for l in site.chapter_lessons(ci) {
            let id = &l.lesson.id;
            let cur = if current.lesson == Some(id.as_str()) {
                " aria-current=\"page\""
            } else {
                ""
            };
            out.push_str(&format!(
                "<li data-lesson-id=\"{eid}\"><a href=\"{root}{path}\"{cur}>{chip}<span class=\"nav-l-title\">{title}</span></a></li>\n",
                eid = escape(id),
                path = paths::lesson(id),
                chip = stage_chip(l.lesson.stage),
                title = escape(&l.lesson.title),
            ));
        }
        out.push_str("</ol>\n</details></li>\n");
    }
    out.push_str("</ol>\n</nav>");
    out
}

/// Fill the page template.
pub(crate) fn page(site: &Site, p: Page) -> String {
    let root = root_of(p.path);
    let lock = &site.course.lock;
    let course_title = &site.course.curriculum.title;
    let full_title = if p.kind == "home" {
        course_title.clone()
    } else {
        format!("{} · {}", p.title, course_title)
    };
    let ono_label = match &lock.ono_sendai.version {
        Some(v) if !v.trim().is_empty() => format!("Ono-Sendai <strong>{}</strong>", escape(v)),
        _ => "Ono-Sendai".to_string(),
    };
    let nav = course_nav(site, root, p.current);
    let links = site_links(root, p.current.path);
    fill(
        PAGE_TEMPLATE,
        &[
            ("title", &escape(&full_title)),
            ("description", &escape(p.description)),
            ("generator_version", &escape(super::GENERATOR_VERSION)),
            ("root", root),
            ("kind", p.kind),
            ("body_attrs", &p.body_attrs),
            ("site_links", &links),
            ("main", p.main.trim_end()),
            ("nav", &nav),
            ("course_version", &escape(&lock.course.version)),
            ("ono_label", &ono_label),
            ("short_commit", &escape(site.short_commit())),
        ],
    )
}

/// A list of tag links (concepts or topics).
pub(crate) fn tag_list(items: &[(String, String)], class: &str, label: &str) -> String {
    let mut out = format!(
        "<ul class=\"tags {class}\" aria-label=\"{}\">",
        escape(label)
    );
    for (href, text) in items {
        out.push_str(&format!(
            "<li><a class=\"tag\" href=\"{href}\">{}</a></li>",
            escape(text)
        ));
    }
    out.push_str("</ul>");
    out
}

/// Concept tags of a lesson, linked to the "Learn Rust" page.
pub(crate) fn concept_tags(site: &Site, ids: &[String], root: &str) -> String {
    let items: Vec<(String, String)> = ids
        .iter()
        .filter_map(|id| site.course.curriculum.concepts.iter().find(|c| &c.id == id))
        .map(|c| {
            (
                format!(
                    "{root}{}#{}",
                    paths::LEARN_RUST,
                    paths::concept_anchor(&c.id)
                ),
                c.title.clone(),
            )
        })
        .collect();
    tag_list(&items, "tags-concepts", "Rust concepts")
}

/// Ono-Sendai topic tags of a lesson, linked to the "Understand Ono-Sendai" page.
pub(crate) fn topic_tags(site: &Site, ids: &[String], root: &str) -> String {
    let items: Vec<(String, String)> = ids
        .iter()
        .filter_map(|id| {
            site.course
                .curriculum
                .ono_topics
                .iter()
                .find(|c| &c.id == id)
        })
        .map(|c| {
            (
                format!("{root}{}#{}", paths::ONO_SENDAI, paths::topic_anchor(&c.id)),
                c.title.clone(),
            )
        })
        .collect();
    tag_list(&items, "tags-topics", "Ono-Sendai topics")
}
