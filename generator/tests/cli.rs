//! End-to-end tests of `build`, `check-offline` and `package` through the command line, on the
//! `mini` fixture course: the happy path produces a checked, packaged site, and each failure
//! condition of spec §64 that only shows up at build time (an illegal external runtime resource,
//! a missing required asset, a stale site) stops the pipeline with a useful message.

mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn mini() -> PathBuf {
    manifest().join("tests/fixtures/mini")
}

/// A private copy of the shipped runtime assets, so tests can break them.
fn assets_copy(dir: &Path) -> PathBuf {
    let a = dir.join("assets");
    support::copy_dir(&manifest().join("../assets"), &a);
    a
}

fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ono-course"))
        .args(args)
        .output()
        .expect("run ono-course")
}

fn p(path: &Path) -> &str {
    path.to_str().unwrap()
}

#[track_caller]
fn assert_exit(o: &Output, code: i32) {
    assert_eq!(
        o.status.code(),
        Some(code),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    );
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

fn build(root: &Path, assets: &Path, out: &Path) -> Output {
    cli(&[
        "build",
        "--root",
        p(root),
        "--assets",
        p(assets),
        "--out",
        p(out),
    ])
}

#[test]
fn build_check_offline_and_package_end_to_end() {
    let tmp = tempfile::tempdir().unwrap();
    let assets = assets_copy(tmp.path());
    let dist = tmp.path().join("dist");
    let o = build(&mini(), &assets, &dist);
    assert_exit(&o, 0);
    assert!(stderr(&o).contains("site check passed"), "{}", stderr(&o));
    assert!(dist.join("index.html").is_file());

    assert_exit(
        &cli(&["check-offline", "--root", p(&mini()), "--dir", p(&dist)]),
        0,
    );

    let release = tmp.path().join("release");
    let o = cli(&[
        "package",
        "--root",
        p(&mini()),
        "--dist",
        p(&dist),
        "--out",
        p(&release),
    ]);
    assert_exit(&o, 0);
    let mut names: Vec<String> = fs::read_dir(&release)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            "SHA256SUMS",
            "ono-sendai-rust-reading-course-v0.9.0.tar.gz",
            "ono-sendai-rust-reading-course-v0.9.0.zip",
        ]
    );
    let sums = fs::read_to_string(release.join("SHA256SUMS")).unwrap();
    for name in &names[1..] {
        let hash = ono_course::package::sha256_file(&release.join(name)).unwrap();
        assert!(sums.contains(&format!("{hash}  {name}\n")), "{sums}");
    }
}

#[test]
fn build_fails_on_a_network_api_in_runtime_javascript() {
    let tmp = tempfile::tempdir().unwrap();
    let assets = assets_copy(tmp.path());
    let js = assets.join("course.js");
    let text = fs::read_to_string(&js).unwrap();
    fs::write(&js, format!("{text}\nfetch('https://example.com/t');\n")).unwrap();
    let o = build(&mini(), &assets, &tmp.path().join("dist"));
    assert_exit(&o, 1);
    let err = stderr(&o);
    assert!(err.contains("network-capable API `fetch(`"), "{err}");
    assert!(
        err.contains("external URL `https://example.com/t`"),
        "{err}"
    );
    assert!(err.contains("site check failed"), "{err}");
}

#[test]
fn build_fails_on_a_remote_stylesheet_resource() {
    let tmp = tempfile::tempdir().unwrap();
    let assets = assets_copy(tmp.path());
    let css = assets.join("course.css");
    let text = fs::read_to_string(&css).unwrap();
    fs::write(
        &css,
        format!("{text}\nbody{{font-family:x;src:url(https://fonts.example/x.woff2)}}\n"),
    )
    .unwrap();
    let o = build(&mini(), &assets, &tmp.path().join("dist"));
    assert_exit(&o, 1);
    assert!(
        stderr(&o).contains("CSS url() must only use inline data: URIs"),
        "{}",
        stderr(&o)
    );
}

#[test]
fn build_fails_on_a_missing_required_asset() {
    let tmp = tempfile::tempdir().unwrap();
    let assets = assets_copy(tmp.path());
    fs::remove_file(assets.join("favicon.svg")).unwrap();
    let dist = tmp.path().join("dist");
    let o = build(&mini(), &assets, &dist);
    assert!(!o.status.success(), "{}", stderr(&o));
    assert!(
        stderr(&o).contains("required asset `favicon.svg` is missing"),
        "{}",
        stderr(&o)
    );
    assert!(!dist.exists(), "no partial site may be left behind");
}

#[test]
fn build_fails_on_an_invalid_course_without_writing_output() {
    let f = support::Fixture::new();
    f.replace(
        "course/chapters/01-basics.yaml",
        "stage: guided",
        "stage: expert",
    );
    let tmp = tempfile::tempdir().unwrap();
    let dist = tmp.path().join("dist");
    let o = build(f.root(), &assets_copy(tmp.path()), &dist);
    assert_exit(&o, 1);
    assert!(
        stderr(&o).contains("course source invalid"),
        "{}",
        stderr(&o)
    );
    assert!(!dist.exists());
}

#[test]
fn check_offline_fails_without_a_site() {
    let tmp = tempfile::tempdir().unwrap();
    let o = cli(&[
        "check-offline",
        "--root",
        p(&mini()),
        "--dir",
        p(&tmp.path().join("nope")),
    ]);
    assert_exit(&o, 1);
    assert!(
        stderr(&o).contains("output directory does not exist"),
        "{}",
        stderr(&o)
    );
}

#[test]
fn package_refuses_a_site_built_for_another_version() {
    let tmp = tempfile::tempdir().unwrap();
    let dist = tmp.path().join("dist");
    assert_exit(&build(&mini(), &assets_copy(tmp.path()), &dist), 0);
    // The lock moves on (a version bump) but the site is not rebuilt.
    let root = tmp.path().join("course-root");
    support::copy_dir(&mini(), &root);
    let lock = root.join("course-lock.yaml");
    let text = fs::read_to_string(&lock).unwrap();
    fs::write(&lock, text.replace("version: 0.9.0", "version: 0.9.1")).unwrap();
    let release = tmp.path().join("release");
    let o = cli(&[
        "package",
        "--root",
        p(&root),
        "--dist",
        p(&dist),
        "--out",
        p(&release),
    ]);
    assert_exit(&o, 2);
    assert!(
        stderr(&o).contains("course.version is 0.9.0 in the site but 0.9.1 in course-lock.yaml"),
        "{}",
        stderr(&o)
    );
    assert!(
        !release.exists(),
        "nothing may be packaged from a stale site"
    );
}
