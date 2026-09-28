//! Helpers shared by the validation and upstream integration tests: a writable copy of the
//! validation fixture course, and a hermetic temporary git checkout of its fake Ono-Sendai tree.

#![allow(dead_code)]

use ono_course::diag::Diagnostics;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// The commit placeholder the checked-in fixture pins; replaced by a real SHA in git tests.
pub const PLACEHOLDER_COMMIT: &str = "a1b2c3d4e5f60718293a4b5c6d7e8f9012345678";

pub const LIB_RS: &str = "crates/ono-demo/src/lib.rs";
pub const EVAL_RS: &str = "crates/ono-demo/src/eval.rs";

/// The checked-in fixture course root (read-only; tests never write into it).
pub fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/validation")
}

/// The fake Ono-Sendai source tree the fixture snippets were taken from.
pub fn ono_source() -> PathBuf {
    fixture_root().join("ono")
}

pub fn copy_dir(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let to = dst.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &to);
        } else {
            fs::copy(entry.path(), &to).unwrap();
        }
    }
}

/// A private, writable copy of the fixture course (without the `ono/` tree).
pub struct Fixture {
    dir: TempDir,
}

impl Fixture {
    pub fn new() -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let src = fixture_root();
        fs::copy(
            src.join("course-lock.yaml"),
            dir.path().join("course-lock.yaml"),
        )
        .unwrap();
        copy_dir(&src.join("course"), &dir.path().join("course"));
        Fixture { dir }
    }

    pub fn root(&self) -> &Path {
        self.dir.path()
    }

    pub fn path(&self, rel: &str) -> PathBuf {
        self.root().join(rel)
    }

    pub fn read(&self, rel: &str) -> String {
        fs::read_to_string(self.path(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
    }

    pub fn write(&self, rel: &str, text: &str) {
        let p = self.path(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
    }

    pub fn remove(&self, rel: &str) {
        fs::remove_file(self.path(rel)).unwrap();
    }

    /// Replace the first occurrence of `from` in `rel`; panics if `from` is absent so a fixture
    /// change can never turn a mutation test into a silent no-op.
    pub fn replace(&self, rel: &str, from: &str, to: &str) {
        let text = self.read(rel);
        assert!(
            text.contains(from),
            "mutation target `{from}` not found in {rel}"
        );
        self.write(rel, &text.replacen(from, to, 1));
    }

    /// Replace every occurrence of `from` in `rel` (must occur at least once).
    pub fn replace_all(&self, rel: &str, from: &str, to: &str) {
        let text = self.read(rel);
        assert!(
            text.contains(from),
            "mutation target `{from}` not found in {rel}"
        );
        self.write(rel, &text.replace(from, to));
    }

    pub fn validate(&self) -> Diagnostics {
        ono_course::load_and_validate(self.root()).1
    }

    /// Point the lock file and every snippet at `commit` instead of the placeholder.
    pub fn pin(&self, commit: &str) {
        self.replace_all("course-lock.yaml", PLACEHOLDER_COMMIT, commit);
        for p in ono_course::load::yaml_files(&self.path("course/snippets")) {
            let text = fs::read_to_string(&p).unwrap();
            fs::write(&p, text.replace(PLACEHOLDER_COMMIT, commit)).unwrap();
        }
    }
}

/// Whether a usable `git` binary is on PATH.
pub fn has_git() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

/// `false` (after printing a clear skip message) when git is not installed; tests needing a
/// checkout do `if !support::git_available() { return; }`.
pub fn git_available() -> bool {
    let ok = has_git();
    if !ok {
        eprintln!("SKIPPED: `git` is not installed; this test needs a temporary git checkout");
    }
    ok
}

/// Run git in `dir` with a hermetic, fixed identity and date; returns trimmed stdout.
pub fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .env("GIT_AUTHOR_DATE", "2024-01-01T00:00:00Z")
        .env("GIT_COMMITTER_DATE", "2024-01-01T00:00:00Z")
        .output()
        .expect("run git");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// A temporary git checkout of the fake Ono-Sendai tree.
pub struct Checkout {
    dir: TempDir,
}

impl Checkout {
    /// `git init` a copy of `ono/` and commit it; returns the checkout and the commit SHA.
    pub fn new() -> (Checkout, String) {
        let dir = tempfile::tempdir().unwrap();
        copy_dir(&ono_source(), dir.path());
        git(dir.path(), &["init", "-q"]);
        let c = Checkout { dir };
        let sha = c.commit("initial");
        (c, sha)
    }

    pub fn root(&self) -> &Path {
        self.dir.path()
    }

    pub fn read(&self, rel: &str) -> String {
        fs::read_to_string(self.root().join(rel)).unwrap()
    }

    pub fn write(&self, rel: &str, text: &str) {
        fs::write(self.root().join(rel), text).unwrap();
    }

    pub fn remove(&self, rel: &str) {
        fs::remove_file(self.root().join(rel)).unwrap();
    }

    /// Commit everything (allowing an empty commit) and return the new HEAD.
    pub fn commit(&self, msg: &str) -> String {
        git(self.root(), &["add", "-A"]);
        git(
            self.root(),
            &["commit", "-q", "--allow-empty", "--no-gpg-sign", "-m", msg],
        );
        git(self.root(), &["rev-parse", "HEAD"])
    }
}

/// A fixture course pinned to a fresh checkout of the fake Ono-Sendai tree.
pub fn pinned_pair() -> (Fixture, Checkout, String) {
    let (checkout, sha) = Checkout::new();
    let fixture = Fixture::new();
    fixture.pin(&sha);
    (fixture, checkout, sha)
}
