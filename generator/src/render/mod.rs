//! Rendering the validated course into a static multi-page site (spec §29–§31, §35–§61).
//!
//! Every page is a plain HTML file at depth 0 (site root) or depth 1 (`lessons/`, `chapters/`),
//! linked with relative, explicit `.html` URLs so the site works under `file://`. Output is fully
//! deterministic: no timestamps, sorted iteration, and identical bytes on every run.
//!
//! The markup contract that the stylesheet, `course.js` and the browser tests rely on is
//! documented in `docs/frontend.md`.

mod code;
mod exercise;
mod layout;
mod lesson;
mod meta;
mod pages;

#[cfg(test)]
mod tests;

pub use code::mark_range;
pub use meta::content_digest;

use crate::load::Course;
use crate::markdown;
use crate::model::{Chapter, Lesson, Stage};
use crate::paths;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Version of the generator that produced a site, shown on the About page and in the metadata.
pub const GENERATOR_VERSION: &str = env!("CARGO_PKG_VERSION");

/// A lesson with its position in the course.
pub(crate) struct LessonRef<'a> {
    pub chapter: &'a Chapter,
    /// 0-based chapter position.
    pub chapter_index: usize,
    /// 0-based position inside the chapter.
    pub index_in_chapter: usize,
    /// 0-based position in the whole course.
    pub global: usize,
    pub lesson: &'a Lesson,
}

/// The course plus derived lookup tables used by every page renderer.
pub(crate) struct Site<'a> {
    pub course: &'a Course,
    pub lessons: Vec<LessonRef<'a>>,
    pub by_id: BTreeMap<&'a str, usize>,
}

impl<'a> Site<'a> {
    pub fn new(course: &'a Course) -> Site<'a> {
        let mut lessons = Vec::new();
        for (ci, (_, chapter)) in course.chapters.iter().enumerate() {
            for (li, lesson) in chapter.lessons.iter().enumerate() {
                let global = lessons.len();
                lessons.push(LessonRef {
                    chapter,
                    chapter_index: ci,
                    index_in_chapter: li,
                    global,
                    lesson,
                });
            }
        }
        let by_id = lessons
            .iter()
            .map(|l| (l.lesson.id.as_str(), l.global))
            .collect();
        Site {
            course,
            lessons,
            by_id,
        }
    }

    pub fn lesson(&self, id: &str) -> Option<&LessonRef<'a>> {
        self.by_id.get(id).map(|&i| &self.lessons[i])
    }

    pub fn chapters(&self) -> impl Iterator<Item = &'a Chapter> {
        self.course.chapters.iter().map(|(_, c)| c)
    }

    /// Lessons of the chapter at `chapter_index`, in order.
    pub fn chapter_lessons(&self, chapter_index: usize) -> impl Iterator<Item = &LessonRef<'a>> {
        self.lessons
            .iter()
            .filter(move |l| l.chapter_index == chapter_index)
    }

    /// Lowest and highest stage of a chapter's lessons.
    pub fn chapter_stages(&self, chapter: &Chapter) -> (Stage, Stage) {
        let lo = chapter
            .lessons
            .iter()
            .map(|l| l.stage)
            .min()
            .unwrap_or(Stage::Guided);
        let hi = chapter
            .lessons
            .iter()
            .map(|l| l.stage)
            .max()
            .unwrap_or(Stage::Guided);
        (lo, hi)
    }

    pub fn short_commit(&self) -> &str {
        let c = &self.course.lock.ono_sendai.commit;
        &c[..c.len().min(7)]
    }

    /// "Ono-Sendai v0.6.2", or just "Ono-Sendai" when the lock names no version.
    pub fn ono_label(&self) -> String {
        match &self.course.lock.ono_sendai.version {
            Some(v) if !v.trim().is_empty() => format!("Ono-Sendai {v}"),
            _ => "Ono-Sendai".to_string(),
        }
    }

    /// Render block Markdown relative to `root`, naming `ctx` in errors.
    pub fn md(&self, text: &str, root: &str, ctx: &str) -> Result<String, String> {
        markdown::render(text, self.course, root).map_err(|e| format!("{ctx}: {}", e.join("; ")))
    }

    /// Render one-line Markdown (the outer paragraph removed).
    pub fn md_inline(&self, text: &str, root: &str, ctx: &str) -> Result<String, String> {
        markdown::render_inline(text, self.course, root)
            .map_err(|e| format!("{ctx}: {}", e.join("; ")))
    }
}

/// The relative path from a page at `path` back to the site root.
pub(crate) fn root_of(path: &str) -> &'static str {
    if path.contains('/') {
        "../"
    } else {
        ""
    }
}

/// All generated files, keyed by their path relative to the output directory.
pub(crate) type Files = BTreeMap<String, Vec<u8>>;

/// Render every page and data file of the site into memory.
pub(crate) fn render_files(course: &Course) -> Result<Files, String> {
    let site = Site::new(course);
    let mut files = Files::new();
    let mut put = |path: String, text: String| {
        files.insert(path, text.into_bytes());
    };
    put(paths::INDEX.to_string(), pages::index(&site)?);
    for (ci, chapter) in site.chapters().enumerate() {
        put(
            paths::chapter(chapter.number, &chapter.id),
            pages::chapter(&site, ci)?,
        );
    }
    for l in &site.lessons {
        put(
            paths::lesson(&l.lesson.id),
            lesson::lesson_page(&site, l.global)?,
        );
    }
    put(paths::LEARN_RUST.to_string(), pages::learn_rust(&site)?);
    put(paths::ONO_SENDAI.to_string(), pages::ono_sendai(&site)?);
    put(paths::GLOSSARY.to_string(), pages::glossary(&site)?);
    let digest = meta::content_digest(&course.root)?;
    put(paths::ABOUT.to_string(), pages::about(&site, &digest)?);
    put(
        paths::METADATA.to_string(),
        meta::metadata_json(&site, &digest)?,
    );
    put(meta::README.to_string(), meta::readme(&site));
    Ok(files)
}

/// Render the course into `out`, copying runtime assets from `assets`.
///
/// `out` is deleted and recreated, but only if it does not exist, is empty, or holds a previously
/// generated course (it contains `course-metadata.json`); any other directory is left untouched
/// and reported as an error.
pub fn render_site(course: &Course, assets: &Path, out: &Path) -> Result<(), String> {
    let mut files = render_files(course)?;
    for (rel, bytes) in read_assets(assets)? {
        let key = format!("assets/{rel}");
        if files.insert(key.clone(), bytes).is_some() {
            return Err(format!("asset `{key}` collides with a generated file"));
        }
    }
    prepare_out(out, assets)?;
    for (rel, bytes) in &files {
        let path = out.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(())
}

/// Runtime assets every page links to; missing ones are a build error.
const REQUIRED_ASSETS: &[&str] = &[
    "course.css",
    "course.js",
    "favicon.svg",
    "ONO-SENDAI-LICENSE.txt",
];

/// All files below `assets`, as sorted `(relative path, bytes)` pairs.
fn read_assets(assets: &Path) -> Result<Vec<(String, Vec<u8>)>, String> {
    if !assets.is_dir() {
        return Err(format!(
            "asset directory {} does not exist",
            assets.display()
        ));
    }
    let mut out = Vec::new();
    let mut stack = vec![assets.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        for entry in entries {
            let p = entry.map_err(|e| e.to_string())?.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                let rel = p
                    .strip_prefix(assets)
                    .map_err(|e| e.to_string())?
                    .to_string_lossy()
                    .replace('\\', "/");
                let bytes = fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?;
                out.push((rel, bytes));
            }
        }
    }
    out.sort();
    for required in REQUIRED_ASSETS {
        if !out.iter().any(|(r, _)| r == required) {
            return Err(format!(
                "required asset `{required}` is missing from {}",
                assets.display()
            ));
        }
    }
    Ok(out)
}

/// Delete and recreate the output directory, refusing to delete anything that is not a
/// generated course.
fn prepare_out(out: &Path, assets: &Path) -> Result<(), String> {
    if out.exists() {
        if !out.is_dir() {
            return Err(format!(
                "output path {} exists and is not a directory",
                out.display()
            ));
        }
        if let (Ok(o), Ok(a)) = (out.canonicalize(), assets.canonicalize()) {
            if a.starts_with(&o) {
                return Err(format!(
                    "refusing to use {} as output directory: it contains the asset sources",
                    out.display()
                ));
            }
        }
        let empty = fs::read_dir(out)
            .map_err(|e| format!("{}: {e}", out.display()))?
            .next()
            .is_none();
        if !empty && !out.join(paths::METADATA).is_file() {
            return Err(format!(
                "refusing to delete {}: it is not empty and does not look like a generated course (no {})",
                out.display(),
                paths::METADATA
            ));
        }
        fs::remove_dir_all(out).map_err(|e| format!("cannot remove {}: {e}", out.display()))?;
    }
    fs::create_dir_all(out).map_err(|e| format!("cannot create {}: {e}", out.display()))
}
