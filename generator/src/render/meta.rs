//! Build metadata (spec §68): the content digest, `course-metadata.json` and `README.txt`.

use super::Site;
use crate::load::{yaml_files, LOCK_FILE};
use crate::paths;
use crate::snippet::hex;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub const README: &str = "README.txt";

/// SHA-256 over every course source file: `course-lock.yaml` and `course/**/*.yaml`, sorted by
/// path. Each file contributes its `/`-separated relative path, a NUL byte, its length as a
/// little-endian u64 and its bytes, so renames and content changes both change the digest.
pub fn content_digest(root: &Path) -> Result<String, String> {
    let mut files: Vec<(String, std::path::PathBuf)> =
        vec![(LOCK_FILE.to_string(), root.join(LOCK_FILE))];
    for p in yaml_files(&root.join("course")) {
        let rel = p
            .strip_prefix(root)
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        files.push((rel, p));
    }
    files.sort();
    let mut h = Sha256::new();
    for (rel, p) in files {
        let bytes = fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?;
        h.update(rel.as_bytes());
        h.update([0u8]);
        h.update((bytes.len() as u64).to_le_bytes());
        h.update(&bytes);
    }
    Ok(format!("sha256:{}", hex(&h.finalize())))
}

#[derive(Serialize)]
struct Metadata<'a> {
    course: CourseMeta<'a>,
    ono_sendai: OnoMeta<'a>,
    generator: GeneratorMeta<'a>,
    content_digest: &'a str,
    lessons: Vec<LessonMeta<'a>>,
    snippets: Vec<SnippetMeta<'a>>,
}

#[derive(Serialize)]
struct CourseMeta<'a> {
    title: &'a str,
    version: &'a str,
}

#[derive(Serialize)]
struct OnoMeta<'a> {
    repository: &'a str,
    commit: &'a str,
    version: Option<&'a str>,
}

#[derive(Serialize)]
struct GeneratorMeta<'a> {
    name: &'a str,
    version: &'a str,
}

#[derive(Serialize)]
struct LessonMeta<'a> {
    id: &'a str,
    title: &'a str,
    chapter: u32,
    chapter_id: &'a str,
    stage: &'a str,
    path: String,
}

#[derive(Serialize)]
struct SnippetMeta<'a> {
    id: &'a str,
    file: &'a str,
    lines: Vec<String>,
    commit: &'a str,
    content_hashes: Vec<&'a str>,
}

pub(crate) fn metadata_json(site: &Site, digest: &str) -> Result<String, String> {
    let lock = &site.course.lock;
    let meta = Metadata {
        course: CourseMeta {
            title: &site.course.curriculum.title,
            version: &lock.course.version,
        },
        ono_sendai: OnoMeta {
            repository: &lock.ono_sendai.repository,
            commit: &lock.ono_sendai.commit,
            version: lock.ono_sendai.version.as_deref(),
        },
        generator: GeneratorMeta {
            name: "ono-course",
            version: super::GENERATOR_VERSION,
        },
        content_digest: digest,
        lessons: site
            .lessons
            .iter()
            .map(|l| LessonMeta {
                id: &l.lesson.id,
                title: &l.lesson.title,
                chapter: l.chapter.number,
                chapter_id: &l.chapter.id,
                stage: l.lesson.stage.slug(),
                path: paths::lesson(&l.lesson.id),
            })
            .collect(),
        snippets: site
            .course
            .snippets
            .values()
            .map(|(_, s)| SnippetMeta {
                id: &s.id,
                file: &s.source.file,
                lines: s
                    .segments
                    .iter()
                    .map(|g| format!("{}-{}", g.start_line, g.end_line))
                    .collect(),
                commit: &s.source.commit,
                content_hashes: s.segments.iter().map(|g| g.content_hash.as_str()).collect(),
            })
            .collect(),
    };
    let mut json = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?;
    json.push('\n');
    Ok(json)
}

pub(crate) fn readme(site: &Site) -> String {
    let lock = &site.course.lock;
    let version = lock
        .ono_sendai
        .version
        .as_deref()
        .filter(|v| !v.trim().is_empty());
    format!(
        "{title}\n{underline}\n\n\
         Course version {cv}\n\
         Reads Ono-Sendai{ov} at commit {commit}\n\n\
         HOW TO OPEN\n\n\
         Open index.html in a web browser (double-click it, or drag it into a browser window).\n\
         That is all. Nothing needs to be installed, no server has to run, and the course works\n\
         completely offline: every page, style, script and code snippet is inside this folder.\n\n\
         Keep the folder together: the pages link to each other and to the assets/ folder with\n\
         relative links.\n\n\
         Your progress (completed lessons, notes, ticked checklists) is stored only in your\n\
         browser, if it allows that. The About page (about.html) can reset it.\n\n\
         Static. Offline. No server. No account. No LLM.\n\n\
         LICENCES\n\n\
         The code snippets are quoted verbatim from Ono-Sendai, which is MIT licensed,\n\
         copyright (c) 2026 Marcel Arentz; see assets/ONO-SENDAI-LICENSE.txt.\n\
         The course itself (explanations, exercises, generator and page design) is licensed\n\
         under the Apache License 2.0.\n",
        title = site.course.curriculum.title,
        underline = "=".repeat(site.course.curriculum.title.chars().count()),
        cv = lock.course.version,
        ov = version.map(|v| format!(" {v}")).unwrap_or_default(),
        commit = lock.ono_sendai.commit,
    )
}
