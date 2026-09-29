//! Integration tests for validation against an Ono-Sendai checkout (spec §21, §22) and for the
//! `ono-course` command line, using a temporary git checkout of the fixture's fake Ono tree.

mod support;

use ono_course::load::Course;
use ono_course::model::Snippet;
use ono_course::snippet::{self, Location};
use ono_course::upstream::{self, Report, SnippetResult, Status};
use std::path::Path;
use std::process::{Command, Output};
use support::{git, Fixture, EVAL_RS, LIB_RS};

fn load(f: &Fixture) -> Course {
    let (course, diags) = ono_course::load_and_validate(f.root());
    assert!(!diags.has_errors(), "{}", diags.report());
    course.unwrap()
}

fn result<'a>(r: &'a Report, id: &str) -> &'a SnippetResult {
    r.results
        .iter()
        .find(|x| x.id == id)
        .unwrap_or_else(|| panic!("no result for `{id}`"))
}

fn statuses(r: &Report) -> Vec<(&str, Status)> {
    r.results
        .iter()
        .map(|x| (x.id.as_str(), x.status))
        .collect()
}

// ---------------------------------------------------------------------------------------------
// validate_pinned

#[test]
fn all_snippets_match_at_the_pinned_commit() {
    if !support::git_available() {
        return;
    }
    let (f, c, sha) = support::pinned_pair();
    // The informational version tag resolves to the pinned commit: no problem either.
    git(c.root(), &["tag", "v0.1.0"]);
    let course = load(&f);
    let r = upstream::validate_pinned(&course, c.root());
    assert!(r.ok(), "{}", r.render("t", &course, c.root()));
    assert!(r.problems.is_empty(), "{:?}", r.problems);
    assert_eq!(r.checkout_commit.as_deref(), Some(sha.as_str()));
    assert_eq!(r.count(Status::Match), 5);
    assert!(r.affected_lessons().is_empty());
    let text = r.render("Ono-Sendai source validation", &course, c.root());
    assert!(text.contains("✓   5 snippets match"), "{text}");
    assert!(text.contains(&format!("checkout commit: {sha}")), "{text}");
    assert!(!text.contains("Details:"), "{text}");
}

#[test]
fn pinned_commit_mismatch_is_reported() {
    if !support::git_available() {
        return;
    }
    let (f, c, sha) = support::pinned_pair();
    let head = c.commit("a later commit");
    let course = load(&f);
    let r = upstream::validate_pinned(&course, c.root());
    assert!(!r.ok());
    assert!(
        r.problems
            .iter()
            .any(|p| p.contains("pinned commit mismatch") && p.contains(&head) && p.contains(&sha)),
        "{:?}",
        r.problems
    );
    // The files are unchanged, so every snippet still matches.
    assert_eq!(r.count(Status::Match), 5);
    assert!(r
        .render("t", &course, c.root())
        .contains("✗ pinned commit mismatch"));
}

#[test]
fn version_tag_resolving_elsewhere_is_reported() {
    if !support::git_available() {
        return;
    }
    let (f, c, sha) = support::pinned_pair();
    let other = c.commit("tagged release");
    git(c.root(), &["tag", "v0.1.0"]);
    git(c.root(), &["checkout", "-q", "--detach", &sha]);
    let course = load(&f);
    let r = upstream::validate_pinned(&course, c.root());
    assert!(!r.ok());
    assert!(
        r.problems
            .iter()
            .any(|p| p.contains("version `v0.1.0` resolves to")
                && p.contains(&other)
                && p.contains(&sha)),
        "{:?}",
        r.problems
    );
    assert!(
        !r.problems
            .iter()
            .any(|p| p.contains("pinned commit mismatch")),
        "{:?}",
        r.problems
    );
}

#[test]
fn directory_that_is_not_a_git_checkout_is_a_problem() {
    let f = Fixture::new();
    let course = load(&f);
    let dir = tempfile::tempdir().unwrap();
    support::copy_dir(&support::ono_source(), dir.path());
    let r = upstream::validate_pinned(&course, dir.path());
    assert!(!r.ok());
    assert!(
        r.problems
            .iter()
            .any(|p| p.contains("cannot determine the checkout's commit")),
        "{:?}",
        r.problems
    );
    // Snippet comparison still works on the plain files.
    assert_eq!(r.count(Status::Match), 5);
}

// ---------------------------------------------------------------------------------------------
// check_newer

#[test]
fn missing_file_is_reported_with_affected_lessons() {
    if !support::git_available() {
        return;
    }
    let (f, c, _) = support::pinned_pair();
    c.remove(EVAL_RS);
    c.commit("remove eval.rs");
    let course = load(&f);
    let r = upstream::check_newer(&course, c.root());
    assert!(!r.ok());
    assert_eq!(result(&r, "parse-word").status, Status::Missing);
    assert_eq!(result(&r, "parse-word").lessons, ["deeper-02"]);
    assert_eq!(result(&r, "eval-word").status, Status::Missing);
    assert_eq!(result(&r, "eval-word").lessons, ["deeper-03"]);
    assert!(result(&r, "eval-word").detail.contains("file missing"));
    assert_eq!(r.count(Status::Match), 3);
    assert_eq!(r.affected_lessons(), ["deeper-02", "deeper-03"]);
}

#[test]
fn changed_content_is_reported_with_affected_lessons() {
    if !support::git_available() {
        return;
    }
    let (f, c, _) = support::pinned_pair();
    c.write(
        LIB_RS,
        &c.read(LIB_RS).replace("hello, {name}", "hi, {name}"),
    );
    c.commit("change greeting");
    let course = load(&f);
    let r = upstream::check_newer(&course, c.root());
    assert!(!r.ok());
    let greet = result(&r, "greet");
    assert_eq!(greet.status, Status::Changed);
    assert_eq!(
        greet.lessons,
        ["basics-01"],
        "used twice in basics-01, reported once"
    );
    assert!(
        greet.detail.contains("3-6 content changed"),
        "{}",
        greet.detail
    );
    assert_eq!(r.count(Status::Changed), 1);
    assert_eq!(r.count(Status::Match), 4);
    assert_eq!(r.affected_lessons(), ["basics-01"]);
}

#[test]
fn moved_content_is_reported_but_not_failing() {
    if !support::git_available() {
        return;
    }
    let (f, c, _) = support::pinned_pair();
    c.write(
        LIB_RS,
        &format!(
            "// SPDX-License-Identifier: MIT\n// Copyright\n{}",
            c.read(LIB_RS)
        ),
    );
    c.commit("add header");
    let course = load(&f);
    let r = upstream::check_newer(&course, c.root());
    assert!(r.ok(), "moved snippets must not fail: {:?}", statuses(&r));
    assert_eq!(r.count(Status::Moved), 3);
    assert_eq!(r.count(Status::Match), 2);
    assert!(
        result(&r, "greet").detail.contains("3-6 → 5-8"),
        "{}",
        result(&r, "greet").detail
    );
    assert!(result(&r, "sum-until-negative")
        .detail
        .contains("13-23 → 15-25"));
    let seg = &course.snippets["greet"].1.segments[0];
    assert_eq!(
        snippet::locate(seg, &c.read(LIB_RS)),
        Location::Moved { start_line: 5 }
    );
    // Moved snippets are still listed as affected so their line numbers get refreshed.
    assert_eq!(
        r.affected_lessons(),
        ["basics-01", "basics-02", "deeper-01"]
    );
}

#[test]
fn change_hidden_in_the_omitted_lines_of_a_shortened_snippet_is_flagged() {
    if !support::git_available() {
        return;
    }
    let (f, c, _) = support::pinned_pair();
    // `parse-word` shows eval.rs 3-6 and 8-14; line 7 (blank) is omitted. Code inserted there
    // leaves both shown segments intact, but what the "⋯ omitted" row hides has changed.
    c.write(
        EVAL_RS,
        &c.read(EVAL_RS).replacen(
            "}\n\n/// Parses",
            "}\n\nconst HIDDEN: i64 = 1;\n\n/// Parses",
            1,
        ),
    );
    c.commit("insert code between the segments");
    let course = load(&f);
    let r = upstream::check_newer(&course, c.root());
    let pw = result(&r, "parse-word");
    assert_eq!(pw.status, Status::Moved, "{}", pw.detail);
    assert!(
        pw.detail
            .contains("omitted lines 7-7 changed (+2 lines): review what the gap hides"),
        "{}",
        pw.detail
    );
    // A snippet moved as a whole (every segment by the same offset) gets no gap note.
    c.write(EVAL_RS, &format!("// header\n{}", c.read(EVAL_RS)));
    c.commit("shift everything");
    let r = upstream::check_newer(&course, c.root());
    assert!(
        result(&r, "eval-word").detail.contains("→"),
        "{}",
        result(&r, "eval-word").detail
    );
    assert!(!result(&r, "eval-word").detail.contains("omitted"));
}

#[test]
fn duplicated_content_is_ambiguous() {
    if !support::git_available() {
        return;
    }
    let (f, c, _) = support::pinned_pair();
    let lib = c.read(LIB_RS);
    let greet = snippet::extract(&lib, 3, 6).unwrap();
    // Shift the original and add an identical copy: the text no longer identifies one place.
    c.write(LIB_RS, &format!("// header\n{lib}\n{greet}"));
    c.commit("duplicate greet");
    let course = load(&f);
    let r = upstream::check_newer(&course, c.root());
    assert!(!r.ok());
    let g = result(&r, "greet");
    assert_eq!(g.status, Status::Ambiguous);
    assert!(g.detail.contains("text found 2 times"), "{}", g.detail);
    assert_eq!(result(&r, "count-words").status, Status::Moved);
}

#[test]
fn range_beyond_end_of_file_is_missing() {
    if !support::git_available() {
        return;
    }
    let (f, c, _) = support::pinned_pair();
    let lib = c.read(LIB_RS);
    let first12: String = lib.lines().take(12).map(|l| format!("{l}\n")).collect();
    c.write(LIB_RS, &first12);
    c.commit("truncate");
    let course = load(&f);
    let r = upstream::check_newer(&course, c.root());
    assert!(!r.ok());
    let s = result(&r, "sum-until-negative");
    assert_eq!(s.status, Status::Missing);
    assert!(
        s.detail.contains("13-23 beyond end of file (12 lines)"),
        "{}",
        s.detail
    );
    assert_eq!(s.lessons, ["deeper-01"]);
    assert_eq!(result(&r, "greet").status, Status::Match);
}

#[test]
fn report_lists_every_category_and_the_affected_lessons() {
    if !support::git_available() {
        return;
    }
    let (f, c, _) = support::pinned_pair();
    let lib = format!("// header\n{}", c.read(LIB_RS)).replace("total += v;", "total += *v;");
    c.write(LIB_RS, &lib);
    c.remove(EVAL_RS);
    let head = c.commit("big refactor");
    let course = load(&f);
    let r = upstream::check_newer(&course, c.root());
    assert!(!r.ok());
    let text = r.render("Ono-Sendai upstream comparison", &course, c.root());
    for line in [
        "Ono-Sendai upstream comparison",
        &format!("checkout commit: {head}"),
        "✓   0 snippets match",
        "⚠   2 references moved but content still matches",
        "✗   1 snippets changed",
        "✗   0 snippets no longer uniquely identifiable",
        "✗   2 source locations no longer exist",
        "Details:",
        "affected lessons: deeper-01",
        "affected lessons: deeper-02",
        "Affected lessons (5): basics-01, basics-02, deeper-01, deeper-02, deeper-03",
    ] {
        assert!(text.contains(line), "missing `{line}` in:\n{text}");
    }
    assert!(text.contains("⚠ moved"), "{text}");
    assert!(text.contains("✗ changed"), "{text}");
    assert!(text.contains("✗ missing"), "{text}");
}

// ---------------------------------------------------------------------------------------------
// Command line

fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ono-course"))
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .expect("run ono-course")
}

fn p(path: &Path) -> &str {
    path.to_str().unwrap()
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

#[track_caller]
fn assert_exit(o: &Output, code: i32) {
    assert_eq!(
        o.status.code(),
        Some(code),
        "stdout:\n{}\nstderr:\n{}",
        stdout(o),
        stderr(o)
    );
}

#[test]
fn cli_validate_accepts_the_fixture() {
    let o = cli(&["validate", "--root", p(&support::fixture_root())]);
    assert_exit(&o, 0);
    assert!(
        stderr(&o).contains("course source valid: 2 chapters, 5 lessons, 5 snippets, 0 warning(s)"),
        "{}",
        stderr(&o)
    );
}

#[test]
fn cli_validate_rejects_a_broken_course() {
    let f = Fixture::new();
    f.replace(
        "course/chapters/01-basics.yaml",
        "          - level: 2\n",
        "          - level: 3\n",
    );
    let o = cli(&["validate", "--root", p(f.root())]);
    assert_exit(&o, 1);
    let err = stderr(&o);
    assert!(
        err.contains(
            "error: course/chapters/01-basics.yaml: exercise `basics-01-q1`: invalid hint ordering"
        ),
        "{err}"
    );
    assert!(err.contains("course source invalid: 1 error(s)"), "{err}");
}

#[test]
fn cli_usage_errors_exit_2() {
    assert_exit(&cli(&[]), 2);
    assert_exit(&cli(&["frobnicate"]), 2);
    assert_exit(
        &cli(&["validate", "--root", p(&support::fixture_root()), "extra"]),
        2,
    );
    assert_exit(
        &cli(&["check-upstream", "--root", p(&support::fixture_root())]),
        2,
    );
}

#[test]
fn cli_validate_with_ono_checkout() {
    if !support::git_available() {
        return;
    }
    let (f, c, _) = support::pinned_pair();
    let o = cli(&["validate", "--root", p(f.root()), "--ono", p(c.root())]);
    assert_exit(&o, 0);
    assert!(
        stdout(&o).contains("✓   5 snippets match"),
        "{}",
        stdout(&o)
    );

    c.write(LIB_RS, &c.read(LIB_RS).replace("hello", "hey"));
    let o = cli(&["validate", "--root", p(f.root()), "--ono", p(c.root())]);
    assert_exit(&o, 1);
    assert!(
        stdout(&o).contains("✗   1 snippets changed"),
        "{}",
        stdout(&o)
    );
    assert!(
        stdout(&o).contains("affected lessons: basics-01"),
        "{}",
        stdout(&o)
    );
}

#[test]
fn cli_snippet_add_writes_an_exact_snippet() {
    if !support::git_available() {
        return;
    }
    let (f, c, sha) = support::pinned_pair();
    let o = cli(&[
        "snippet",
        "add",
        "--root",
        p(f.root()),
        "--ono",
        p(c.root()),
        "--id",
        "parse-again",
        "--file",
        EVAL_RS,
        "--lines",
        "3-6",
        "--anchor",
        "pub enum Word",
        "--lines",
        "8-14",
        "--anchor",
        "pub fn parse",
    ]);
    assert_exit(&o, 0);
    assert!(
        stdout(&o).contains("(2 segment(s), 11 lines)"),
        "{}",
        stdout(&o)
    );
    let written: Snippet =
        serde_yaml::from_str(&f.read("course/snippets/ono-demo/parse-again.yaml")).unwrap();
    assert_eq!(written.id, "parse-again");
    assert_eq!(written.source.file, EVAL_RS);
    assert_eq!(written.source.commit, sha);
    // Identical to the checked-in fixture snippet for the same ranges.
    let fixture: Snippet =
        serde_yaml::from_str(&f.read("course/snippets/ono-demo/parse-word.yaml")).unwrap();
    assert_eq!(written.segments, fixture.segments);
    for seg in &written.segments {
        assert_eq!(snippet::content_hash(&seg.code), seg.content_hash);
    }
    // The course is still valid; the new snippet is merely unused.
    let o = cli(&["validate", "--root", p(f.root())]);
    assert_exit(&o, 0);
    assert!(
        stderr(&o).contains("snippet `parse-again` is not used by any lesson"),
        "{}",
        stderr(&o)
    );
}

#[test]
fn cli_snippet_add_refuses_a_checkout_at_another_commit() {
    if !support::git_available() {
        return;
    }
    let (f, c, sha) = support::pinned_pair();
    let head = c.commit("newer");
    let o = cli(&[
        "snippet",
        "add",
        "--root",
        p(f.root()),
        "--ono",
        p(c.root()),
        "--id",
        "crate-doc",
        "--file",
        LIB_RS,
        "--lines",
        "1",
        "--anchor",
        "demo crate",
    ]);
    assert_exit(&o, 2);
    assert!(
        stderr(&o).contains(&format!(
            "checkout is at {head}, but course-lock.yaml pins {sha}"
        )),
        "{}",
        stderr(&o)
    );
    assert!(!f.path("course/snippets/ono-demo/crate-doc.yaml").exists());
}

#[test]
fn cli_snippet_add_refuses_to_overwrite_without_force() {
    if !support::git_available() {
        return;
    }
    let (f, c, _) = support::pinned_pair();
    let add = |id: &str, anchor: &str, force: bool| {
        let mut args = vec![
            "snippet",
            "add",
            "--root",
            p(f.root()),
            "--ono",
            p(c.root()),
            "--id",
            id,
            "--file",
            LIB_RS,
            "--lines",
            "1-4",
            "--anchor",
            anchor,
        ];
        if force {
            args.push("--force");
        }
        cli(&args)
    };
    assert_exit(&add("crate-doc", "demo crate", false), 0);
    let before = f.read("course/snippets/ono-demo/crate-doc.yaml");
    let o = add("crate-doc", "pub fn greet", false);
    assert_exit(&o, 2);
    assert!(
        stderr(&o).contains("already exists (use --force to replace)"),
        "{}",
        stderr(&o)
    );
    assert_eq!(f.read("course/snippets/ono-demo/crate-doc.yaml"), before);
    // An existing course snippet is protected too.
    let greet = f.read("course/snippets/ono-demo/greet.yaml");
    assert_exit(&add("greet", "demo crate", false), 2);
    assert_eq!(f.read("course/snippets/ono-demo/greet.yaml"), greet);
    // --force replaces.
    assert_exit(&add("crate-doc", "pub fn greet", true), 0);
    let after: Snippet =
        serde_yaml::from_str(&f.read("course/snippets/ono-demo/crate-doc.yaml")).unwrap();
    assert_eq!(after.segments[0].anchor, "pub fn greet");
}

#[test]
fn cli_snippet_add_rejects_bad_anchor_and_range() {
    if !support::git_available() {
        return;
    }
    let (f, c, _) = support::pinned_pair();
    let run = |lines: &str, anchor: &str| {
        cli(&[
            "snippet",
            "add",
            "--root",
            p(f.root()),
            "--ono",
            p(c.root()),
            "--id",
            "bad",
            "--file",
            LIB_RS,
            "--lines",
            lines,
            "--anchor",
            anchor,
        ])
    };
    let o = run("13-23", "total");
    assert_exit(&o, 2);
    assert!(
        stderr(&o).contains("must occur exactly once"),
        "{}",
        stderr(&o)
    );
    let o = run("20-40", "total");
    assert_exit(&o, 2);
    assert!(
        stderr(&o).contains("exceeds the file's 23 lines"),
        "{}",
        stderr(&o)
    );
    assert!(!f.path("course/snippets/ono-demo/bad.yaml").exists());
}

#[test]
fn cli_check_upstream_exit_codes() {
    if !support::git_available() {
        return;
    }
    let (f, c, _) = support::pinned_pair();
    let run = || {
        cli(&[
            "check-upstream",
            "--root",
            p(f.root()),
            "--ono",
            p(c.root()),
        ])
    };

    let o = run();
    assert_exit(&o, 0);
    assert!(
        stdout(&o).contains("✓   5 snippets match"),
        "{}",
        stdout(&o)
    );
    assert!(
        stdout(&o).contains("never updates the pin"),
        "{}",
        stdout(&o)
    );

    // Moved only: still success, but reported.
    c.write(LIB_RS, &format!("// header\n{}", c.read(LIB_RS)));
    c.commit("header");
    let o = run();
    assert_exit(&o, 0);
    assert!(
        stdout(&o).contains("⚠   3 references moved but content still matches"),
        "{}",
        stdout(&o)
    );

    // Changed: failure.
    c.write(
        EVAL_RS,
        &c.read(EVAL_RS).replace("Some(*n)", "Some(n.abs())"),
    );
    c.commit("change eval");
    let o = run();
    assert_exit(&o, 1);
    assert!(
        stdout(&o).contains("✗   1 snippets changed"),
        "{}",
        stdout(&o)
    );
    assert!(
        stdout(&o).contains("affected lessons: deeper-03"),
        "{}",
        stdout(&o)
    );

    // The pin was never touched.
    assert!(!f
        .read("course-lock.yaml")
        .contains(&upstream::git_rev(c.root(), "HEAD").unwrap()));
}

#[test]
fn cli_check_upstream_refuses_an_invalid_course() {
    if !support::git_available() {
        return;
    }
    let (f, c, _) = support::pinned_pair();
    f.replace("course/chapters/02-deeper.yaml", "number: 2", "number: 5");
    let o = cli(&[
        "check-upstream",
        "--root",
        p(f.root()),
        "--ono",
        p(c.root()),
    ]);
    assert_exit(&o, 1);
    assert!(
        stderr(&o).contains("has number 5 but is listed at position 2"),
        "{}",
        stderr(&o)
    );
}

#[test]
fn checkout_sha_is_written_into_the_temp_course() {
    if !support::git_available() {
        return;
    }
    let (f, _c, sha) = support::pinned_pair();
    let course = load(&f);
    assert_eq!(course.lock.ono_sendai.commit, sha);
    assert!(course
        .snippets
        .values()
        .all(|(_, s)| s.source.commit == sha));
    // The checked-in fixture itself is untouched.
    let original =
        std::fs::read_to_string(support::fixture_root().join("course-lock.yaml")).unwrap();
    assert!(original.contains(support::PLACEHOLDER_COMMIT));
}
