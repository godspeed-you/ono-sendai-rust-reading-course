//! Command-line interface. `scripts/course <command>` runs this binary from the repository root.

use ono_course::{load, load_and_validate, package, render, sitecheck, snippet, upstream};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "\
Ono-Sendai Rust Reading Course — generator and maintenance tool

Usage: scripts/course <command> [options]

Commands:
  build [--out DIR] [--assets DIR]  validate the course, generate the site (default: dist/)
                                    and run the offline and link checks on it; runtime assets
                                    are copied from --assets (default: <root>/assets)
  validate [--ono CHECKOUT]         validate the course source; with --ono also validate every
                                    snippet against an Ono-Sendai checkout at the pinned commit
  check-offline [--dir DIR]         check a generated site for external resources, network APIs,
                                    broken internal links and missing assets (default: dist/)
  package [--dist DIR] [--out DIR]  create the release .zip, .tar.gz and SHA256SUMS
                                    (defaults: dist/ and release/)
  check-upstream --ono CHECKOUT     compare every snippet with another (newer) Ono-Sendai
                                    checkout and report changed, moved and missing snippets
                                    together with the lessons they affect
  snippet add --ono CHECKOUT --id ID --file PATH --lines A-B [--lines C-D ...]
              --anchor TEXT [--anchor TEXT ...] [--force]
                                    extract an exact snippet from a checkout of the pinned
                                    commit into course/snippets/<crate>/<id>.yaml
  help                              show this text

Options:
  --root DIR                        course repository root (default: current directory)
";

struct Args {
    rest: Vec<String>,
}

impl Args {
    fn value(&mut self, flag: &str) -> Result<Option<String>, String> {
        if let Some(i) = self.rest.iter().position(|a| a == flag) {
            if i + 1 >= self.rest.len() {
                return Err(format!("{flag} needs a value"));
            }
            let v = self.rest.remove(i + 1);
            self.rest.remove(i);
            return Ok(Some(v));
        }
        Ok(None)
    }

    fn values(&mut self, flag: &str) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        while let Some(v) = self.value(flag)? {
            out.push(v);
        }
        Ok(out)
    }

    fn flag(&mut self, flag: &str) -> bool {
        if let Some(i) = self.rest.iter().position(|a| a == flag) {
            self.rest.remove(i);
            return true;
        }
        false
    }

    fn finish(&self) -> Result<(), String> {
        match self.rest.first() {
            Some(a) => Err(format!("unexpected argument `{a}`")),
            None => Ok(()),
        }
    }
}

fn main() -> ExitCode {
    let mut argv: Vec<String> = std::env::args().skip(1).collect();
    if argv.is_empty() {
        eprint!("{USAGE}");
        return ExitCode::from(2);
    }
    let cmd = argv.remove(0);
    let mut args = Args { rest: argv };
    let result = (|| -> Result<bool, String> {
        let root = PathBuf::from(args.value("--root")?.unwrap_or_else(|| ".".into()));
        match cmd.as_str() {
            "build" => {
                let out = args
                    .value("--out")?
                    .map_or_else(|| root.join("dist"), PathBuf::from);
                let assets = args
                    .value("--assets")?
                    .map_or_else(|| root.join("assets"), PathBuf::from);
                args.finish()?;
                cmd_build(&root, &assets, &out)
            }
            "validate" => {
                let ono = args.value("--ono")?;
                args.finish()?;
                cmd_validate(&root, ono.as_deref().map(Path::new))
            }
            "check-offline" => {
                let dir = args
                    .value("--dir")?
                    .map_or_else(|| root.join("dist"), PathBuf::from);
                args.finish()?;
                cmd_check_offline(&root, &dir)
            }
            "package" => {
                let dist = args
                    .value("--dist")?
                    .map_or_else(|| root.join("dist"), PathBuf::from);
                let out = args
                    .value("--out")?
                    .map_or_else(|| root.join("release"), PathBuf::from);
                args.finish()?;
                cmd_package(&root, &dist, &out)
            }
            "check-upstream" => {
                let ono = args
                    .value("--ono")?
                    .ok_or("check-upstream needs --ono CHECKOUT")?;
                args.finish()?;
                cmd_check_upstream(&root, Path::new(&ono))
            }
            "snippet" => {
                let sub = if args.rest.is_empty() {
                    String::new()
                } else {
                    args.rest.remove(0)
                };
                if sub != "add" {
                    return Err("usage: scripts/course snippet add ...".into());
                }
                cmd_snippet_add(&root, &mut args)
            }
            "help" | "--help" | "-h" => {
                print!("{USAGE}");
                Ok(true)
            }
            other => Err(format!("unknown command `{other}`\n\n{USAGE}")),
        }
    })();
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(2)
        }
    }
}

fn validated(root: &Path) -> Option<load::Course> {
    let (course, diags) = load_and_validate(root);
    eprint!("{}", diags.report());
    let errors = diags.errors().count();
    let warnings = diags.items.len() - errors;
    match course {
        Some(c) if errors == 0 => {
            let lessons: usize = c.chapters.iter().map(|(_, ch)| ch.lessons.len()).sum();
            eprintln!(
                "course source valid: {} chapters, {lessons} lessons, {} snippets, {warnings} warning(s)",
                c.chapters.len(),
                c.snippets.len()
            );
            Some(c)
        }
        _ => {
            eprintln!("course source invalid: {errors} error(s), {warnings} warning(s)");
            None
        }
    }
}

fn site_ok(course: &load::Course, dir: &Path) -> bool {
    let diags = sitecheck::check_site(
        dir,
        &sitecheck::SiteCheckOptions {
            allowed_text_url: &course.lock.ono_sendai.repository,
        },
    );
    eprint!("{}", diags.report());
    if diags.has_errors() {
        eprintln!(
            "site check failed: {} error(s) in {}",
            diags.errors().count(),
            dir.display()
        );
        false
    } else {
        eprintln!(
            "site check passed: offline, links and assets OK ({})",
            dir.display()
        );
        true
    }
}

fn cmd_build(root: &Path, assets: &Path, out: &Path) -> Result<bool, String> {
    let Some(course) = validated(root) else {
        return Ok(false);
    };
    render::render_site(&course, assets, out)?;
    eprintln!("generated {}", out.display());
    Ok(site_ok(&course, out))
}

fn cmd_validate(root: &Path, ono: Option<&Path>) -> Result<bool, String> {
    let Some(course) = validated(root) else {
        return Ok(false);
    };
    let Some(ono) = ono else { return Ok(true) };
    let report = upstream::validate_pinned(&course, ono);
    print!(
        "{}",
        report.render("Ono-Sendai source validation", &course, ono)
    );
    Ok(report.ok())
}

fn cmd_check_offline(root: &Path, dir: &Path) -> Result<bool, String> {
    let mut diags = ono_course::diag::Diagnostics::default();
    let course = load::load(root, &mut diags).ok_or("cannot load course-lock.yaml")?;
    Ok(site_ok(&course, dir))
}

fn cmd_package(root: &Path, dist: &Path, out: &Path) -> Result<bool, String> {
    let mut diags = ono_course::diag::Diagnostics::default();
    let course = load::load(root, &mut diags).ok_or("cannot load course-lock.yaml")?;
    if !site_ok(&course, dist) {
        return Ok(false);
    }
    package::check_metadata(dist, &course.lock)?;
    package::check_digest(dist, &render::content_digest(root)?)?;
    let p = package::package(dist, out, &course.lock.course.version)?;
    for f in [&p.zip, &p.tar_gz, &p.sums] {
        let size = std::fs::metadata(f).map(|m| m.len()).unwrap_or(0);
        println!("{}  ({} KiB)", f.display(), size.div_ceil(1024));
    }
    Ok(true)
}

fn cmd_check_upstream(root: &Path, ono: &Path) -> Result<bool, String> {
    let Some(course) = validated(root) else {
        return Ok(false);
    };
    let report = upstream::check_newer(&course, ono);
    print!(
        "{}",
        report.render("Ono-Sendai upstream comparison", &course, ono)
    );
    println!(
        "\nThis comparison never updates the pin. Changed snippets need their lessons reviewed and\n\
         the snippet re-extracted with `scripts/course snippet add` once course-lock.yaml pins the new commit."
    );
    Ok(report.ok())
}

fn cmd_snippet_add(root: &Path, args: &mut Args) -> Result<bool, String> {
    let ono = args.value("--ono")?.ok_or("--ono CHECKOUT is required")?;
    let id = args.value("--id")?.ok_or("--id is required")?;
    let file = args.value("--file")?.ok_or("--file is required")?;
    let lines = args.values("--lines")?;
    let anchors = args.values("--anchor")?;
    let force = args.flag("--force");
    args.finish()?;
    if lines.is_empty() || lines.len() != anchors.len() {
        return Err("give one --anchor per --lines range".into());
    }
    let mut diags = ono_course::diag::Diagnostics::default();
    let course = load::load(root, &mut diags).ok_or("cannot load course-lock.yaml")?;
    let pinned = &course.lock.ono_sendai.commit;
    let ono = Path::new(&ono);
    match upstream::git_rev(ono, "HEAD") {
        Some(h) if &h == pinned => {}
        Some(h) => {
            return Err(format!(
                "checkout is at {h}, but course-lock.yaml pins {pinned}"
            ))
        }
        None => return Err("cannot determine the checkout's commit".into()),
    }
    let mut ranges = Vec::new();
    for (l, a) in lines.iter().zip(anchors) {
        let spec: ono_course::model::LineSpec = l.parse()?;
        ranges.push((spec.start, spec.end, a));
    }
    let text = std::fs::read_to_string(ono.join(&file)).map_err(|e| format!("{file}: {e}"))?;
    let s = snippet::build_snippet(&id, &file, pinned, &text, &ranges)?;
    let group = file
        .strip_prefix("crates/")
        .and_then(|r| r.split('/').next())
        .unwrap_or_else(|| file.split('/').next().unwrap_or("misc"));
    let dir = root.join(load::SNIPPET_DIR).join(group);
    let path = dir.join(format!("{id}.yaml"));
    if path.exists() && !force {
        return Err(format!(
            "{} already exists (use --force to replace)",
            path.display()
        ));
    }
    if let Some((existing, _)) = course.snippets.get(&id) {
        if !force {
            return Err(format!("snippet id `{id}` already exists in {existing}"));
        }
    }
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let yaml = serde_yaml::to_string(&s).map_err(|e| e.to_string())?;
    std::fs::write(&path, yaml).map_err(|e| e.to_string())?;
    println!(
        "wrote {} ({} segment(s), {} lines)",
        path.display(),
        s.segments.len(),
        s.segments
            .iter()
            .map(|g| g.end_line - g.start_line + 1)
            .sum::<u32>()
    );
    Ok(true)
}
