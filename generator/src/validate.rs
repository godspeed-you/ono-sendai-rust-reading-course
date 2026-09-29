//! Semantic validation of the course source (spec §26, §64). Structural problems are errors and
//! make the build fail; nothing is silently ignored.

use crate::diag::Diagnostics;
use crate::load::{Course, CURRICULUM_FILE, GLOSSARY_FILE, LOCK_FILE};
use crate::markdown;
use crate::model::*;
use crate::snippet;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

/// Validate everything. Diagnostics are appended to `diags`.
pub fn validate(course: &Course, diags: &mut Diagnostics) {
    validate_lock(course, diags);
    validate_curriculum(course, diags);
    validate_snippets(course, diags);
    validate_lessons(course, diags);
    validate_glossary(course, diags);
    validate_unseen_code(course, diags);
    validate_progression(course, diags);
}

pub fn is_kebab(id: &str) -> bool {
    !id.is_empty()
        && !id.starts_with('-')
        && !id.ends_with('-')
        && !id.contains("--")
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

fn is_full_sha(s: &str) -> bool {
    s.len() == 40
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn is_semver(s: &str) -> bool {
    let core = s.split(['-', '+']).next().unwrap_or("");
    let parts: Vec<&str> = core.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
}

fn validate_lock(course: &Course, diags: &mut Diagnostics) {
    let lock = &course.lock;
    if !is_semver(&lock.course.version) {
        diags.error(
            LOCK_FILE,
            format!(
                "course.version `{}` is not semver (X.Y.Z)",
                lock.course.version
            ),
        );
    }
    if !is_full_sha(&lock.ono_sendai.commit) {
        diags.error(
            LOCK_FILE,
            format!(
                "ono_sendai.commit `{}` must be a full 40-character lowercase commit SHA",
                lock.ono_sendai.commit
            ),
        );
    }
    if lock.ono_sendai.repository.trim().is_empty() {
        diags.error(LOCK_FILE, "ono_sendai.repository must not be empty");
    }
}

fn check_md(course: &Course, diags: &mut Diagnostics, file: &str, ctx: &str, md: &str) {
    if md.trim().is_empty() {
        diags.error(file, format!("{ctx}: text must not be empty"));
        return;
    }
    if let Err(errs) = markdown::render(md, course, "") {
        for e in errs {
            diags.error(file, format!("{ctx}: {e}"));
        }
    }
}

/// Titles and lesson summaries are plain text (they also appear in `<title>`, `<meta>` and the
/// navigator), where only `code` spans are shown as code. A Markdown link there would be printed
/// literally instead of linking, so it is an error rather than silently broken prose.
fn check_plain(diags: &mut Diagnostics, file: &str, ctx: &str, text: &str) {
    if text.contains("](") {
        diags.error(
            file,
            format!("{ctx}: links are not supported in plain-text fields; link from the lesson's prose instead"),
        );
    }
}

fn validate_curriculum(course: &Course, diags: &mut Diagnostics) {
    let c = &course.curriculum;
    for (kind, list) in [("concept", &c.concepts), ("ono_topic", &c.ono_topics)] {
        let mut seen = HashSet::new();
        for t in list {
            if !is_kebab(&t.id) {
                diags.error(
                    CURRICULUM_FILE,
                    format!("{kind} id `{}` must be kebab-case", t.id),
                );
            }
            if !seen.insert(t.id.as_str()) {
                diags.error(CURRICULUM_FILE, format!("duplicate {kind} id `{}`", t.id));
            }
            check_md(
                course,
                diags,
                CURRICULUM_FILE,
                &format!("{kind} `{}` summary", t.id),
                &t.summary,
            );
        }
    }
    for (id, cl) in &c.checklists {
        if !is_kebab(id) {
            diags.error(
                CURRICULUM_FILE,
                format!("checklist id `{id}` must be kebab-case"),
            );
        }
        if cl.items.is_empty() {
            diags.error(CURRICULUM_FILE, format!("checklist `{id}` has no items"));
        }
    }
    if c.chapters.is_empty() {
        diags.error(CURRICULUM_FILE, "the curriculum lists no chapters");
    }
    let mut seen = HashSet::new();
    for (i, (file, ch)) in course.chapters.iter().enumerate() {
        if !is_kebab(&ch.id) {
            diags.error(file, format!("chapter id `{}` must be kebab-case", ch.id));
        }
        if !seen.insert(ch.id.as_str()) {
            diags.error(file, format!("duplicate chapter id `{}`", ch.id));
        }
        if ch.number as usize != i + 1 {
            diags.error(
                file,
                format!(
                    "chapter `{}` has number {} but is listed at position {}",
                    ch.id,
                    ch.number,
                    i + 1
                ),
            );
        }
        check_plain(
            diags,
            file,
            &format!("chapter `{}` title", ch.id),
            &ch.title,
        );
        if ch.lessons.is_empty() {
            diags.error(file, format!("chapter `{}` has no lessons", ch.id));
        }
        check_md(
            course,
            diags,
            file,
            &format!("chapter `{}` summary", ch.id),
            &ch.summary,
        );
    }
}

fn validate_snippets(course: &Course, diags: &mut Diagnostics) {
    let pinned = &course.lock.ono_sendai.commit;
    for (id, (file, s)) in &course.snippets {
        if !is_kebab(id) {
            diags.error(file, format!("snippet id `{id}` must be kebab-case"));
        }
        let stem = file
            .rsplit('/')
            .next()
            .unwrap_or("")
            .trim_end_matches(".yaml");
        if stem != id {
            diags.error(
                file,
                format!("snippet id `{id}` must match its file name `{stem}.yaml`"),
            );
        }
        if &s.source.commit != pinned {
            diags.error(
                file,
                format!(
                    "snippet `{id}` was taken from commit {} but course-lock.yaml pins {pinned}",
                    s.source.commit
                ),
            );
        }
        let p = &s.source.file;
        if p.is_empty() || p.starts_with('/') || p.split('/').any(|c| c == ".." || c.is_empty()) {
            diags.error(
                file,
                format!("snippet `{id}` has an invalid source path `{p}`"),
            );
        }
        if s.segments.is_empty() {
            diags.error(file, format!("snippet `{id}` has no segments"));
        }
        let mut last_end = 0;
        for seg in &s.segments {
            let range = format!("{}-{}", seg.start_line, seg.end_line);
            if seg.start_line == 0 || seg.end_line < seg.start_line {
                diags.error(
                    file,
                    format!("snippet `{id}`: invalid source range {range}"),
                );
                continue;
            }
            if seg.start_line <= last_end {
                diags.error(
                    file,
                    format!(
                        "snippet `{id}`: segments must be ascending and non-overlapping ({range})"
                    ),
                );
            }
            last_end = seg.end_line;
            let n = snippet::source_lines(&seg.code).len() as u32;
            if !seg.code.ends_with('\n') || n != seg.end_line - seg.start_line + 1 {
                diags.error(
                    file,
                    format!(
                        "snippet `{id}`: segment {range} must contain exactly {} lines, found {n}",
                        seg.end_line - seg.start_line + 1
                    ),
                );
            }
            if snippet::content_hash(&seg.code) != seg.content_hash {
                diags.error(
                    file,
                    format!("snippet `{id}`: content hash mismatch for {range}; the embedded code was edited — re-extract it with `scripts/course snippet add`"),
                );
            }
            let occurrences = snippet::count_occurrences(&seg.code, &seg.anchor);
            if occurrences != 1 {
                diags.error(
                    file,
                    format!("snippet `{id}`: anchor `{}` must occur exactly once in {range}, found {occurrences}", seg.anchor),
                );
            }
        }
    }
}

/// Every snippet id a lesson shows, in order of appearance.
pub fn lesson_snippets(lesson: &Lesson) -> Vec<&str> {
    let mut out = Vec::new();
    for s in &lesson.sections {
        match s {
            Section::Code { snippet, .. } => out.push(snippet.as_str()),
            Section::Exercise(e) => out.extend(e.snippets.iter().map(String::as_str)),
            _ => {}
        }
    }
    out
}

pub fn all_lessons(course: &Course) -> impl Iterator<Item = (&str, &Chapter, &Lesson)> {
    course
        .chapters
        .iter()
        .flat_map(|(f, c)| c.lessons.iter().map(move |l| (f.as_str(), c, l)))
}

fn validate_lessons(course: &Course, diags: &mut Diagnostics) {
    let concepts: HashSet<&str> = course
        .curriculum
        .concepts
        .iter()
        .map(|c| c.id.as_str())
        .collect();
    let topics: HashSet<&str> = course
        .curriculum
        .ono_topics
        .iter()
        .map(|c| c.id.as_str())
        .collect();
    let order: HashMap<&str, usize> = all_lessons(course)
        .enumerate()
        .map(|(i, (_, _, l))| (l.id.as_str(), i))
        .collect();
    let mut lesson_ids: HashMap<&str, &str> = HashMap::new();
    let mut exercise_ids: HashMap<&str, &str> = HashMap::new();
    let mut used_snippets = BTreeSet::new();

    for (idx, (file, _chapter, lesson)) in all_lessons(course).enumerate() {
        let lid = lesson.id.as_str();
        let at = |what: &str| format!("lesson `{lid}`: {what}");
        if !is_kebab(lid) {
            diags.error(file, at("id must be kebab-case"));
        }
        if let Some(other) = lesson_ids.insert(lid, file) {
            diags.error(
                file,
                format!("duplicate lesson id `{lid}` (also in {other})"),
            );
        }
        if lesson.title.trim().is_empty() {
            diags.error(file, at("title must not be empty"));
        }
        if lesson.summary.trim().is_empty() {
            diags.error(file, at("summary must not be empty"));
        }
        for (field, text) in [("title", &lesson.title), ("summary", &lesson.summary)] {
            check_plain(diags, file, &at(field), text);
        }
        if lesson.objectives.is_empty() {
            diags.error(file, at("needs at least one objective"));
        }
        if lesson.concepts.is_empty() {
            diags.error(file, at("needs at least one concept tag"));
        }
        if lesson.ono_topics.is_empty() {
            diags.error(file, at("needs at least one ono_topics tag"));
        }
        for c in &lesson.concepts {
            if !concepts.contains(c.as_str()) {
                diags.error(file, at(&format!("unknown concept tag `{c}`")));
            }
        }
        for t in &lesson.ono_topics {
            if !topics.contains(t.as_str()) {
                diags.error(file, at(&format!("unknown ono_topics tag `{t}`")));
            }
        }
        for p in &lesson.prerequisites {
            match order.get(p.as_str()) {
                None => diags.error(file, at(&format!("unknown prerequisite lesson `{p}`"))),
                Some(&i) if i >= idx => diags.error(
                    file,
                    at(&format!("prerequisite `{p}` must come earlier in the course (cyclic or forward prerequisite)")),
                ),
                _ => {}
            }
        }
        for o in &lesson.objectives {
            check_md(course, diags, file, &at("objective"), o);
        }
        if lesson.sections.is_empty() {
            diags.error(file, at("has no sections"));
        }
        let snippets = lesson_snippets(lesson);
        if snippets.is_empty() {
            diags.error(
                file,
                at("shows no Ono-Sendai code; every lesson must read real code"),
            );
        }
        let mut annotations = 0;
        let mut exercises = 0;
        for (si, section) in lesson.sections.iter().enumerate() {
            let sat = |what: &str| format!("lesson `{lid}` section {}: {what}", si + 1);
            match section {
                Section::Prose { body, title, .. } => {
                    check_md(course, diags, file, &sat("prose"), body);
                    if title.as_deref().is_some_and(|t| t.trim().is_empty()) {
                        diags.error(file, sat("title must not be empty"));
                    }
                }
                Section::Diagram {
                    art,
                    description,
                    title,
                    ..
                } => {
                    if art.trim().is_empty() || title.trim().is_empty() {
                        diags.error(file, sat("diagram needs a title and art"));
                    }
                    check_md(
                        course,
                        diags,
                        file,
                        &sat("diagram description (text equivalent)"),
                        description,
                    );
                }
                Section::Code {
                    snippet: sid,
                    caption,
                    highlight,
                    annotations: anns,
                } => {
                    used_snippets.insert(sid.as_str());
                    let Some((_, snip)) = course.snippets.get(sid) else {
                        diags.error(file, sat(&format!("unknown snippet `{sid}`")));
                        continue;
                    };
                    if let Some(c) = caption {
                        check_md(course, diags, file, &sat("caption"), c);
                    }
                    for h in highlight {
                        if !(h.start..=h.end).all(|l| snip.shows_line(l)) {
                            diags.error(
                                file,
                                sat(&format!("highlight {h} is outside snippet `{sid}`")),
                            );
                        }
                    }
                    for a in anns {
                        annotations += 1;
                        if !(a.lines.start..=a.lines.end).all(|l| snip.shows_line(l)) {
                            diags.error(
                                file,
                                sat(&format!(
                                    "annotation lines {} are outside snippet `{sid}`",
                                    a.lines
                                )),
                            );
                        } else if let Some(tok) = &a.token {
                            let line = line_text(snip, a.lines.start).unwrap_or("");
                            if tok.is_empty() || !line.contains(tok.as_str()) {
                                diags.error(
                                    file,
                                    sat(&format!("annotation token `{tok}` does not occur on line {} of snippet `{sid}`", a.lines.start)),
                                );
                            }
                        }
                        check_md(course, diags, file, &sat("annotation"), &a.body);
                    }
                }
                Section::Exercise(ex) => {
                    exercises += 1;
                    if let Some(other) = exercise_ids.insert(ex.id.as_str(), file) {
                        diags.error(
                            file,
                            format!("duplicate exercise id `{}` (also in {other})", ex.id),
                        );
                    }
                    for sid in &ex.snippets {
                        used_snippets.insert(sid.as_str());
                        if !course.snippets.contains_key(sid) {
                            diags.error(
                                file,
                                sat(&format!(
                                    "exercise `{}` refers to unknown snippet `{sid}`",
                                    ex.id
                                )),
                            );
                        }
                    }
                    validate_exercise(course, diags, file, lesson.stage, ex);
                }
            }
        }
        if lesson.stage == Stage::Guided && annotations == 0 {
            diags.error(file, at("guided lessons must annotate their code"));
        }
        if lesson.stage == Stage::Independent && exercises == 0 {
            diags.error(
                file,
                at("independent lessons must contain a reading exercise"),
            );
        }
    }

    for (id, (file, _)) in &course.snippets {
        if !used_snippets.contains(id.as_str()) {
            diags.warning(file, format!("snippet `{id}` is not used by any lesson"));
        }
    }
}

fn line_text(s: &Snippet, line: u32) -> Option<&str> {
    let seg = s
        .segments
        .iter()
        .find(|g| (g.start_line..=g.end_line).contains(&line))?;
    snippet::source_lines(&seg.code)
        .get((line - seg.start_line) as usize)
        .copied()
}

fn validate_exercise(
    course: &Course,
    diags: &mut Diagnostics,
    file: &str,
    stage: Stage,
    ex: &Exercise,
) {
    let eid = ex.id.as_str();
    let at = |what: &str| format!("exercise `{eid}`: {what}");
    if !is_kebab(eid) {
        diags.error(file, at("id must be kebab-case"));
    }
    check_md(course, diags, file, &at("prompt"), &ex.prompt);

    let mc = ex.exercise_type == ExerciseType::MultipleChoice;
    if mc {
        if stage == Stage::Independent {
            diags.error(
                file,
                at("multiple choice reveals an answer and is not allowed in independent lessons"),
            );
        }
        if !(2..=6).contains(&ex.choices.len()) {
            diags.error(file, at("multiple choice needs 2 to 6 choices"));
        }
        let correct = ex.choices.iter().filter(|c| c.correct).count();
        if correct != 1 {
            diags.error(
                file,
                at(&format!(
                    "multiple choice needs exactly one correct choice, found {correct}"
                )),
            );
        }
        for c in &ex.choices {
            check_md(course, diags, file, &at("choice text"), &c.text);
            check_md(course, diags, file, &at("choice feedback"), &c.feedback);
        }
    } else if !ex.choices.is_empty() {
        diags.error(file, at("only multiple-choice exercises may have choices"));
    }

    // Hints: levels 1..=n in order, at most the stage allows.
    for (i, h) in ex.hints.iter().enumerate() {
        if h.level as usize != i + 1 {
            diags.error(
                file,
                at(&format!(
                    "invalid hint ordering: hint {} has level {}, expected {}",
                    i + 1,
                    h.level,
                    i + 1
                )),
            );
        }
        check_md(course, diags, file, &at("hint"), &h.body);
    }
    if ex.hints.len() > stage.max_hints() {
        diags.error(
            file,
            at(&format!(
                "{stage} lessons allow at most {} hint level(s), found {}",
                stage.max_hints(),
                ex.hints.len()
            )),
        );
    }

    // Solutions: forbidden where the stage says so, required everywhere else.
    match (&ex.solution, stage.allows_solution()) {
        (Some(_), false) => diags.error(
            file,
            at("solution provided, but independent lessons must not have solutions"),
        ),
        (None, true) if !mc => diags.error(
            file,
            at(&format!("missing solution; {stage} lessons require one")),
        ),
        _ => {}
    }
    let needs_structured = ex.exercise_type.needs_structured_solution()
        || (!mc && matches!(stage, Stage::Practice | Stage::Transfer));
    match &ex.solution {
        Some(Solution::Text(t)) => {
            if needs_structured {
                diags.error(file, at("this exercise needs a structured worked solution (summary, analysis, guarantees, semantics, interpretation)"));
            }
            check_md(course, diags, file, &at("solution"), t);
        }
        Some(Solution::Structured(s)) => {
            check_md(course, diags, file, &at("solution summary"), &s.summary);
            check_md(
                course,
                diags,
                file,
                &at("solution guarantees"),
                &s.guarantees,
            );
            check_md(course, diags, file, &at("solution semantics"), &s.semantics);
            check_md(
                course,
                diags,
                file,
                &at("solution interpretation"),
                &s.interpretation,
            );
            if s.analysis.len() < 3 {
                diags.error(
                    file,
                    at("a structured solution needs at least three analysis aspects"),
                );
            }
            let mut seen = BTreeSet::new();
            for a in &s.analysis {
                if !seen.insert(a.aspect) {
                    diags.error(
                        file,
                        at(&format!(
                            "analysis aspect `{}` appears twice",
                            a.aspect.label()
                        )),
                    );
                }
                check_md(course, diags, file, &at("analysis"), &a.body);
            }
        }
        None => {}
    }

    if let Some(cl) = &ex.checklist {
        if !course.curriculum.checklists.contains_key(cl) {
            diags.error(file, at(&format!("unknown checklist `{cl}`")));
        }
    } else if stage == Stage::Independent || ex.exercise_type == ExerciseType::SelfAssessment {
        diags.error(
            file,
            at("independent and self-assessment exercises must reference a checklist"),
        );
    }
}

fn validate_glossary(course: &Course, diags: &mut Diagnostics) {
    let lessons: HashSet<&str> = all_lessons(course).map(|(_, _, l)| l.id.as_str()).collect();
    let mut seen = HashSet::new();
    for t in &course.glossary.terms {
        if !is_kebab(&t.id) {
            diags.error(
                GLOSSARY_FILE,
                format!("glossary id `{}` must be kebab-case", t.id),
            );
        }
        if !seen.insert(t.id.as_str()) {
            diags.error(GLOSSARY_FILE, format!("duplicate glossary id `{}`", t.id));
        }
        check_md(
            course,
            diags,
            GLOSSARY_FILE,
            &format!("glossary `{}`", t.id),
            &t.body,
        );
        for e in &t.examples {
            if !lessons.contains(e.as_str()) {
                diags.error(
                    GLOSSARY_FILE,
                    format!("glossary `{}` links to unknown lesson `{e}`", t.id),
                );
            }
        }
    }
}

/// A source range shown by a lesson, for the unseen-code check.
struct Shown<'a> {
    start: u32,
    end: u32,
    /// Position of the showing lesson in course order.
    pos: usize,
    lesson: &'a str,
    snippet: &'a str,
}

/// Transfer and independent lessons must show code no earlier lesson shows (spec §14.4).
fn validate_unseen_code(course: &Course, diags: &mut Diagnostics) {
    // source file -> ranges shown by lessons
    let mut shown: BTreeMap<&str, Vec<Shown>> = BTreeMap::new();
    for (pos, (_, _, l)) in all_lessons(course).enumerate() {
        let mut ids: Vec<&str> = lesson_snippets(l);
        ids.sort_unstable();
        ids.dedup();
        for sid in ids {
            if let Some((_, s)) = course.snippets.get(sid) {
                for seg in &s.segments {
                    shown
                        .entry(s.source.file.as_str())
                        .or_default()
                        .push(Shown {
                            start: seg.start_line,
                            end: seg.end_line,
                            pos,
                            lesson: l.id.as_str(),
                            snippet: sid,
                        });
                }
            }
        }
    }
    for (pos, (file, _, l)) in all_lessons(course).enumerate() {
        if !l.stage.requires_unseen_code() {
            continue;
        }
        for sid in lesson_snippets(l) {
            let Some((_, s)) = course.snippets.get(sid) else {
                continue;
            };
            let Some(ranges) = shown.get(s.source.file.as_str()) else {
                continue;
            };
            for seg in &s.segments {
                for o in ranges {
                    let (a, b) = (o.start, o.end);
                    // Only code shown *before* this lesson counts as seen; a later lesson that
                    // reuses this code is reported against that later lesson instead.
                    if o.pos < pos && seg.start_line <= b && a <= seg.end_line {
                        diags.error(
                            file,
                            format!(
                                "lesson `{}` ({}) must use unseen code, but snippet `{sid}` overlaps `{}` shown in lesson `{}` ({}:{a}-{b})",
                                l.id, l.stage, o.snippet, o.lesson, s.source.file
                            ),
                        );
                    }
                }
            }
        }
    }
}

/// Stages never go backwards, and the full progression exists (spec §9, §90).
fn validate_progression(course: &Course, diags: &mut Diagnostics) {
    let mut prev: Option<(Stage, &str)> = None;
    let mut present = BTreeSet::new();
    for (file, _, l) in all_lessons(course) {
        present.insert(l.stage);
        if let Some((p, pid)) = prev {
            if l.stage < p {
                diags.error(
                    file,
                    format!("lesson `{}` is `{}` but follows `{pid}` which is `{p}`; support may only decrease", l.id, l.stage),
                );
            }
        }
        prev = Some((l.stage, l.id.as_str()));
    }
    for s in Stage::ALL {
        if !present.contains(&s) {
            diags.error(
                CURRICULUM_FILE,
                format!("the course has no `{s}` lesson; all five stages are required"),
            );
        }
    }
    if let Some((s, id)) = prev {
        if s != Stage::Independent {
            diags.error(
                CURRICULUM_FILE,
                format!("the final lesson `{id}` must be an independent-reading lesson"),
            );
        }
    }
}
