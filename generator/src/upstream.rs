//! Validation of snippets against an Ono-Sendai checkout (spec §21, §22).
//!
//! `validate --ono` checks the pinned revision: the checkout must be at the pinned commit, and
//! every snippet must still be found exactly where it was recorded. `check-upstream --ono`
//! compares the same snippets against a newer checkout and reports what an update of the pin
//! would affect. Neither ever rewrites course content.

use crate::load::Course;
use crate::snippet::{self, Location};
use crate::validate::{all_lessons, lesson_snippets};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    Match,
    Moved,
    Changed,
    Ambiguous,
    Missing,
}

impl Status {
    fn failing(self) -> bool {
        !matches!(self, Status::Match | Status::Moved)
    }
}

#[derive(Debug, Clone)]
pub struct SnippetResult {
    pub id: String,
    pub file: String,
    pub status: Status,
    pub detail: String,
    pub lessons: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Report {
    pub checkout_commit: Option<String>,
    pub problems: Vec<String>,
    pub results: Vec<SnippetResult>,
}

impl Report {
    pub fn count(&self, s: Status) -> usize {
        self.results.iter().filter(|r| r.status == s).count()
    }

    pub fn ok(&self) -> bool {
        self.problems.is_empty() && !self.results.iter().any(|r| r.status.failing())
    }

    /// Lesson ids affected by any non-matching snippet.
    pub fn affected_lessons(&self) -> Vec<String> {
        let mut v: Vec<String> = self
            .results
            .iter()
            .filter(|r| r.status != Status::Match)
            .flat_map(|r| r.lessons.clone())
            .collect();
        v.sort();
        v.dedup();
        v
    }

    pub fn render(&self, title: &str, course: &Course, root: &Path) -> String {
        let mut out = String::new();
        let pin = &course.lock.ono_sendai;
        let _ = writeln!(out, "{title}\n");
        let _ = writeln!(out, "checkout: {}", root.display());
        let _ = writeln!(
            out,
            "checkout commit: {}",
            self.checkout_commit
                .as_deref()
                .unwrap_or("unknown (not a git checkout?)")
        );
        let _ = writeln!(
            out,
            "pinned commit:   {}{}\n",
            pin.commit,
            pin.version
                .as_deref()
                .map(|v| format!(" ({v})"))
                .unwrap_or_default()
        );
        for p in &self.problems {
            let _ = writeln!(out, "✗ {p}");
        }
        if !self.problems.is_empty() {
            out.push('\n');
        }
        let _ = writeln!(out, "✓ {:>3} snippets match", self.count(Status::Match));
        let _ = writeln!(
            out,
            "⚠ {:>3} references moved but content still matches",
            self.count(Status::Moved)
        );
        let _ = writeln!(out, "✗ {:>3} snippets changed", self.count(Status::Changed));
        let _ = writeln!(
            out,
            "✗ {:>3} snippets no longer uniquely identifiable",
            self.count(Status::Ambiguous)
        );
        let _ = writeln!(
            out,
            "✗ {:>3} source locations no longer exist",
            self.count(Status::Missing)
        );
        let details: Vec<&SnippetResult> = self
            .results
            .iter()
            .filter(|r| r.status != Status::Match)
            .collect();
        if !details.is_empty() {
            let _ = writeln!(out, "\nDetails:");
            for r in details {
                let mark = if r.status.failing() { "✗" } else { "⚠" };
                let status = match r.status {
                    Status::Match => "match",
                    Status::Moved => "moved",
                    Status::Changed => "changed",
                    Status::Ambiguous => "ambiguous",
                    Status::Missing => "missing",
                };
                let lessons = if r.lessons.is_empty() {
                    "no lessons".to_string()
                } else {
                    r.lessons.join(", ")
                };
                let _ = writeln!(
                    out,
                    "{mark} {status:<9} {:<32} {}  {}",
                    r.id, r.detail, r.file
                );
                let _ = writeln!(out, "    affected lessons: {lessons}");
            }
            let affected = self.affected_lessons();
            if !affected.is_empty() {
                let _ = writeln!(
                    out,
                    "\nAffected lessons ({}): {}",
                    affected.len(),
                    affected.join(", ")
                );
            }
        }
        out
    }
}

/// `git rev-parse` in the checkout, if git and the ref are available.
pub fn git_rev(root: &Path, rev: &str) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--verify", "--quiet"])
        .arg(format!("{rev}^{{commit}}"))
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Lessons using each snippet, in course order.
pub fn snippet_users(course: &Course) -> BTreeMap<String, Vec<String>> {
    let mut m: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (_, _, l) in all_lessons(course) {
        for sid in lesson_snippets(l) {
            let v = m.entry(sid.to_string()).or_default();
            if !v.contains(&l.id) {
                v.push(l.id.clone());
            }
        }
    }
    m
}

/// Compare every snippet with the files of `root`.
pub fn compare(course: &Course, root: &Path) -> Vec<SnippetResult> {
    let users = snippet_users(course);
    let mut results = Vec::new();
    for (id, (_, s)) in &course.snippets {
        let lessons = users.get(id).cloned().unwrap_or_default();
        let path = root.join(&s.source.file);
        let (status, detail) = match std::fs::read_to_string(&path) {
            Err(_) => (Status::Missing, "file missing".to_string()),
            Ok(text) => {
                let mut worst = Status::Match;
                let mut details = Vec::new();
                // Line offset of the previous located segment, to notice omitted lines that
                // changed length between two segments that were both found (spec §20: the gap is
                // shown as "⋯ N lines omitted", and what it hides may have changed).
                let mut prev: Option<(u32, i64)> = None;
                for seg in &s.segments {
                    let location = snippet::locate(seg, &text);
                    let offset = match location {
                        Location::Exact => Some(0),
                        Location::Moved { start_line } => {
                            Some(i64::from(start_line) - i64::from(seg.start_line))
                        }
                        _ => None,
                    };
                    if let (Some((prev_end, prev_off)), Some(off)) = (prev, offset) {
                        if off != prev_off {
                            details.push(format!(
                                "omitted lines {}-{} changed ({:+} lines): review what the gap hides",
                                prev_end + 1,
                                seg.start_line - 1,
                                off - prev_off
                            ));
                        }
                    }
                    prev = offset.map(|o| (seg.end_line, o));
                    let range = format!("{}-{}", seg.start_line, seg.end_line);
                    let (st, d) = match location {
                        Location::Exact => (Status::Match, format!("{range} ok")),
                        Location::Moved { start_line } => (
                            Status::Moved,
                            format!(
                                "{range} → {start_line}-{}",
                                start_line + (seg.end_line - seg.start_line)
                            ),
                        ),
                        Location::Ambiguous { occurrences } => (
                            Status::Ambiguous,
                            format!("{range} text found {occurrences} times"),
                        ),
                        Location::Changed => (Status::Changed, format!("{range} content changed")),
                        Location::OutOfRange { file_lines } => (
                            Status::Missing,
                            format!(
                                "{range} beyond end of file ({file_lines} lines), text not found"
                            ),
                        ),
                    };
                    worst = worst.max(st);
                    details.push(d);
                }
                (worst, details.join("; "))
            }
        };
        results.push(SnippetResult {
            id: id.clone(),
            file: s.source.file.clone(),
            status,
            detail,
            lessons,
        });
    }
    results
}

/// Validate the course against a checkout that must be at the pinned commit.
pub fn validate_pinned(course: &Course, root: &Path) -> Report {
    let pin = &course.lock.ono_sendai;
    let head = git_rev(root, "HEAD");
    let mut problems = Vec::new();
    match &head {
        Some(h) if h != &pin.commit => problems.push(format!(
            "pinned commit mismatch: checkout is at {h}, course-lock.yaml pins {}",
            pin.commit
        )),
        None => problems.push(
            "cannot determine the checkout's commit (is it a git checkout and is git installed?)"
                .to_string(),
        ),
        _ => {}
    }
    if let Some(v) = &pin.version {
        if let Some(tagged) = git_rev(root, v) {
            if tagged != pin.commit {
                problems.push(format!(
                    "version `{v}` resolves to {tagged} in the checkout, not the pinned commit {}",
                    pin.commit
                ));
            }
        }
    }
    Report {
        checkout_commit: head,
        problems,
        results: compare(course, root),
    }
}

/// Compare against a newer checkout, for an explicit pin update (spec §22).
pub fn check_newer(course: &Course, root: &Path) -> Report {
    Report {
        checkout_commit: git_rev(root, "HEAD"),
        problems: Vec::new(),
        results: compare(course, root),
    }
}
