//! Loading the course source tree. Parsing errors are collected as diagnostics with the file they
//! came from; nothing is skipped silently.

use crate::diag::Diagnostics;
use crate::model::{Chapter, CourseLock, Curriculum, Glossary, Snippet};
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// The whole course source, as loaded from a repository root.
#[derive(Debug, Clone)]
pub struct Course {
    pub root: PathBuf,
    pub lock: CourseLock,
    pub curriculum: Curriculum,
    pub glossary: Glossary,
    /// Chapters in course order, with the path they were loaded from (relative to `root`).
    pub chapters: Vec<(String, Chapter)>,
    /// Snippets by id, with the path they were loaded from (relative to `root`).
    pub snippets: BTreeMap<String, (String, Snippet)>,
}

pub const LOCK_FILE: &str = "course-lock.yaml";
pub const CURRICULUM_FILE: &str = "course/curriculum.yaml";
pub const GLOSSARY_FILE: &str = "course/glossary.yaml";
pub const SNIPPET_DIR: &str = "course/snippets";

fn rel(root: &Path, p: &Path) -> String {
    p.strip_prefix(root)
        .unwrap_or(p)
        .to_string_lossy()
        .replace('\\', "/")
}

fn read_yaml<T: DeserializeOwned>(root: &Path, path: &Path, diags: &mut Diagnostics) -> Option<T> {
    let name = rel(root, path);
    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            diags.error(&name, format!("cannot read file: {e}"));
            return None;
        }
    };
    match serde_yaml::from_str(&text) {
        Ok(v) => Some(v),
        Err(e) => {
            diags.error(&name, format!("schema error: {e}"));
            None
        }
    }
}

/// All `*.yaml` files below `dir`, sorted by path so loading order is deterministic.
pub fn yaml_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = fs::read_dir(&d) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "yaml") {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

/// Load a course. Returns `None` if a structural parse error makes validation impossible; all
/// problems found are recorded in `diags` either way.
pub fn load(root: &Path, diags: &mut Diagnostics) -> Option<Course> {
    let lock: Option<CourseLock> = read_yaml(root, &root.join(LOCK_FILE), diags);
    let curriculum: Option<Curriculum> = read_yaml(root, &root.join(CURRICULUM_FILE), diags);
    let glossary: Option<Glossary> = read_yaml(root, &root.join(GLOSSARY_FILE), diags);

    let mut chapters = Vec::new();
    if let Some(c) = &curriculum {
        for entry in &c.chapters {
            let path = root.join("course").join(entry);
            if !path.is_file() {
                diags.error(
                    CURRICULUM_FILE,
                    format!("chapter file `{entry}` does not exist"),
                );
                continue;
            }
            if let Some(ch) = read_yaml::<Chapter>(root, &path, diags) {
                chapters.push((rel(root, &path), ch));
            }
        }
        // A chapter file that exists but is not listed would silently vanish from the course.
        for p in yaml_files(&root.join("course/chapters")) {
            let name = rel(root, &p);
            let listed = c.chapters.iter().any(|e| format!("course/{e}") == name);
            if !listed {
                diags.error(
                    &name,
                    "chapter file is not listed in course/curriculum.yaml",
                );
            }
        }
    }

    let mut snippets = BTreeMap::new();
    let snippet_dir = root.join(SNIPPET_DIR);
    for p in yaml_files(&snippet_dir) {
        let name = rel(root, &p);
        if let Some(s) = read_yaml::<Snippet>(root, &p, diags) {
            if let Some((other, _)) = snippets.get(&s.id) {
                diags.error(
                    &name,
                    format!("duplicate snippet id `{}` (also in {other})", s.id),
                );
                continue;
            }
            snippets.insert(s.id.clone(), (name, s));
        }
    }

    Some(Course {
        root: root.to_path_buf(),
        lock: lock?,
        curriculum: curriculum?,
        glossary: glossary?,
        chapters,
        snippets,
    })
}
