//! End-to-end rendering of the `mini` fixture course: the generated site must pass the offline
//! and link checks, be byte-for-byte deterministic, and match the golden snapshots of a few
//! representative pages (spec §74).
//!
//! Golden files live in `tests/golden/`. After an intended template change, regenerate them with
//!
//! ```text
//! UPDATE_GOLDEN=1 cargo test -p ono-course --test render
//! ```
//!
//! and review the diff like any other change.

use ono_course::{load_and_validate, render, sitecheck};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Pages snapshotted as golden files: an annotated guided lesson with a multiple-choice exercise,
/// a practice lesson with a structured solution, the final independent lesson, and a chapter page.
const GOLDEN: &[(&str, &str)] = &[
    ("lessons/values-01.html", "lesson-guided.html"),
    ("lessons/eval-01.html", "lesson-practice.html"),
    ("lessons/eval-03.html", "lesson-independent.html"),
    ("chapters/01-values.html", "chapter.html"),
];

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn build(out: &Path) -> ono_course::load::Course {
    let root = manifest().join("tests/fixtures/mini");
    let (course, diags) = load_and_validate(&root);
    assert!(
        !diags.has_errors(),
        "fixture course must be valid:\n{}",
        diags.report()
    );
    let course = course.unwrap();
    render::render_site(&course, &manifest().join("../assets"), out).expect("render");
    course
}

fn snapshot(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else {
                let rel = p
                    .strip_prefix(dir)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.insert(rel, fs::read(&p).unwrap());
            }
        }
    }
    out
}

#[test]
fn fixture_site_passes_the_site_check() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("dist");
    let course = build(&out);
    let diags = sitecheck::check_site(
        &out,
        &sitecheck::SiteCheckOptions {
            allowed_text_url: &course.lock.ono_sendai.repository,
        },
    );
    assert!(!diags.has_errors(), "{}", diags.report());
    for f in [
        "README.txt",
        "lessons/values-01.html",
        "chapters/02-evaluation.html",
        "assets/ONO-SENDAI-LICENSE.txt",
    ] {
        assert!(out.join(f).is_file(), "{f} missing");
    }
}

#[test]
fn output_is_deterministic() {
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    build(&a.path().join("dist"));
    build(&b.path().join("dist"));
    let (sa, sb) = (
        snapshot(&a.path().join("dist")),
        snapshot(&b.path().join("dist")),
    );
    assert_eq!(sa.keys().collect::<Vec<_>>(), sb.keys().collect::<Vec<_>>());
    for (k, v) in &sa {
        assert!(v == &sb[k], "{k} differs between two builds");
    }
}

#[test]
fn golden_pages() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("dist");
    build(&out);
    let golden_dir = manifest().join("tests/golden");
    let update = std::env::var_os("UPDATE_GOLDEN").is_some_and(|v| v == "1");
    let mut mismatches = Vec::new();
    for (page, golden) in GOLDEN {
        let actual = fs::read_to_string(out.join(page)).unwrap();
        let path = golden_dir.join(golden);
        if update {
            fs::create_dir_all(&golden_dir).unwrap();
            fs::write(&path, &actual).unwrap();
            continue;
        }
        let expected = fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("{} missing; run with UPDATE_GOLDEN=1", path.display()));
        if expected != actual {
            let line = expected
                .lines()
                .zip(actual.lines())
                .position(|(e, a)| e != a)
                .map_or_else(
                    || expected.lines().count().min(actual.lines().count()) + 1,
                    |i| i + 1,
                );
            mismatches.push(format!(
                "{page} differs from tests/golden/{golden} (first difference at line {line}):\n  expected: {}\n  actual:   {}",
                expected.lines().nth(line - 1).unwrap_or("<end of file>"),
                actual.lines().nth(line - 1).unwrap_or("<end of file>"),
            ));
        }
    }
    assert!(
        mismatches.is_empty(),
        "{}\n\nIf the change is intended, run `UPDATE_GOLDEN=1 cargo test -p ono-course --test render` and review the diff.",
        mismatches.join("\n\n")
    );
}
