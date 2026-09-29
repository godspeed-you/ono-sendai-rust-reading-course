//! One page per lesson (spec §9, §30, §48–§50, §56).

use super::code::{code_figure, CodeSpec, PageCtx};
use super::exercise::{exercise, ExerciseCtx};
use super::layout::{concept_tags, page, topic_tags, Current, Page};
use super::{text_html, text_plain, LessonRef, Site};
use crate::html::escape;
use crate::model::{ProseKind, Section, Stage};
use crate::paths;

/// A short visual symbol per prose kind; always shown next to the text label (never alone).
fn prose_icon(kind: ProseKind) -> &'static str {
    match kind {
        ProseKind::Context => "›",
        ProseKind::Rust => "{ }",
        ProseKind::Ono => "◆",
        ProseKind::Note => "i",
        ProseKind::BlackBox => "■",
        ProseKind::Recap => "✓",
    }
}

pub(crate) fn lesson_page(site: &Site, index: usize) -> Result<String, String> {
    let l: &LessonRef = &site.lessons[index];
    let lesson = l.lesson;
    let path = paths::lesson(&lesson.id);
    let root = "../";
    let ctx = format!("lesson `{}`", lesson.id);
    let chapter = l.chapter;
    let in_chapter = chapter.lessons.len();
    let total = site.lessons.len();
    let chapter_href = format!("{root}{}", paths::chapter(chapter.number, &chapter.id));
    let stage = lesson.stage;

    let mut m = String::new();
    m.push_str(&format!(
        "<nav class=\"breadcrumb\" aria-label=\"Breadcrumb\"><ol><li><a href=\"{root}{index}\">Course home</a></li><li><a href=\"{chapter_href}\">Chapter {n}: {ct}</a></li><li><span aria-current=\"page\">Lesson {k} of {in_chapter}</span></li></ol></nav>\n",
        index = paths::INDEX,
        n = chapter.number,
        ct = text_html(&chapter.title),
        k = l.index_in_chapter + 1,
    ));
    m.push_str(&format!(
        "<header class=\"page-head lesson-head\">\n<p class=\"kicker\">Chapter {n} · Lesson {k} of {in_chapter} <span class=\"kicker-overall\">· Lesson {g} of {total} in the course</span></p>\n<h1>{title}</h1>\n<p class=\"lede\">{summary}</p>\n<div class=\"stage-badge stage-{slug}\" data-stage=\"{slug}\"><p class=\"stage-badge-name\">Stage {sn} of 5 · <strong>{stitle}</strong></p><p class=\"stage-badge-support\">{support}</p></div>\n</header>\n",
        n = chapter.number,
        k = l.index_in_chapter + 1,
        g = l.global + 1,
        title = text_html(&lesson.title),
        summary = text_html(&lesson.summary),
        slug = stage.slug(),
        sn = stage.number(),
        stitle = escape(stage.title()),
        support = escape(stage.support()),
    ));

    // Overview: objectives, prerequisites, tags.
    m.push_str("<section class=\"lesson-overview\" aria-labelledby=\"overview-h\">\n<h2 id=\"overview-h\">In this lesson</h2>\n<dl class=\"overview\">\n<dt>You will learn to</dt>\n<dd><ul class=\"objectives\">\n");
    for o in &lesson.objectives {
        m.push_str(&format!(
            "<li>{}</li>\n",
            site.md_inline(o, root, &format!("{ctx} objective"))?
        ));
    }
    m.push_str("</ul></dd>\n<dt>Before this lesson</dt>\n<dd>");
    if lesson.prerequisites.is_empty() {
        m.push_str("<p>No prerequisites beyond the lessons before it.</p>");
    } else {
        m.push_str("<ul class=\"prereqs\">");
        for p in &lesson.prerequisites {
            let pl = site
                .lesson(p)
                .ok_or_else(|| format!("{ctx}: unknown prerequisite `{p}`"))?;
            m.push_str(&format!(
                "<li><a href=\"{root}{}\">{}</a> <span class=\"muted\">(Chapter {}, lesson {})</span></li>",
                paths::lesson(p),
                text_html(&pl.lesson.title),
                pl.chapter.number,
                pl.index_in_chapter + 1
            ));
        }
        m.push_str("</ul>");
    }
    m.push_str("</dd>\n");
    m.push_str(&format!(
        "<dt>Rust concepts</dt>\n<dd>{}</dd>\n",
        concept_tags(site, &lesson.concepts, root)
    ));
    m.push_str(&format!(
        "<dt>Ono-Sendai topics</dt>\n<dd>{}</dd>\n",
        topic_tags(site, &lesson.ono_topics, root)
    ));
    m.push_str("</dl>\n</section>\n");

    // Sections in order.
    let mut pctx = PageCtx::default();
    let mut exercise_no = 0;
    m.push_str("<section class=\"lesson-body\" aria-labelledby=\"reading-h\">\n<h2 id=\"reading-h\">Reading</h2>\n");
    for (si, section) in lesson.sections.iter().enumerate() {
        let sctx = format!("{ctx} section {}", si + 1);
        match section {
            Section::Prose { kind, title, body } => {
                let heading = match title {
                    Some(t) => format!("<h3>{}</h3>\n", text_html(t)),
                    None => String::new(),
                };
                m.push_str(&format!(
                    "<div class=\"prose prose-{slug}\" data-kind=\"{slug}\">\n<p class=\"prose-label\"><span class=\"prose-icon\" aria-hidden=\"true\">{icon}</span>{label}</p>\n{heading}{body}</div>\n",
                    slug = kind.slug(),
                    icon = escape(prose_icon(*kind)),
                    label = escape(kind.label()),
                    body = site.md(body, root, &sctx)?,
                ));
            }
            Section::Code {
                snippet,
                caption,
                highlight,
                annotations,
            } => {
                let (_, snip) = site
                    .course
                    .snippets
                    .get(snippet)
                    .ok_or_else(|| format!("{sctx}: unknown snippet `{snippet}`"))?;
                let spec = CodeSpec {
                    snippet: snip,
                    highlight,
                    annotations,
                    caption: caption.as_deref(),
                };
                m.push_str(&code_figure(site, &mut pctx, root, &spec)?);
                m.push('\n');
            }
            Section::Diagram {
                kind,
                title,
                art,
                description,
            } => {
                let slug = kind.label().to_lowercase().replace(' ', "-");
                m.push_str(&format!(
                    "<figure class=\"diagram {slug}\">\n<figcaption><span class=\"diagram-kind\">{klabel}</span> <span class=\"diagram-title\">{title}</span></figcaption>\n<div class=\"diagram-scroll\" tabindex=\"0\" role=\"img\" aria-label=\"{klabel}: {plain_title}. A text description follows.\"><pre class=\"diagram-art\" aria-hidden=\"true\">{art}</pre></div>\n<div class=\"diagram-text\">\n<p class=\"diagram-text-label\">Text description</p>\n{desc}</div>\n</figure>\n",
                    klabel = escape(kind.label()),
                    title = text_html(title),
                    plain_title = escape(&text_plain(title)),
                    art = escape(art.trim_end_matches('\n')),
                    desc = site.md(description, root, &sctx)?,
                ));
            }
            Section::Exercise(ex) => {
                exercise_no += 1;
                let cx = ExerciseCtx {
                    stage,
                    number: exercise_no,
                    root,
                };
                m.push_str(&exercise(site, &mut pctx, ex, &cx)?);
                m.push('\n');
            }
        }
    }
    m.push_str("</section>\n");

    // The very last lesson closes the course.
    if index + 1 == total {
        m.push_str(&course_end(site));
    }

    // Completion and prev/next navigation.
    m.push_str(&format!(
        "<div class=\"lesson-end\">\n<div class=\"complete\" hidden><button type=\"button\" class=\"btn mark-complete\" data-lesson=\"{id}\" aria-pressed=\"false\">Mark lesson complete</button><p class=\"complete-status\" role=\"status\"></p></div>\n",
        id = escape(&lesson.id)
    ));
    m.push_str("<nav class=\"pager\" aria-label=\"Previous and next lesson\">\n");
    if index > 0 {
        let p = &site.lessons[index - 1];
        let dir = if p.chapter_index != l.chapter_index {
            format!("Previous lesson · Chapter {}", p.chapter.number)
        } else {
            "Previous lesson".to_string()
        };
        m.push_str(&format!(
            "<a class=\"pager-prev\" rel=\"prev\" href=\"{root}{}\"><span class=\"pager-dir\">{dir}</span><span class=\"pager-title\">{}</span></a>\n",
            paths::lesson(&p.lesson.id),
            text_html(&p.lesson.title)
        ));
    } else {
        m.push_str(&format!(
            "<a class=\"pager-prev\" href=\"{root}{}\"><span class=\"pager-dir\">Back to</span><span class=\"pager-title\">Course home</span></a>\n",
            paths::INDEX
        ));
    }
    if let Some(n) = site.lessons.get(index + 1) {
        let dir = if n.chapter_index != l.chapter_index {
            format!("Next lesson · Chapter {}", n.chapter.number)
        } else {
            "Next lesson".to_string()
        };
        m.push_str(&format!(
            "<a class=\"pager-next\" rel=\"next\" href=\"{root}{}\"><span class=\"pager-dir\">{dir}</span><span class=\"pager-title\">{}</span></a>\n",
            paths::lesson(&n.lesson.id),
            text_html(&n.lesson.title)
        ));
    } else {
        m.push_str(&format!(
            "<a class=\"pager-next\" href=\"{root}{}\"><span class=\"pager-dir\">Finished</span><span class=\"pager-title\">Back to course home</span></a>\n",
            paths::INDEX
        ));
    }
    m.push_str("</nav>\n</div>\n");

    let title = lesson.title.clone();
    let description = format!("{} — {}", lesson.summary, stage_text(stage));
    Ok(page(
        site,
        Page {
            path: &path,
            title: &title,
            description: &description,
            kind: "lesson",
            body_attrs: format!(
                " data-lesson=\"{}\" data-stage=\"{}\"",
                escape(&lesson.id),
                stage.slug()
            ),
            main: m,
            current: Current {
                path: &path,
                chapter: Some(l.chapter_index),
                lesson: Some(&lesson.id),
            },
        },
    ))
}

fn stage_text(stage: Stage) -> String {
    format!("Stage {} of 5: {}", stage.number(), stage.title())
}

/// The closing message of the final lesson (spec §9, Stage 5). The repository URL appears as
/// text only, never as a link: the course makes no external references.
fn course_end(site: &Site) -> String {
    let lock = &site.course.lock.ono_sendai;
    let tag = match lock.version.as_deref().filter(|v| !v.trim().is_empty()) {
        Some(v) => format!(" (tagged <code>{}</code>)", escape(v)),
        None => String::new(),
    };
    let dir = lock
        .repository
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("ono-sendai")
        .trim_end_matches(".git");
    let dir = if dir.is_empty() { "ono-sendai" } else { dir };
    format!(
        "<section class=\"course-end\" aria-labelledby=\"course-end-h\">\n<h2 id=\"course-end-h\">You do not need this course anymore. Open Ono-Sendai.</h2>\n<p>This was the last lesson. From here on, independent source reading <em>is</em> the activity: pick a file, read it the way you read the code in this course, and use the checklist whenever you want to test your own understanding.</p>\n<p>Get the source with Git. The repository is <code class=\"repo-url\">{repo}</code>:</p>\n<pre class=\"commands\" translate=\"no\" tabindex=\"0\"><code>git clone {repo}\ncd {dir}\ngit checkout {commit}</code></pre>\n<p>That commit{tag} is the exact revision every snippet in this course was quoted from. Newer revisions are just as readable — that is the point.</p>\n</section>\n",
        repo = escape(&lock.repository),
        dir = escape(dir),
        commit = escape(&lock.commit),
    )
}
