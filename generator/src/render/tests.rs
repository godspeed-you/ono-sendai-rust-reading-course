//! Unit tests for the renderer, using the small fixture course in `tests/fixtures/mini`.

use super::code::PageCtx;
use super::exercise::{exercise, ExerciseCtx};
use super::*;
use crate::model::{Exercise, ExerciseType, Hint, Solution, Stage};
use std::path::PathBuf;

fn fixture() -> Course {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mini");
    let (course, diags) = crate::load_and_validate(&root);
    assert!(
        !diags.has_errors(),
        "fixture must be valid:\n{}",
        diags.report()
    );
    course.expect("fixture loads")
}

fn files() -> Files {
    render_files(&fixture()).expect("fixture renders")
}

fn page(files: &Files, path: &str) -> String {
    String::from_utf8(
        files
            .get(path)
            .unwrap_or_else(|| panic!("{path} not generated"))
            .clone(),
    )
    .unwrap()
}

/// All values of `attr="..."` in a page.
fn attr_values(html: &str, attr: &str) -> Vec<String> {
    let pat = format!(" {attr}=\"");
    html.match_indices(&pat)
        .map(|(i, _)| {
            let rest = &html[i + pat.len()..];
            rest[..rest.find('"').unwrap()].to_string()
        })
        .collect()
}

#[test]
fn every_page_has_the_required_skeleton() {
    let f = files();
    for (path, bytes) in &f {
        if !path.ends_with(".html") {
            continue;
        }
        let html = String::from_utf8(bytes.clone()).unwrap();
        assert!(
            html.starts_with("<!doctype html>\n<html lang=\"en\">"),
            "{path}"
        );
        assert_eq!(html.matches("<h1").count(), 1, "{path}: exactly one h1");
        assert!(
            html.contains(
                "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">"
            ),
            "{path}"
        );
        assert!(
            html.contains("assets/course.js\" defer></script>"),
            "{path}"
        );
        assert!(
            html.contains("<a class=\"skip-link\" href=\"#main\">"),
            "{path}"
        );
        assert!(html.contains("<main id=\"main\""), "{path}");
        assert!(
            html.contains("0.9.0") && html.contains("v0.1.0") && html.contains("0123456"),
            "{path}: versions in footer"
        );
        // Heading levels never skip downwards (h1 -> h2 -> h3 -> h4).
        let mut last = 1;
        for (i, _) in html.match_indices("<h") {
            if let Some(d) = html[i + 2..].chars().next().and_then(|c| c.to_digit(10)) {
                assert!(d <= last + 1, "{path}: heading h{d} after h{last}");
                last = d;
            }
        }
    }
}

#[test]
fn links_are_relative_to_the_page_depth() {
    let f = files();
    let index = page(&f, "index.html");
    let lesson = page(&f, "lessons/values-01.html");
    let chapter = page(&f, "chapters/01-values.html");
    assert!(index.contains("href=\"assets/course.css\""));
    assert!(index.contains("href=\"lessons/values-01.html\""));
    assert!(!attr_values(&index, "href")
        .iter()
        .any(|h| h.starts_with("../")));
    assert!(lesson.contains("href=\"../assets/course.css\""));
    assert!(lesson.contains("href=\"../index.html\""));
    assert!(
        lesson.contains("href=\"../learn-rust.html#concept-borrowing\""),
        "markdown links use the page root"
    );
    assert!(chapter.contains("href=\"../lessons/values-01.html\""));
    for html in [&index, &lesson, &chapter] {
        for h in attr_values(html, "href")
            .into_iter()
            .chain(attr_values(html, "src"))
        {
            assert!(
                !h.contains("://") && !h.starts_with('/'),
                "not relative: {h}"
            );
            assert!(
                h.starts_with('#') || h.contains(".html") || h.contains("assets/"),
                "explicit file: {h}"
            );
        }
    }
}

#[test]
fn text_is_escaped_everywhere() {
    let f = files();
    let lesson = page(&f, "lessons/values-01.html");
    assert!(lesson.contains("<h1>The &lt;Value&gt; enum &amp; its methods</h1>"));
    assert!(lesson
        .contains("<title>The &lt;Value&gt; enum &amp; its methods · Mini Reading Course</title>"));
    assert!(
        lesson.contains("Values &amp; variants"),
        "chapter title in breadcrumb and navigator"
    );
    assert!(
        !lesson.contains("<Value>"),
        "raw title text must never reach the page"
    );
    // Source code: `<` and `&&` are escaped inside highlighted lines.
    let l2 = page(&f, "lessons/values-02.html");
    assert!(
        l2.contains("i &gt; <span class=\"tok-num\">0</span> &amp;&amp; i &lt; items"),
        "code is escaped"
    );
    // Attribute context: the nav's data-lesson-id and the description meta.
    let index = page(&f, "index.html");
    assert!(index.contains("Start with lesson 1: The &lt;Value&gt; enum &amp; its methods"));
    assert!(lesson.contains("<meta name=\"description\" content=\"Read an enum declaration and a method that borrows `self`."));
}

#[test]
fn ids_are_unique_even_when_a_snippet_repeats() {
    let f = files();
    for (path, bytes) in &f {
        if path.ends_with(".html") {
            let html = String::from_utf8(bytes.clone()).unwrap();
            let ids = crate::sitecheck::ids(&html);
            let set: std::collections::BTreeSet<_> = ids.iter().collect();
            assert_eq!(set.len(), ids.len(), "{path}: duplicate ids");
        }
    }
    let l2 = page(&f, "lessons/values-02.html");
    assert_eq!(l2.matches("data-snippet=\"value-methods\"").count(), 2);
    assert!(l2.contains("id=\"f1\"") && l2.contains("id=\"f2\"") && l2.contains("id=\"f3\""));
}

#[test]
fn provenance_is_compact_with_details() {
    let f = files();
    let l2 = page(&f, "lessons/values-02.html");
    assert!(l2.contains(
        "<p class=\"provenance\"><span class=\"prov-ono\">Ono-Sendai v0.1.0</span> · <span class=\"prov-file\"><code>crates/mini-shell/src/value.rs</code>:15–36</span> · <span class=\"prov-commit\">commit <code>0123456</code></span> <span class=\"badge badge-shortened\">Shortened</span></p>"
    ));
    assert!(l2.contains("<details class=\"source-details\"><summary>Source details</summary>"));
    assert!(
        l2.contains("0123456789abcdef0123456789abcdef01234567"),
        "full commit in details"
    );
    assert!(l2.contains(
        "Lines 15–17 · anchor <code>impl Value {</code> · <code class=\"hash\">sha256:a8a2c6e2"
    ));
    assert!(l2.contains("12 lines of the original file are not shown (18–29)"));
    let l1 = page(&f, "lessons/values-01.html");
    assert!(
        !l1.contains("badge-shortened"),
        "unshortened snippets carry no marker"
    );
}

#[test]
fn omitted_ranges_are_visible_rows() {
    let l2 = page(&files(), "lessons/values-02.html");
    assert!(l2.contains(
        "<span class=\"line omitted\" data-omitted=\"18-29\"><span class=\"gut\"><span class=\"marks\"></span><span class=\"ln\" data-n=\"⋯\" aria-hidden=\"true\"></span></span><span class=\"lc\">⋯ 12 lines omitted (18–29)</span></span>"
    ));
}

#[test]
fn repeated_snippets_get_unique_region_names() {
    let l2 = page(&files(), "lessons/values-02.html");
    let labels: Vec<&str> = l2
        .split("role=\"region\" aria-label=\"")
        .skip(1)
        .map(|s| s.split('"').next().unwrap())
        .collect();
    assert!(labels.len() >= 2, "values-02 shows a snippet twice");
    let mut unique = labels.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(
        unique.len(),
        labels.len(),
        "duplicate region names: {labels:?}"
    );
    assert!(labels.iter().any(|l| l.ends_with("(view 2)")));
}

#[test]
fn code_lines_carry_real_numbers_highlights_and_markers() {
    let l1 = page(&files(), "lessons/values-01.html");
    assert!(l1.contains("<div class=\"code-scroll\" tabindex=\"0\" role=\"region\" aria-label=\"Source code: crates/mini-shell/src/value.rs, lines 1–13\"><pre class=\"source\" translate=\"no\"><code>"));
    assert!(
        l1.contains("<span class=\"line hl ann\" data-line=\"10\" data-ann=\"3\">"),
        "highlighted line inside annotation 3"
    );
    assert!(
        l1.contains("<span class=\"line\" data-line=\"4\">"),
        "plain line"
    );
    assert!(
        l1.contains("<span class=\"ln\" data-n=\"13\" aria-hidden=\"true\"></span>"),
        "number via data attribute, hidden from AT"
    );
    // Two annotations start on line 8: both markers, and the gutter widens.
    assert!(l1.contains("style=\"--ln-digits:2;--ann-cols:2\""));
    assert!(l1.contains("<span class=\"line ann\" data-line=\"8\" data-ann=\"2 3\">"));
    assert!(l1.contains("<a class=\"ann-marker\" href=\"#f1-ann-2\" data-ann=\"2\" data-n=\"2\" aria-label=\"Annotation 2: Rust, line 8\"></a><a class=\"ann-marker\" href=\"#f1-ann-3\""));
    assert!(l1.contains("<details class=\"annotation ann-kind-ono\" id=\"f1-ann-3\" data-ann=\"3\" data-lines=\"8-13\"><summary><span class=\"ann-num\">3</span><span class=\"ann-meta\"><span class=\"ann-kind\">Ono-Sendai</span> <span class=\"ann-lines\">Lines 8–13</span></span></summary>"));
    // A URL inside a source comment stays inside <pre class="source">.
    assert!(l1.contains("https://example.com/ono/values"));
}

#[test]
fn annotation_tokens_are_marked_robustly() {
    let l1 = page(&files(), "lessons/values-01.html");
    // `enum` is a keyword span: the mark sits inside it.
    assert!(
        l1.contains(
            "<span class=\"tok-kw\"><mark class=\"ann-token\" data-ann=\"2\">enum</mark></span>"
        ),
        "keyword token"
    );
    // `&self` crosses a plain `&` and a keyword span: two well-nested mark pieces.
    assert!(l1.contains(
        "<mark class=\"ann-token\" data-ann=\"1\">&amp;</mark><span class=\"tok-kw\"><mark class=\"ann-token\" data-ann=\"1\">self</mark></span>"
    ));
    assert!(l1.contains("<code class=\"ann-token-label\">&amp;self</code>"));
    // Only the first occurrence on the line is marked (`self` in values-02 line 30).
    let l2 = page(&files(), "lessons/values-02.html");
    let line30 = l2
        .split("data-line=\"30\"")
        .nth(1)
        .unwrap()
        .split("<span class=\"line")
        .next()
        .unwrap();
    assert_eq!(line30.matches("<mark").count(), 1, "{line30}");
}

#[test]
fn multiple_choice_markup() {
    let l1 = page(&files(), "lessons/values-01.html");
    let ex = l1
        .split("id=\"ex-values-01-q1\"")
        .nth(1)
        .unwrap()
        .split("</section>")
        .next()
        .unwrap();
    assert!(l1.contains("<section class=\"exercise\" id=\"ex-values-01-q1\" data-exercise=\"values-01-q1\" data-exercise-type=\"multiple-choice\" data-has-solution=\"true\""));
    assert!(ex.contains("<form class=\"mc\" data-exercise=\"values-01-q1\" novalidate>"));
    assert!(
        ex.contains(
            "<legend>After <code>v.type_name()</code> returns, who owns <code>v</code>?</legend>"
        ),
        "one-paragraph prompt is the legend"
    );
    assert_eq!(
        ex.matches("type=\"radio\" name=\"ex-values-01-q1-answer\"")
            .count(),
        3
    );
    assert_eq!(ex.matches("data-correct=\"true\"").count(), 1);
    assert!(
        ex.contains("<button type=\"submit\" class=\"btn mc-check\" hidden>Check answer</button>")
    );
    assert!(ex.contains("<div class=\"mc-feedback\" role=\"status\" aria-live=\"polite\">"));
    assert!(
        ex.contains("<details class=\"mc-answer\"><summary>Show answer and explanations</summary>"),
        "no-JS fallback"
    );
    assert!(
        ex.contains("Correct answer</span>") && ex.contains("Not correct</span>"),
        "correctness in words"
    );
    assert!(
        !ex.contains("<textarea"),
        "multiple choice has no scratchpad by default"
    );
}

#[test]
fn hints_and_solutions() {
    let f = files();
    let l1 = page(&f, "lessons/values-01.html");
    assert!(l1.contains("<details class=\"hint\" data-level=\"1\"><summary><span class=\"hint-title\">Hint 1</span></summary>"));
    assert!(l1.contains("<details class=\"hint\" data-level=\"2\"><summary><span class=\"hint-title\">Hint 2</span><span class=\"hint-lock\" hidden> — open Hint 1 first</span></summary>"));
    assert!(l1.contains("<details class=\"solution\"><summary>Show worked solution</summary>"));
    assert!(l1.contains("<label for=\"ex-values-01-q2-notes\">Private notes — stay in this browser and are never graded</label>"));
    let practice = page(&f, "lessons/eval-01.html");
    assert!(
        practice.contains("<summary>I have made my attempt — show the worked analysis</summary>")
    );
    for block in [
        "Short answer",
        "What the code explicitly guarantees",
        "What follows from Rust semantics",
        "Architectural interpretation",
    ] {
        assert!(practice.contains(&format!("<h4>{block}</h4>")), "{block}");
    }
    // Analysis items follow the frame order of spec §11, not authoring order.
    let purpose = practice.find("<dt>Purpose</dt>").unwrap();
    let inputs = practice.find("<dt>Inputs</dt>").unwrap();
    let flow = practice.find("<dt>Control flow</dt>").unwrap();
    assert!(purpose < inputs && inputs < flow);
}

#[test]
fn independent_lessons_have_no_solution_markup() {
    let f = files();
    let last = page(&f, "lessons/eval-03.html");
    for forbidden in [
        "class=\"solution",
        "mc-answer",
        "class=\"hint",
        "worked",
        "data-has-solution=\"true\"",
        "data-correct",
    ] {
        assert!(
            !last.contains(forbidden),
            "independent lesson contains `{forbidden}`"
        );
    }
    assert!(last.contains("data-has-solution=\"false\""));
    assert!(last.contains("This exercise deliberately has no solution."));
    assert!(last.contains("<fieldset class=\"checklist\" data-checklist=\"independent-reading\" data-exercise=\"eval-03-q1\">"));
    assert_eq!(last.matches("type=\"checkbox\"").count(), 4);
    assert!(last.contains("You do not need this course anymore. Open Ono-Sendai."));
    assert!(last.contains("git clone https://github.com/godspeed-you/ono-sendai\ncd ono-sendai\ngit checkout 0123456789abcdef"));
    assert!(!last.contains("href=\"https://"), "repository only as text");
    assert!(
        !last.contains("rel=\"next\""),
        "last lesson has no next lesson"
    );
    // Only the final lesson closes the course.
    assert!(!page(&f, "lessons/eval-02.html").contains("course-end"));
}

#[test]
fn stage_policy_is_enforced_even_for_invalid_input() {
    let course = fixture();
    let site = Site::new(&course);
    let ex = Exercise {
        id: "sneaky".into(),
        exercise_type: ExerciseType::ReadFunction,
        prompt: "Read it.".into(),
        snippets: vec![],
        choices: vec![],
        hints: vec![Hint {
            level: 1,
            body: "a hint".into(),
        }],
        solution: Some(Solution::Text("the secret answer".into())),
        checklist: None,
        scratchpad: Some(false),
    };
    let mut pc = PageCtx::default();
    let html = exercise(
        &site,
        &mut pc,
        &ex,
        &ExerciseCtx {
            stage: Stage::Independent,
            number: 1,
            root: "../",
        },
    )
    .unwrap();
    assert!(
        !html.contains("secret") && !html.contains("a hint"),
        "{html}"
    );
    assert!(html.contains("deliberately has no solution"));
    let html = exercise(
        &site,
        &mut pc,
        &ex,
        &ExerciseCtx {
            stage: Stage::Guided,
            number: 1,
            root: "../",
        },
    )
    .unwrap();
    assert!(html.contains("secret") && html.contains("a hint"));
}

#[test]
fn lesson_navigation_and_position() {
    let f = files();
    let l2 = page(&f, "lessons/values-02.html");
    assert!(l2.contains("Chapter 1 · Lesson 2 of 2 <span class=\"kicker-overall\">· Lesson 2 of 5 in the course</span>"));
    assert!(l2.contains("rel=\"prev\" href=\"../lessons/values-01.html\""));
    assert!(l2.contains("rel=\"next\" href=\"../lessons/eval-01.html\"><span class=\"pager-dir\">Next lesson · Chapter 2</span>"), "next works across chapters");
    assert!(
        l2.contains("<span class=\"stage-badge-support\">") || l2.contains("stage-badge-support")
    );
    assert!(l2.contains("Stage 2 of 5 · <strong>Assisted interpretation</strong>"));
    assert!(l2.contains("<li><a href=\"../lessons/values-01.html\">The &lt;Value&gt; enum &amp; its methods</a>"), "prerequisite linked");
    assert!(
        l2.contains("<a href=\"../lessons/values-02.html\" aria-current=\"page\">"),
        "navigator marks the current lesson"
    );
    assert!(l2.contains("<button type=\"button\" class=\"btn mark-complete\" data-lesson=\"values-02\" aria-pressed=\"false\">Mark lesson complete</button>"));
    let first = page(&f, "lessons/values-01.html");
    assert!(!first.contains("rel=\"prev\""));
}

#[test]
fn axis_glossary_and_about_pages() {
    let f = files();
    let lr = page(&f, "learn-rust.html");
    let enums = lr.find("id=\"concept-enums\"").unwrap();
    let errors = lr.find("id=\"concept-error-handling\"").unwrap();
    assert!(enums < errors, "curriculum order");
    let os = page(&f, "ono-sendai.html");
    assert!(os.contains("id=\"topic-evaluation\""));
    let g = page(&f, "glossary.html");
    assert!(
        g.find("id=\"term-borrow\"").unwrap() < g.find("id=\"term-enum\"").unwrap(),
        "alphabetical"
    );
    let about = page(&f, "about.html");
    assert!(about.contains("Static. Offline. No server. No account. No LLM."));
    assert!(about
        .contains("<code class=\"repo-url\">https://github.com/godspeed-you/ono-sendai</code>"));
    assert!(about.contains(&format!("ono-course {GENERATOR_VERSION}")));
    assert!(about.contains("7 source snippets"));
    assert!(about.contains("href=\"assets/ONO-SENDAI-LICENSE.txt\""));
    assert!(about.contains("Apache License 2.0"));
    assert!(about.contains(
        "<button type=\"button\" class=\"btn reset-progress\">Reset local progress</button>"
    ));
    let digest = content_digest(&fixture().root).unwrap();
    assert!(digest.starts_with("sha256:") && digest.len() == 71);
    assert!(about.contains(&digest));
}

#[test]
fn metadata_json_is_complete() {
    let f = files();
    let v: serde_json::Value = serde_json::from_slice(&f["course-metadata.json"]).unwrap();
    assert_eq!(v["course"]["version"], "0.9.0");
    assert_eq!(
        v["ono_sendai"]["commit"],
        "0123456789abcdef0123456789abcdef01234567"
    );
    assert_eq!(v["ono_sendai"]["version"], "v0.1.0");
    assert_eq!(v["generator"]["version"], GENERATOR_VERSION);
    assert_eq!(v["lessons"].as_array().unwrap().len(), 5);
    assert_eq!(v["lessons"][4]["path"], "lessons/eval-03.html");
    assert_eq!(v["lessons"][4]["stage"], "independent");
    let snips = v["snippets"].as_array().unwrap();
    assert_eq!(snips.len(), 7);
    let methods = snips.iter().find(|s| s["id"] == "value-methods").unwrap();
    assert_eq!(methods["lines"], serde_json::json!(["15-17", "30-36"]));
    assert_eq!(methods["content_hashes"].as_array().unwrap().len(), 2);
    let readme = page(&f, "README.txt");
    assert!(
        readme.contains("Open index.html")
            && readme.contains("0123456789abcdef0123456789abcdef01234567")
    );
}

#[test]
fn output_directory_is_only_replaced_when_generated() {
    let course = fixture();
    let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../assets");
    let tmp = tempfile::tempdir().unwrap();
    let foreign = tmp.path().join("precious");
    fs::create_dir_all(&foreign).unwrap();
    fs::write(foreign.join("notes.txt"), "keep me").unwrap();
    let err = render_site(&course, &assets, &foreign).unwrap_err();
    assert!(err.contains("refusing to delete"), "{err}");
    assert!(foreign.join("notes.txt").is_file());

    let out = tmp.path().join("dist");
    render_site(&course, &assets, &out).unwrap();
    fs::write(out.join("stale.html"), "old").unwrap();
    render_site(&course, &assets, &out).unwrap();
    assert!(
        !out.join("stale.html").exists(),
        "a generated site is replaced completely"
    );
    assert!(out.join("assets/favicon.svg").is_file());

    let file = tmp.path().join("file");
    fs::write(&file, "x").unwrap();
    assert!(render_site(&course, &assets, &file).is_err());
}
