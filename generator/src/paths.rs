//! Output paths of the generated site. Every page is at depth 0 (site root) or depth 1
//! (`lessons/`, `chapters/`), so relative links never need more than one `../`.

use crate::load::Course;
use crate::markdown::LinkResolver;

pub const INDEX: &str = "index.html";
pub const LEARN_RUST: &str = "learn-rust.html";
pub const ONO_SENDAI: &str = "ono-sendai.html";
pub const GLOSSARY: &str = "glossary.html";
pub const ABOUT: &str = "about.html";
pub const METADATA: &str = "course-metadata.json";

pub fn lesson(id: &str) -> String {
    format!("lessons/{id}.html")
}

pub fn chapter(number: u32, id: &str) -> String {
    format!("chapters/{number:02}-{id}.html")
}

pub fn concept_anchor(id: &str) -> String {
    format!("concept-{id}")
}

pub fn topic_anchor(id: &str) -> String {
    format!("topic-{id}")
}

pub fn term_anchor(id: &str) -> String {
    format!("term-{id}")
}

impl LinkResolver for Course {
    fn resolve(&self, scheme: &str, id: &str, root: &str) -> Option<String> {
        let path = match scheme {
            "lesson" => self
                .chapters
                .iter()
                .flat_map(|(_, c)| &c.lessons)
                .any(|l| l.id == id)
                .then(|| lesson(id))?,
            "chapter" => self
                .chapters
                .iter()
                .find(|(_, c)| c.id == id)
                .map(|(_, c)| chapter(c.number, &c.id))?,
            "concept" => self
                .curriculum
                .concepts
                .iter()
                .any(|c| c.id == id)
                .then(|| format!("{LEARN_RUST}#{}", concept_anchor(id)))?,
            "topic" => self
                .curriculum
                .ono_topics
                .iter()
                .any(|c| c.id == id)
                .then(|| format!("{ONO_SENDAI}#{}", topic_anchor(id)))?,
            "glossary" => self
                .glossary
                .terms
                .iter()
                .any(|t| t.id == id)
                .then(|| format!("{GLOSSARY}#{}", term_anchor(id)))?,
            _ => return None,
        };
        Some(format!("{root}{path}"))
    }
}
