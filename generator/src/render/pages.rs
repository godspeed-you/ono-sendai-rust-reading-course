//! Course home, chapter overviews, the two navigation axes (spec §8), the glossary (spec §57)
//! and the About page (spec §68).

use super::layout::{
    concept_tags, page, stage_chip, stage_label, stage_range_text, topic_tags, Current, Page,
};
use super::{text_html, LessonRef, Site};
use crate::html::escape;
use crate::model::{Lesson, Stage, Taxon};
use crate::paths;

/// One lesson as a list entry: stage chip + linked title (+ optional extra HTML).
fn lesson_item(l: &LessonRef, root: &str, extra: &str) -> String {
    format!(
        "<li data-lesson-id=\"{id}\"><a href=\"{root}{path}\">{chip}<span class=\"l-title\">{title}</span></a>{extra}</li>\n",
        id = escape(&l.lesson.id),
        path = paths::lesson(&l.lesson.id),
        chip = stage_chip(l.lesson.stage),
        title = text_html(&l.lesson.title),
    )
}

fn versions_kicker(site: &Site) -> String {
    format!(
        "Course {} · reads {} at commit <code>{}</code>",
        escape(&site.course.lock.course.version),
        escape(&site.ono_label()),
        escape(site.short_commit())
    )
}

pub(crate) fn index(site: &Site) -> Result<String, String> {
    let c = &site.course.curriculum;
    let root = "";
    let mut m = String::new();
    m.push_str(&format!(
        "<header class=\"page-head home-head\">\n<p class=\"kicker\">{}</p>\n<h1>{}</h1>\n<p class=\"lede\">{}</p>\n</header>\n",
        versions_kicker(site),
        escape(&c.title),
        escape(&c.tagline)
    ));
    m.push_str("<div class=\"home-actions\">\n");
    if let Some(first) = site.lessons.first() {
        m.push_str(&format!(
            "<a class=\"btn btn-primary start-link\" href=\"{}\">Start with lesson 1: {}</a>\n",
            paths::lesson(&first.lesson.id),
            text_html(&first.lesson.title)
        ));
    }
    m.push_str(
        "<p class=\"continue\" hidden></p>\n<p class=\"progress-summary\" hidden></p>\n</div>\n",
    );

    m.push_str("<section class=\"home-section\" aria-labelledby=\"what-h\">\n<h2 id=\"what-h\">What this course is</h2>\n");
    m.push_str("<p>This course teaches you to <strong>read</strong> Rust by reading a real program: the source code of Ono-Sendai, pinned to one exact commit. It is not a generic Rust tutorial and you will not write toy programs. Every lesson puts a bounded piece of real Ono-Sendai code in front of you, explains the Rust it uses <em>and</em> why Ono-Sendai is built that way, and then asks you to interpret code on your own.</p>\n");
    m.push_str("<p>You need no compiler, editor or Rust installation. The whole course is a set of static pages that works offline on your device.</p>\n</section>\n");

    // Stages.
    m.push_str("<section class=\"home-section\" aria-labelledby=\"stages-h\">\n<h2 id=\"stages-h\">How the support fades</h2>\n<p>The course deliberately removes its own scaffolding. Every lesson belongs to one of five stages; each lesson page shows its stage and what help to expect.</p>\n<ol class=\"stages\">\n");
    for s in Stage::ALL {
        let ls: Vec<&LessonRef> = site
            .lessons
            .iter()
            .filter(|l| l.lesson.stage == s)
            .collect();
        let range = match (ls.first(), ls.last()) {
            (Some(a), Some(b)) if a.global == b.global => format!("lesson {}", a.global + 1),
            (Some(a), Some(b)) => format!("lessons {}–{}", a.global + 1, b.global + 1),
            _ => "no lessons".to_string(),
        };
        m.push_str(&format!(
            "<li class=\"stage-item stage-{slug}\"><p class=\"stage-item-name\"><span class=\"stage-num\">{n}</span> <strong>{title}</strong> <span class=\"muted\">({range})</span></p><p class=\"stage-item-support\">{support}</p></li>\n",
            slug = s.slug(),
            n = s.number(),
            title = escape(s.title()),
            support = escape(s.support()),
        ));
    }
    m.push_str("</ol>\n</section>\n");

    // Chapters grouped by the stage range they cover.
    m.push_str("<section class=\"home-section\" aria-labelledby=\"chapters-h\">\n<h2 id=\"chapters-h\">Chapters</h2>\n");
    let chapters: Vec<_> = site.chapters().collect();
    let mut i = 0;
    while i < chapters.len() {
        let range = site.chapter_stages(chapters[i]);
        let mut j = i;
        while j < chapters.len() && site.chapter_stages(chapters[j]) == range {
            j += 1;
        }
        m.push_str(&format!(
            "<div class=\"chapter-group\">\n<h3 class=\"group-title\">{}</h3>\n<ol class=\"chapter-list\">\n",
            escape(&stage_range_text(range.0, range.1))
        ));
        for (ci, ch) in chapters.iter().enumerate().take(j).skip(i) {
            m.push_str(&format!(
                "<li class=\"chapter-card\">\n<h4><a href=\"{path}\"><span class=\"ch-num\">Chapter {n}</span> {title}</a></h4>\n<div class=\"chapter-summary\">\n{summary}</div>\n<ol class=\"lesson-list\">\n",
                path = paths::chapter(ch.number, &ch.id),
                n = ch.number,
                title = text_html(&ch.title),
                summary = site.md(&ch.summary, root, &format!("chapter `{}` summary", ch.id))?,
            ));
            for l in site.chapter_lessons(ci) {
                m.push_str(&lesson_item(l, root, ""));
            }
            m.push_str("</ol>\n</li>\n");
        }
        m.push_str("</ol>\n</div>\n");
        i = j;
    }
    m.push_str("</section>\n");

    m.push_str("<section class=\"home-section\" aria-labelledby=\"ways-h\">\n<h2 id=\"ways-h\">Other ways in</h2>\n<ul class=\"cards\">\n");
    for (path, title, text) in [
        (paths::LEARN_RUST, "Learn Rust", "Every Rust concept the course teaches, in teaching order, with the lessons that cover it."),
        (paths::ONO_SENDAI, "Understand Ono-Sendai", "The same lessons organised by the part of Ono-Sendai they explain."),
        (paths::GLOSSARY, "Glossary", "Short definitions of the terms you will meet, with lessons that show them in real code."),
        (paths::ABOUT, "About this build", "Versions, the pinned Ono-Sendai commit, licence, and resetting your local progress."),
    ] {
        m.push_str(&format!(
            "<li class=\"card\"><a href=\"{path}\">{title}</a><p>{text}</p></li>\n",
            title = escape(title),
            text = escape(text)
        ));
    }
    m.push_str("</ul>\n</section>\n");

    Ok(page(
        site,
        Page {
            path: paths::INDEX,
            title: &c.title,
            description: &c.tagline,
            kind: "home",
            body_attrs: String::new(),
            main: m,
            current: Current {
                path: paths::INDEX,
                chapter: None,
                lesson: None,
            },
        },
    ))
}

pub(crate) fn chapter(site: &Site, ci: usize) -> Result<String, String> {
    let chapters: Vec<_> = site.chapters().collect();
    let ch = chapters[ci];
    let path = paths::chapter(ch.number, &ch.id);
    let root = "../";
    let (lo, hi) = site.chapter_stages(ch);
    let mut m = String::new();
    m.push_str(&format!(
        "<nav class=\"breadcrumb\" aria-label=\"Breadcrumb\"><ol><li><a href=\"{root}{}\">Course home</a></li><li><span aria-current=\"page\">Chapter {}</span></li></ol></nav>\n",
        paths::INDEX,
        ch.number
    ));
    m.push_str(&format!(
        "<header class=\"page-head chapter-head\">\n<p class=\"kicker\">Chapter {n} of {total} · {stages}</p>\n<h1>{title}</h1>\n<div class=\"lede\">\n{summary}</div>\n</header>\n",
        n = ch.number,
        total = chapters.len(),
        stages = escape(&stage_range_text(lo, hi)),
        title = text_html(&ch.title),
        summary = site.md(&ch.summary, root, &format!("chapter `{}` summary", ch.id))?,
    ));
    if let Some(first) = site.chapter_lessons(ci).next() {
        m.push_str(&format!(
            "<div class=\"home-actions\"><a class=\"btn btn-primary\" href=\"{root}{}\">Start the chapter: {}</a><p class=\"progress-summary\" hidden></p></div>\n",
            paths::lesson(&first.lesson.id),
            text_html(&first.lesson.title)
        ));
    }
    m.push_str("<section aria-labelledby=\"lessons-h\">\n<h2 id=\"lessons-h\">Lessons</h2>\n<ol class=\"lesson-cards\">\n");
    for l in site.chapter_lessons(ci) {
        let lesson = l.lesson;
        m.push_str(&format!(
            "<li class=\"lesson-card\" data-lesson-id=\"{id}\">\n<h3><a href=\"{root}{path}\"><span class=\"l-num\">Lesson {k}</span> <span class=\"l-title\">{title}</span></a></h3>\n<p class=\"lesson-card-stage\">{stage}</p>\n<p class=\"lesson-card-summary\">{summary}</p>\n<div class=\"lesson-card-tags\">{concepts}{topics}</div>\n</li>\n",
            id = escape(&lesson.id),
            path = paths::lesson(&lesson.id),
            k = l.index_in_chapter + 1,
            title = text_html(&lesson.title),
            stage = stage_label(lesson.stage),
            summary = text_html(&lesson.summary),
            concepts = concept_tags(site, &lesson.concepts, root),
            topics = topic_tags(site, &lesson.ono_topics, root),
        ));
    }
    m.push_str("</ol>\n</section>\n");

    m.push_str("<nav class=\"pager\" aria-label=\"Previous and next chapter\">\n");
    match ci.checked_sub(1).map(|p| chapters[p]) {
        Some(p) => m.push_str(&format!(
            "<a class=\"pager-prev\" rel=\"prev\" href=\"{root}{}\"><span class=\"pager-dir\">Previous chapter</span><span class=\"pager-title\">{}</span></a>\n",
            paths::chapter(p.number, &p.id),
            text_html(&p.title)
        )),
        None => m.push_str(&format!(
            "<a class=\"pager-prev\" href=\"{root}{}\"><span class=\"pager-dir\">Back to</span><span class=\"pager-title\">Course home</span></a>\n",
            paths::INDEX
        )),
    }
    if let Some(n) = chapters.get(ci + 1) {
        m.push_str(&format!(
            "<a class=\"pager-next\" rel=\"next\" href=\"{root}{}\"><span class=\"pager-dir\">Next chapter</span><span class=\"pager-title\">{}</span></a>\n",
            paths::chapter(n.number, &n.id),
            text_html(&n.title)
        ));
    }
    m.push_str("</nav>\n");

    let title = format!("Chapter {}: {}", ch.number, ch.title);
    let description = format!("Chapter {} of the course: {}", ch.number, ch.title);
    Ok(page(
        site,
        Page {
            path: &path,
            title: &title,
            description: &description,
            kind: "chapter",
            body_attrs: format!(" data-chapter=\"{}\"", escape(&ch.id)),
            main: m,
            current: Current {
                path: &path,
                chapter: Some(ci),
                lesson: None,
            },
        },
    ))
}

/// Shared renderer for the two axis pages: a table of contents and one section per taxon.
struct Axis<'a> {
    path: &'static str,
    kind: &'static str,
    title: &'static str,
    lede: &'static str,
    taxa: &'a [Taxon],
    anchor: fn(&str) -> String,
    matches: fn(&Lesson, &str) -> bool,
}

fn axis_page(site: &Site, a: Axis) -> Result<String, String> {
    let root = "";
    let mut m = format!(
        "<header class=\"page-head\">\n<h1>{}</h1>\n<p class=\"lede\">{}</p>\n</header>\n",
        escape(a.title),
        escape(a.lede)
    );
    m.push_str(&format!(
        "<nav class=\"toc\" aria-label=\"{} index\">\n<ol>\n",
        escape(a.title)
    ));
    for t in a.taxa {
        m.push_str(&format!(
            "<li><a href=\"#{}\">{}</a></li>\n",
            (a.anchor)(&t.id),
            text_html(&t.title)
        ));
    }
    m.push_str("</ol>\n</nav>\n");
    for t in a.taxa {
        let lessons: Vec<&LessonRef> = site
            .lessons
            .iter()
            .filter(|l| (a.matches)(l.lesson, &t.id))
            .collect();
        m.push_str(&format!(
            "<section class=\"taxon\" id=\"{anchor}\" aria-labelledby=\"{anchor}-h\">\n<h2 id=\"{anchor}-h\">{title}</h2>\n<div class=\"taxon-summary\">\n{summary}</div>\n",
            anchor = (a.anchor)(&t.id),
            title = text_html(&t.title),
            summary = site.md(&t.summary, root, &format!("`{}` summary", t.id))?,
        ));
        if lessons.is_empty() {
            m.push_str("<p class=\"muted\">No lesson covers this yet.</p>\n");
        } else {
            m.push_str("<ol class=\"lesson-list taxon-lessons\">\n");
            for (i, l) in lessons.iter().enumerate() {
                let extra = format!(
                    " <span class=\"muted\">Chapter {}{}</span>",
                    l.chapter.number,
                    if i == 0 { " · introduced here" } else { "" }
                );
                m.push_str(&lesson_item(l, root, &extra));
            }
            m.push_str("</ol>\n");
        }
        m.push_str("</section>\n");
    }
    Ok(page(
        site,
        Page {
            path: a.path,
            title: a.title,
            description: a.lede,
            kind: a.kind,
            body_attrs: String::new(),
            main: m,
            current: Current {
                path: a.path,
                chapter: None,
                lesson: None,
            },
        },
    ))
}

pub(crate) fn learn_rust(site: &Site) -> Result<String, String> {
    axis_page(
        site,
        Axis {
            path: paths::LEARN_RUST,
            kind: "learn-rust",
            title: "Learn Rust",
            lede: "The Rust concepts this course teaches, in the order it teaches them. Each entry lists the lessons where the concept appears in real Ono-Sendai code.",
            taxa: &site.course.curriculum.concepts,
            anchor: paths::concept_anchor,
            matches: |l, id| l.concepts.iter().any(|c| c == id),
        },
    )
}

pub(crate) fn ono_sendai(site: &Site) -> Result<String, String> {
    axis_page(
        site,
        Axis {
            path: paths::ONO_SENDAI,
            kind: "ono-sendai",
            title: "Understand Ono-Sendai",
            lede: "The same lessons, organised by the part of Ono-Sendai they explain: how the shell is built, and where each piece of it is read in this course.",
            taxa: &site.course.curriculum.ono_topics,
            anchor: paths::topic_anchor,
            matches: |l, id| l.ono_topics.iter().any(|c| c == id),
        },
    )
}

pub(crate) fn glossary(site: &Site) -> Result<String, String> {
    let root = "";
    let mut terms: Vec<_> = site.course.glossary.terms.iter().collect();
    terms.sort_by(|a, b| {
        a.term
            .to_lowercase()
            .cmp(&b.term.to_lowercase())
            .then(a.id.cmp(&b.id))
    });
    let lede = "Short definitions of the terms used in this course. Each entry links to lessons where you can see the term at work in real Ono-Sendai code.";
    let mut m = format!(
        "<header class=\"page-head\">\n<h1>Glossary</h1>\n<p class=\"lede\">{}</p>\n</header>\n",
        escape(lede)
    );
    m.push_str("<nav class=\"toc\" aria-label=\"Glossary index\">\n<ol>\n");
    for t in &terms {
        m.push_str(&format!(
            "<li><a href=\"#{}\">{}</a></li>\n",
            paths::term_anchor(&t.id),
            text_html(&t.term)
        ));
    }
    m.push_str("</ol>\n</nav>\n");
    for t in &terms {
        let anchor = paths::term_anchor(&t.id);
        m.push_str(&format!(
            "<section class=\"term\" id=\"{anchor}\" aria-labelledby=\"{anchor}-h\">\n<h2 id=\"{anchor}-h\">{}</h2>\n<div class=\"term-body\">\n{}</div>\n",
            text_html(&t.term),
            site.md(&t.body, root, &format!("glossary `{}`", t.id))?
        ));
        let examples: Vec<&LessonRef> = t.examples.iter().filter_map(|e| site.lesson(e)).collect();
        if !examples.is_empty() {
            m.push_str("<p class=\"term-examples-label\">See it in Ono-Sendai:</p>\n<ul class=\"lesson-list\">\n");
            for l in examples {
                m.push_str(&lesson_item(
                    l,
                    root,
                    &format!(" <span class=\"muted\">Chapter {}</span>", l.chapter.number),
                ));
            }
            m.push_str("</ul>\n");
        }
        m.push_str("</section>\n");
    }
    Ok(page(
        site,
        Page {
            path: paths::GLOSSARY,
            title: "Glossary",
            description: lede,
            kind: "glossary",
            body_attrs: String::new(),
            main: m,
            current: Current {
                path: paths::GLOSSARY,
                chapter: None,
                lesson: None,
            },
        },
    ))
}

pub(crate) fn about(site: &Site, digest: &str) -> Result<String, String> {
    let lock = &site.course.lock;
    let mut m = String::from("<header class=\"page-head\">\n<h1>About this course build</h1>\n<p class=\"lede\">Which course, which Ono-Sendai, and how this copy was built.</p>\n</header>\n");
    m.push_str("<section aria-labelledby=\"versions-h\">\n<h2 id=\"versions-h\">Versions</h2>\n<dl class=\"facts\">\n");
    let mut fact =
        |k: &str, v: String| m.push_str(&format!("<dt>{}</dt><dd>{v}</dd>\n", escape(k)));
    fact("Course version", escape(&lock.course.version));
    fact(
        "Ono-Sendai repository",
        format!(
            "<code class=\"repo-url\">{}</code>",
            escape(&lock.ono_sendai.repository)
        ),
    );
    fact(
        "Pinned Ono-Sendai commit",
        format!(
            "<code class=\"hash\">{}</code>",
            escape(&lock.ono_sendai.commit)
        ),
    );
    fact(
        "Ono-Sendai version",
        match lock
            .ono_sendai
            .version
            .as_deref()
            .filter(|v| !v.trim().is_empty())
        {
            Some(v) => escape(v),
            None => "not tagged".to_string(),
        },
    );
    fact(
        "Generator",
        format!("ono-course {}", escape(super::GENERATOR_VERSION)),
    );
    fact(
        "Content digest",
        format!("<code class=\"hash\">{}</code>", escape(digest)),
    );
    fact(
        "Contents",
        format!(
            "{} chapters, {} lessons, {} source snippets",
            site.course.chapters.len(),
            site.lessons.len(),
            site.course.snippets.len()
        ),
    );
    m.push_str("</dl>\n<p class=\"muted\">The content digest is a SHA-256 over every course source file (<code>course-lock.yaml</code> and <code>course/**/*.yaml</code>, sorted by path). Two builds with the same digest and generator version are identical.</p>\n</section>\n");

    m.push_str("<section aria-labelledby=\"runtime-h\">\n<h2 id=\"runtime-h\">How this course runs</h2>\n<p class=\"contract\"><strong>Static. Offline. No server. No account. No LLM.</strong></p>\n<p>Every page, stylesheet, script and code snippet ships inside this folder. The course never makes a network request, never loads code from elsewhere and sends nothing anywhere. Opening <code>index.html</code> in a browser is all it needs.</p>\n</section>\n");

    m.push_str("<section aria-labelledby=\"licence-h\">\n<h2 id=\"licence-h\">Licence and attribution</h2>\n<p><strong>Ono-Sendai</strong> is free software under the MIT licence, copyright © 2026 Marcel Arentz. Every code snippet in this course is quoted verbatim from it at the pinned commit above, with its file path and real line numbers, and remains the work of the Ono-Sendai authors; where a snippet is shortened, every omission is marked. The full licence text ships with this course: <a href=\"assets/ONO-SENDAI-LICENSE.txt\">Ono-Sendai licence (MIT)</a>.</p>\n<p><strong>This course</strong> — its explanations, exercises, generator and page design — is a separate project, licensed under the Apache License 2.0. It is not part of Ono-Sendai.</p>\n</section>\n");

    m.push_str("<section aria-labelledby=\"progress-h\">\n<h2 id=\"progress-h\">Your local progress</h2>\n<p>If your browser allows it, the course remembers completed lessons, the last lesson you visited, opened hints, ticked checklist items and your private notes — on this device only. Nothing leaves your device.</p>\n");
    m.push_str("<div class=\"reset\" hidden>\n<button type=\"button\" class=\"btn reset-progress\">Reset local progress</button>\n<div class=\"reset-confirm\" role=\"group\" aria-labelledby=\"reset-q\" hidden>\n<p id=\"reset-q\">Remove everything this course has stored on this device — completed lessons, hints, checklists and notes? This cannot be undone.</p>\n<div class=\"button-row\"><button type=\"button\" class=\"btn btn-danger reset-yes\">Yes, reset my progress</button><button type=\"button\" class=\"btn reset-no\">Cancel</button></div>\n</div>\n<p class=\"reset-status\" role=\"status\"></p>\n</div>\n");
    m.push_str("<noscript><p>JavaScript is disabled, so the course is not storing anything and there is nothing to reset.</p></noscript>\n</section>\n");

    Ok(page(
        site,
        Page {
            path: paths::ABOUT,
            title: "About this course build",
            description: "Course version, pinned Ono-Sendai revision, build information, licence and local progress.",
            kind: "about",
            body_attrs: String::new(),
            main: m,
            current: Current { path: paths::ABOUT, chapter: None, lesson: None },
        },
    ))
}
