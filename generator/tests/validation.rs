//! Integration tests for the course-source validator (spec §26, §64).
//!
//! Every test copies the valid fixture under `tests/fixtures/validation/` into a temporary
//! directory, applies one targeted mutation and asserts the specific diagnostic.

mod support;

use ono_course::diag::Diagnostics;
use ono_course::snippet;
use support::{Fixture, EVAL_RS, LIB_RS, PLACEHOLDER_COMMIT};

const LOCK: &str = "course-lock.yaml";
const CURRICULUM: &str = "course/curriculum.yaml";
const GLOSSARY: &str = "course/glossary.yaml";
const CH1: &str = "course/chapters/01-basics.yaml";
const CH2: &str = "course/chapters/02-deeper.yaml";
const GREET: &str = "course/snippets/ono-demo/greet.yaml";
const SUM: &str = "course/snippets/ono-demo/sum-until-negative.yaml";

#[track_caller]
fn assert_error(diags: &Diagnostics, needle: &str) {
    assert!(
        diags.has_error_containing(needle),
        "expected an error containing `{needle}`, got:\n{}",
        diags.report()
    );
}

/// Apply `mutate` to a fresh fixture copy and assert an error containing `needle`.
#[track_caller]
fn expect_error(mutate: impl FnOnce(&Fixture), needle: &str) -> Diagnostics {
    let f = Fixture::new();
    mutate(&f);
    let diags = f.validate();
    assert_error(&diags, needle);
    diags
}

// ---------------------------------------------------------------------------------------------
// The valid fixture

#[test]
fn valid_fixture_passes_without_diagnostics() {
    let (course, diags) = ono_course::load_and_validate(&support::fixture_root());
    assert!(
        diags.items.is_empty(),
        "unexpected diagnostics:\n{}",
        diags.report()
    );
    let course = course.expect("course loads");
    assert_eq!(course.chapters.len(), 2);
    assert_eq!(course.snippets.len(), 5);
    let stages: Vec<_> = ono_course::validate::all_lessons(&course)
        .map(|(_, _, l)| l.stage.slug())
        .collect();
    assert_eq!(
        stages,
        ["guided", "assisted", "practice", "transfer", "independent"]
    );
}

#[test]
fn valid_fixture_copy_passes() {
    let diags = Fixture::new().validate();
    assert!(!diags.has_errors(), "{}", diags.report());
}

#[test]
fn fixture_snippets_match_the_fake_ono_sources() {
    let (course, _) = ono_course::load_and_validate(&support::fixture_root());
    let course = course.unwrap();
    for (id, (_, s)) in &course.snippets {
        let text = std::fs::read_to_string(support::ono_source().join(&s.source.file)).unwrap();
        for seg in &s.segments {
            let code = snippet::extract(&text, seg.start_line, seg.end_line).unwrap();
            assert_eq!(code, seg.code, "snippet `{id}` differs from its source");
            assert_eq!(
                snippet::content_hash(&code),
                seg.content_hash,
                "snippet `{id}` hash"
            );
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Schema and parsing

#[test]
fn unknown_field_in_lesson_is_rejected() {
    expect_error(
        |f| {
            f.replace(
                CH1,
                "    stage: guided\n",
                "    stage: guided\n    colour: red\n",
            )
        },
        "unknown field `colour`",
    );
}

#[test]
fn unknown_field_in_exercise_is_rejected() {
    expect_error(
        |f| {
            f.replace(
                CH1,
                "        id: basics-01-q2\n",
                "        id: basics-01-q2\n        difficulty: 3\n",
            )
        },
        "difficulty",
    );
}

#[test]
fn unknown_field_in_snippet_and_lock_is_rejected() {
    expect_error(
        |f| f.replace(GREET, "source:\n", "source:\n  branch: main\n"),
        "unknown field `branch`",
    );
    expect_error(
        |f| f.replace(LOCK, "course:\n", "course:\n  name: x\n"),
        "unknown field `name`",
    );
}

#[test]
fn malformed_yaml_is_rejected() {
    let d = expect_error(
        |f| f.replace(CH2, "lessons:\n", "lessons: [\n"),
        "schema error",
    );
    assert!(d.errors().any(|e| e.file == CH2), "{}", d.report());
}

#[test]
fn invalid_stage_value_is_rejected() {
    expect_error(
        |f| f.replace(CH1, "stage: assisted", "stage: expert"),
        "unknown variant `expert`",
    );
}

// ---------------------------------------------------------------------------------------------
// Identifiers and references

#[test]
fn duplicate_lesson_ids_are_rejected() {
    expect_error(
        |f| f.replace(CH2, "  - id: deeper-02\n", "  - id: deeper-01\n"),
        "duplicate lesson id `deeper-01`",
    );
}

#[test]
fn duplicate_exercise_ids_are_rejected() {
    expect_error(
        |f| f.replace(CH1, "id: basics-02-q1", "id: basics-01-q1"),
        "duplicate exercise id `basics-01-q1`",
    );
    // Also across chapter files.
    expect_error(
        |f| f.replace(CH2, "id: deeper-01-q1", "id: basics-01-q2"),
        "duplicate exercise id `basics-01-q2`",
    );
}

#[test]
fn duplicate_snippet_ids_are_rejected() {
    expect_error(
        |f| {
            let text = f.read(GREET);
            f.write("course/snippets/other/greet.yaml", &text);
        },
        "duplicate snippet id `greet`",
    );
}

#[test]
fn unknown_snippet_in_code_section_is_rejected() {
    expect_error(
        |f| f.replace(CH1, "snippet: count-words", "snippet: no-such-snippet"),
        "unknown snippet `no-such-snippet`",
    );
}

#[test]
fn unknown_snippet_in_exercise_is_rejected() {
    expect_error(
        |f| f.replace(CH2, "snippets: [eval-word]", "snippets: [no-such-snippet]"),
        "exercise `deeper-03-q1` refers to unknown snippet `no-such-snippet`",
    );
}

#[test]
fn missing_prerequisite_is_rejected() {
    expect_error(
        |f| {
            f.replace(
                CH1,
                "prerequisites: [basics-01]",
                "prerequisites: [no-such-lesson]",
            )
        },
        "unknown prerequisite lesson `no-such-lesson`",
    );
}

#[test]
fn forward_prerequisite_is_rejected() {
    expect_error(
        |f| f.replace(CH1, "prerequisites: []", "prerequisites: [deeper-01]"),
        "prerequisite `deeper-01` must come earlier",
    );
}

#[test]
fn cyclic_prerequisite_is_rejected() {
    // A lesson requiring itself, and a two-lesson cycle.
    expect_error(
        |f| {
            f.replace(
                CH1,
                "prerequisites: [basics-01]",
                "prerequisites: [basics-02]",
            )
        },
        "prerequisite `basics-02` must come earlier",
    );
    let d = expect_error(
        |f| f.replace(CH1, "prerequisites: []", "prerequisites: [basics-02]"),
        "cyclic or forward prerequisite",
    );
    assert!(
        d.has_error_containing("lesson `basics-01`"),
        "{}",
        d.report()
    );
}

#[test]
fn unknown_concept_tag_is_rejected() {
    expect_error(
        |f| f.replace(CH2, "concepts: [control-flow]", "concepts: [lifetimes]"),
        "unknown concept tag `lifetimes`",
    );
}

#[test]
fn unknown_ono_topic_is_rejected() {
    expect_error(
        |f| f.replace(CH2, "ono_topics: [evaluation]", "ono_topics: [networking]"),
        "unknown ono_topics tag `networking`",
    );
}

#[test]
fn unknown_checklist_is_rejected() {
    expect_error(
        |f| {
            f.replace(
                CH2,
                "checklist: independent-reading",
                "checklist: no-such-checklist",
            )
        },
        "unknown checklist `no-such-checklist`",
    );
}

#[test]
fn glossary_example_to_unknown_lesson_is_rejected() {
    expect_error(
        |f| {
            f.replace(
                GLOSSARY,
                "examples: [deeper-02]",
                "examples: [no-such-lesson]",
            )
        },
        "glossary `variant` links to unknown lesson `no-such-lesson`",
    );
}

#[test]
fn empty_objectives_are_rejected() {
    expect_error(
        |f| {
            f.replace(
                CH1,
                "    objectives:\n      - Trace a value through an iterator chain.\n",
                "    objectives: []\n",
            )
        },
        "lesson `basics-02`: needs at least one objective",
    );
}

// ---------------------------------------------------------------------------------------------
// Stage progression

#[test]
fn stage_going_backwards_is_rejected() {
    expect_error(
        |f| f.replace(CH1, "stage: assisted", "stage: transfer"),
        "lesson `deeper-01` is `practice` but follows `basics-02` which is `transfer`",
    );
}

#[test]
fn missing_stage_is_rejected() {
    expect_error(
        |f| f.replace(CH2, "stage: practice", "stage: assisted"),
        "the course has no `practice` lesson",
    );
}

#[test]
fn final_lesson_must_be_independent() {
    expect_error(
        |f| {
            // Append a transfer lesson after the independent one.
            let extra = "  - id: deeper-04\n    title: After the end\n    stage: transfer\n    summary: x\n    concepts: [enums]\n    ono_topics: [evaluation]\n    objectives: [x]\n    sections:\n      - type: code\n        snippet: eval-word\n";
            let text = f.read(CH2);
            f.write(CH2, &format!("{text}{extra}"));
        },
        "the final lesson `deeper-04` must be an independent-reading lesson",
    );
}

// ---------------------------------------------------------------------------------------------
// Solutions and hints

#[test]
fn solution_in_independent_lesson_is_rejected() {
    expect_error(
        |f| f.replace(CH2, "        checklist: independent-reading\n", "        checklist: independent-reading\n        solution: It evaluates.\n"),
        "exercise `deeper-03-q1`: solution provided, but independent lessons must not have solutions",
    );
}

#[test]
fn missing_solution_in_guided_lesson_is_rejected() {
    expect_error(
        |f| {
            f.replace(
            CH1,
            "        solution: |\n          It builds a new `String` with `format!` and returns it as the last expression.\n",
            "",
        )
        },
        "exercise `basics-01-q2`: missing solution; guided lessons require one",
    );
}

#[test]
fn missing_solution_in_assisted_lesson_is_rejected() {
    expect_error(
        |f| {
            f.replace(
            CH1,
            "        solution: |\n          It returns `3`: `split_whitespace` skips runs of whitespace.\n",
            "",
        )
        },
        "exercise `basics-02-q1`: missing solution; assisted lessons require one",
    );
}

#[test]
fn missing_solution_in_practice_lesson_is_rejected() {
    expect_error(
        |f| {
            let text = f.read(CH2);
            let start = text
                .find("        solution:\n          summary: It sums")
                .unwrap();
            let end = text.find("  - id: deeper-02").unwrap();
            f.write(CH2, &format!("{}{}", &text[..start], &text[end..]));
        },
        "exercise `deeper-01-q1`: missing solution; practice lessons require one",
    );
}

#[test]
fn unstructured_solution_for_reading_exercise_is_rejected() {
    expect_error(
        |f| {
            f.replace(
                CH1,
                "exercise_type: explain-line",
                "exercise_type: read-function",
            )
        },
        "exercise `basics-01-q2`: this exercise needs a structured worked solution",
    );
}

#[test]
fn unstructured_solution_in_practice_lesson_is_rejected() {
    expect_error(
        |f| {
            let text = f.read(CH2);
            let start = text
                .find("        solution:\n          summary: It sums")
                .unwrap();
            let end = text.find("  - id: deeper-02").unwrap();
            let repl = "        exercise_type: trace-value\n        solution: It sums a prefix.\n";
            let t = format!("{}{}{}", &text[..start], repl, &text[end..]);
            f.write(
                CH2,
                &t.replacen("        exercise_type: read-function\n", "", 1),
            );
        },
        "exercise `deeper-01-q1`: this exercise needs a structured worked solution",
    );
}

#[test]
fn hints_in_independent_lesson_are_rejected() {
    expect_error(
        |f| {
            f.replace(
            CH2,
            "        checklist: independent-reading\n",
            "        checklist: independent-reading\n        hints:\n          - level: 1\n            body: Look at the match.\n",
        )
        },
        "independent lessons allow at most 0 hint level(s), found 1",
    );
}

#[test]
fn too_many_hints_for_transfer_are_rejected() {
    expect_error(
        |f| {
            f.replace(
            CH2,
            "            body: What does `str::parse` return?\n",
            "            body: What does `str::parse` return?\n          - level: 2\n            body: Two arms.\n",
        )
        },
        "transfer lessons allow at most 1 hint level(s), found 2",
    );
}

#[test]
fn hint_level_two_before_one_is_rejected() {
    expect_error(
        |f| {
            f.replace(
                CH2,
                "          - level: 1\n            body: Where can",
                "          - level: 9\n            body: Where can",
            );
            f.replace(CH2, "          - level: 2\n", "          - level: 1\n");
            f.replace(CH2, "          - level: 9\n", "          - level: 2\n");
        },
        "invalid hint ordering: hint 1 has level 2, expected 1",
    );
}

#[test]
fn hint_level_gap_is_rejected() {
    expect_error(
        |f| f.replace(CH1, "          - level: 2\n", "          - level: 3\n"),
        "invalid hint ordering: hint 2 has level 3, expected 2",
    );
}

// ---------------------------------------------------------------------------------------------
// Exercise types

#[test]
fn multiple_choice_with_no_correct_choice_is_rejected() {
    expect_error(
        |f| f.replace(CH1, "correct: true", "correct: false"),
        "multiple choice needs exactly one correct choice, found 0",
    );
}

#[test]
fn multiple_choice_with_two_correct_choices_is_rejected() {
    expect_error(
        |f| f.replace(CH1, "correct: false", "correct: true"),
        "multiple choice needs exactly one correct choice, found 2",
    );
}

#[test]
fn multiple_choice_in_independent_lesson_is_rejected() {
    expect_error(
        |f| {
            f.replace(
            CH2,
            "        exercise_type: read-function\n        prompt: Read `eval`",
            "        exercise_type: multiple-choice\n        choices:\n          - text: A\n            correct: true\n            feedback: yes\n          - text: B\n            feedback: no\n        prompt: Read `eval`",
        )
        },
        "multiple choice reveals an answer and is not allowed in independent lessons",
    );
}

#[test]
fn choices_on_non_multiple_choice_exercise_are_rejected() {
    expect_error(
        |f| {
            f.replace(
                CH1,
                "exercise_type: multiple-choice",
                "exercise_type: predict",
            )
        },
        "exercise `basics-01-q1`: only multiple-choice exercises may have choices",
    );
}

#[test]
fn independent_exercise_without_checklist_is_rejected() {
    expect_error(
        |f| f.replace(CH2, "        checklist: independent-reading\n", ""),
        "exercise `deeper-03-q1`: independent and self-assessment exercises must reference a checklist",
    );
}

// ---------------------------------------------------------------------------------------------
// Markdown prose

#[test]
fn broken_internal_links_are_rejected() {
    for (from, to, needle) in [
        (
            "(lesson:basics-02)",
            "(lesson:nope)",
            "broken internal link `lesson:nope`",
        ),
        (
            "(glossary:borrow)",
            "(glossary:nope)",
            "broken internal link `glossary:nope`",
        ),
        (
            "(concept:borrowing)",
            "(concept:nope)",
            "broken internal link `concept:nope`",
        ),
        (
            "(topic:text-handling)",
            "(topic:nope)",
            "broken internal link `topic:nope`",
        ),
        (
            "(chapter:deeper)",
            "(chapter:nope)",
            "broken internal link `chapter:nope`",
        ),
    ] {
        expect_error(|f| f.replace(CH1, from, to), needle);
    }
    // Links in the glossary are checked too.
    expect_error(
        |f| f.replace(GLOSSARY, "(concept:enums)", "(concept:traits)"),
        "broken internal link `concept:traits`",
    );
}

#[test]
fn external_links_in_prose_are_rejected() {
    expect_error(
        |f| {
            f.replace(
                CH1,
                "(concept:borrowing)",
                "(https://doc.rust-lang.org/book/)",
            )
        },
        "not allowed",
    );
    expect_error(
        |f| {
            f.replace(
                CH1,
                "Two short functions,",
                "See <https://example.com>. Two short functions,",
            )
        },
        "external links are not allowed",
    );
    expect_error(
        |f| f.replace(CH1, "(lesson:basics-02)", "(basics-02.html)"),
        "is not a course reference",
    );
}

#[test]
fn raw_html_in_prose_is_rejected() {
    expect_error(
        |f| {
            f.replace(
                CH1,
                "This lesson reads `greet`.",
                "This lesson reads <b>greet</b>.",
            )
        },
        "raw HTML is not allowed",
    );
}

#[test]
fn heading_in_prose_is_rejected() {
    expect_error(
        |f| {
            f.replace(
                CH1,
                "          This lesson reads",
                "          # A heading\n\n          This lesson reads",
            )
        },
        "headings are not allowed in prose",
    );
}

// ---------------------------------------------------------------------------------------------
// Code sections

#[test]
fn annotation_lines_outside_snippet_are_rejected() {
    expect_error(
        |f| f.replace(CH1, "          - lines: 5-6\n", "          - lines: 5-7\n"),
        "annotation lines 5–7 are outside snippet `greet`",
    );
}

#[test]
fn annotation_token_not_on_line_is_rejected() {
    expect_error(
        |f| f.replace(CH1, "token: \"&str\"", "token: \"&mut\""),
        "annotation token `&mut` does not occur on line 4 of snippet `greet`",
    );
}

#[test]
fn highlight_outside_snippet_is_rejected() {
    expect_error(
        |f| f.replace(CH1, "highlight: [4]", "highlight: [9]"),
        "highlight 9 is outside snippet `greet`",
    );
}

#[test]
fn highlight_in_gap_of_shortened_snippet_is_rejected() {
    expect_error(
        |f| f.replace(CH2, "highlight: [9-10]", "highlight: [6-8]"),
        "highlight 6–8 is outside snippet `parse-word`",
    );
}

#[test]
fn guided_lesson_without_annotations_is_rejected() {
    expect_error(
        |f| {
            let text = f.read(CH1);
            let start = text.find("        annotations:\n").unwrap();
            let end = text.find("      - type: diagram").unwrap();
            f.write(CH1, &format!("{}{}", &text[..start], &text[end..]));
        },
        "lesson `basics-01`: guided lessons must annotate their code",
    );
}

#[test]
fn lesson_without_code_is_rejected() {
    expect_error(
        |f| {
            f.replace(
                CH1,
                "      - type: code\n        snippet: count-words\n",
                "",
            )
        },
        "lesson `basics-02`: shows no Ono-Sendai code",
    );
}

// ---------------------------------------------------------------------------------------------
// Snippet integrity

#[test]
fn edited_snippet_code_is_a_hash_mismatch() {
    expect_error(
        |f| {
            f.replace(
                GREET,
                "format!(\"hello, {name}\")",
                "format!(\"hi, {name}\")",
            )
        },
        "snippet `greet`: content hash mismatch for 3-6",
    );
}

#[test]
fn segment_line_count_mismatch_is_rejected() {
    expect_error(
        |f| f.replace(GREET, "    end_line: 6\n", "    end_line: 7\n"),
        "snippet `greet`: segment 3-7 must contain exactly 5 lines, found 4",
    );
    expect_error(
        |f| f.replace(SUM, "          total\n", ""),
        "segment 13-23 must contain exactly 11 lines, found 10",
    );
}

#[test]
fn segment_without_trailing_newline_is_rejected() {
    expect_error(
        |f| f.replace(GREET, "    code: |\n", "    code: |-\n"),
        "must contain exactly 4 lines",
    );
}

#[test]
fn anchor_must_be_unique() {
    expect_error(
        |f| {
            f.replace(
                SUM,
                "anchor: \"pub fn sum_until_negative\"",
                "anchor: \"total\"",
            )
        },
        "anchor `total` must occur exactly once in 13-23, found 3",
    );
}

#[test]
fn anchor_must_exist() {
    expect_error(
        |f| {
            f.replace(
                SUM,
                "anchor: \"pub fn sum_until_negative\"",
                "anchor: \"fn product\"",
            )
        },
        "anchor `fn product` must occur exactly once in 13-23, found 0",
    );
}

#[test]
fn snippet_commit_must_match_pinned_commit() {
    let other = "b".repeat(40);
    expect_error(
        |f| f.replace(GREET, PLACEHOLDER_COMMIT, &other),
        &format!("snippet `greet` was taken from commit {other} but course-lock.yaml pins {PLACEHOLDER_COMMIT}"),
    );
}

#[test]
fn snippet_file_name_must_match_id() {
    expect_error(
        |f| f.replace(GREET, "id: greet\n", "id: greeting\n"),
        "must match its file name `greet.yaml`",
    );
}

#[test]
fn invalid_snippet_source_path_is_rejected() {
    expect_error(
        |f| {
            f.replace(
                GREET,
                "file: crates/ono-demo/src/lib.rs",
                "file: ../ono-demo/src/lib.rs",
            )
        },
        "invalid source path",
    );
}

#[test]
fn inverted_segment_range_is_rejected() {
    expect_error(
        |f| f.replace(GREET, "- start_line: 3\n", "- start_line: 9\n"),
        "invalid source range 9-6",
    );
}

// ---------------------------------------------------------------------------------------------
// Lock file

#[test]
fn short_lock_commit_is_rejected() {
    expect_error(
        |f| {
            f.replace(
                LOCK,
                &format!("commit: {PLACEHOLDER_COMMIT}"),
                "commit: a1b2c3d",
            )
        },
        "ono_sendai.commit `a1b2c3d` must be a full 40-character lowercase commit SHA",
    );
}

#[test]
fn uppercase_lock_commit_is_rejected() {
    let upper = PLACEHOLDER_COMMIT.to_uppercase();
    expect_error(
        |f| f.replace(LOCK, PLACEHOLDER_COMMIT, &upper),
        "must be a full 40-character lowercase commit SHA",
    );
}

#[test]
fn invalid_course_version_is_rejected() {
    expect_error(
        |f| f.replace(LOCK, "version: 1.0.0", "version: \"1.0\""),
        "course.version `1.0` is not semver",
    );
    expect_error(
        |f| f.replace(LOCK, "version: 1.0.0", "version: v1.0.0"),
        "course.version `v1.0.0` is not semver",
    );
}

// ---------------------------------------------------------------------------------------------
// Chapters and curriculum

#[test]
fn chapter_number_mismatch_is_rejected() {
    expect_error(
        |f| f.replace(CH2, "number: 2", "number: 3"),
        "chapter `deeper` has number 3 but is listed at position 2",
    );
}

#[test]
fn unlisted_chapter_file_is_rejected() {
    let d = expect_error(
        |f| f.write("course/chapters/03-extra.yaml", "id: extra\n"),
        "chapter file is not listed in course/curriculum.yaml",
    );
    assert!(
        d.errors()
            .any(|e| e.file == "course/chapters/03-extra.yaml"),
        "{}",
        d.report()
    );
}

#[test]
fn missing_chapter_file_is_rejected() {
    expect_error(
        |f| {
            f.replace(
                CURRICULUM,
                "  - chapters/02-deeper.yaml\n",
                "  - chapters/02-deeper.yaml\n  - chapters/03-missing.yaml\n",
            )
        },
        "chapter file `chapters/03-missing.yaml` does not exist",
    );
    expect_error(
        |f| f.remove(CH2),
        "chapter file `chapters/02-deeper.yaml` does not exist",
    );
}

#[test]
fn duplicate_concept_ids_are_rejected() {
    expect_error(
        |f| f.replace(CURRICULUM, "  - id: enums\n", "  - id: borrowing\n"),
        "duplicate concept id `borrowing`",
    );
}

// ---------------------------------------------------------------------------------------------
// Unseen code

#[test]
fn transfer_lesson_reusing_earlier_snippet_is_rejected() {
    expect_error(
        |f| f.replace(CH2, "snippets: [parse-word]", "snippets: [sum-until-negative]"),
        "lesson `deeper-02` (transfer) must use unseen code, but snippet `sum-until-negative` overlaps `sum-until-negative` shown in lesson `deeper-01`",
    );
}

/// Write a snippet file for `file` lines `start..=end`, extracted from the fake Ono tree.
fn add_snippet(f: &Fixture, id: &str, file: &str, start: u32, end: u32, anchor: &str) {
    let text = std::fs::read_to_string(support::ono_source().join(file)).unwrap();
    let s = snippet::build_snippet(
        id,
        file,
        PLACEHOLDER_COMMIT,
        &text,
        &[(start, end, anchor.to_string())],
    )
    .unwrap();
    f.write(
        &format!("course/snippets/ono-demo/{id}.yaml"),
        &serde_yaml::to_string(&s).unwrap(),
    );
}

#[test]
fn transfer_snippet_partially_overlapping_earlier_range_is_rejected() {
    let d = expect_error(
        |f| {
            // lib.rs 14-16 overlaps `sum-until-negative` (13-23), a different snippet.
            add_snippet(f, "sum-head", LIB_RS, 14, 16, "let mut total");
            f.replace(CH2, "snippets: [parse-word]", "snippets: [sum-head]");
        },
        "snippet `sum-head` overlaps `sum-until-negative` shown in lesson `deeper-01` (crates/ono-demo/src/lib.rs:13-23)",
    );
    assert!(
        !d.has_error_containing("lesson `deeper-01` (practice)"),
        "{}",
        d.report()
    );
}

#[test]
fn independent_snippet_overlapping_transfer_snippet_is_rejected_only_for_the_later_lesson() {
    let d = expect_error(
        |f| {
            // eval.rs 9-12 lies inside `parse-word`, shown by the earlier transfer lesson.
            add_snippet(f, "parse-body", EVAL_RS, 9, 12, "match word");
            f.replace(CH2, "snippets: [eval-word]", "snippets: [eval-word, parse-body]");
        },
        "lesson `deeper-03` (independent) must use unseen code, but snippet `parse-body` overlaps `parse-word` shown in lesson `deeper-02`",
    );
    // The transfer lesson came first: its code was unseen when the learner read it.
    assert!(
        !d.has_error_containing("lesson `deeper-02` (transfer) must use unseen code"),
        "the earlier lesson must not be blamed:\n{}",
        d.report()
    );
}

#[test]
fn non_overlapping_snippet_from_same_file_is_unseen() {
    let f = Fixture::new();
    // lib.rs line 1 is shown by no lesson.
    add_snippet(&f, "crate-doc", LIB_RS, 1, 1, "demo crate");
    f.replace(
        CH2,
        "snippets: [eval-word]",
        "snippets: [eval-word, crate-doc]",
    );
    let d = f.validate();
    assert!(!d.has_errors(), "{}", d.report());
}

// ---------------------------------------------------------------------------------------------
// Missing content, duplicate identifiers and unsupported constructs

#[test]
fn chapter_without_lessons_is_rejected() {
    expect_error(
        |f| {
            f.write(
                CH2,
                "id: deeper\nnumber: 2\ntitle: Deeper\nsummary: Less help.\nlessons: []\n",
            )
        },
        "chapter `deeper` has no lessons",
    );
}

#[test]
fn lesson_without_sections_is_rejected() {
    expect_error(
        |f| {
            let text = f.read(CH1);
            let cut = text.find("  - id: basics-02\n").unwrap();
            f.write(
                CH1,
                &format!(
                    "{}  - id: basics-02\n    title: Counting words\n    stage: assisted\n    summary: x\n    prerequisites: [basics-01]\n    concepts: [borrowing]\n    ono_topics: [text-handling]\n    objectives: [x]\n    sections: []\n",
                    &text[..cut]
                ),
            );
        },
        "lesson `basics-02`: has no sections",
    );
}

#[test]
fn empty_curriculum_is_rejected() {
    let d = expect_error(
        |f| {
            f.replace(
                CURRICULUM,
                "chapters:\n  - chapters/01-basics.yaml\n  - chapters/02-deeper.yaml\n",
                "chapters: []\n",
            )
        },
        "the curriculum lists no chapters",
    );
    // The chapter files still on disk are reported, not silently dropped.
    assert_error(&d, "chapter file is not listed in course/curriculum.yaml");
}

#[test]
fn duplicate_chapter_ids_are_rejected() {
    expect_error(
        |f| f.replace(CH2, "id: deeper\n", "id: basics\n"),
        "duplicate chapter id `basics`",
    );
}

#[test]
fn duplicate_glossary_ids_are_rejected() {
    expect_error(
        |f| f.replace(GLOSSARY, "  - id: variant\n", "  - id: borrow\n"),
        "duplicate glossary id `borrow`",
    );
}

#[test]
fn duplicate_ono_topic_ids_are_rejected() {
    expect_error(
        |f| {
            f.replace(
                CURRICULUM,
                "  - id: evaluation\n",
                "  - id: text-handling\n",
            )
        },
        "duplicate ono_topic id `text-handling`",
    );
}

#[test]
fn non_kebab_case_ids_are_rejected() {
    expect_error(
        |f| f.replace_all(CH1, "basics-01-q1", "Basics_01_Q1"),
        "exercise `Basics_01_Q1`: id must be kebab-case",
    );
    expect_error(
        |f| f.replace(CURRICULUM, "  - id: enums\n", "  - id: Enums\n"),
        "concept id `Enums` must be kebab-case",
    );
}

#[test]
fn unknown_section_type_is_rejected() {
    expect_error(
        |f| {
            f.replace(
                CH1,
                "      - type: diagram\n        kind: ownership\n",
                "      - type: video\n        kind: ownership\n",
            )
        },
        "unknown variant `video`",
    );
}

#[test]
fn unknown_exercise_type_and_prose_kind_are_rejected() {
    expect_error(
        |f| f.replace(CH1, "exercise_type: explain-line", "exercise_type: essay"),
        "unknown variant `essay`",
    );
    expect_error(
        |f| f.replace(CH1, "        kind: context\n", "        kind: sidebar\n"),
        "unknown variant `sidebar`",
    );
}

#[test]
fn images_in_prose_are_rejected() {
    expect_error(
        |f| {
            f.replace(
                CH1,
                "          This lesson reads `greet`.",
                "          ![diagram](diagram.png) This lesson reads `greet`.",
            )
        },
        "images are not supported in prose",
    );
}

#[test]
fn empty_prose_is_rejected() {
    expect_error(
        |f| {
            f.replace(
                CH1,
                "        caption: The whole function.\n",
                "        caption: \"  \"\n",
            )
        },
        "caption: text must not be empty",
    );
}

#[test]
fn diagram_without_art_or_text_equivalent_is_rejected() {
    expect_error(
        |f| {
            f.replace(
                CH1,
                "        art: |\n          caller ──lends──► greet\n",
                "        art: \"\"\n",
            )
        },
        "diagram needs a title and art",
    );
    expect_error(
        |f| {
            f.replace(
                CH1,
                "        description: The caller keeps ownership; `greet` only borrows the name.\n",
                "        description: \"\"\n",
            )
        },
        "diagram description (text equivalent): text must not be empty",
    );
}

// ---------------------------------------------------------------------------------------------
// Malformed exercises

#[test]
fn multiple_choice_with_one_choice_is_rejected() {
    expect_error(
        |f| {
            f.replace(
                CH1,
                "          - text: The function `greet`.\n            correct: false\n            feedback: A `&str` parameter does not take ownership.\n",
                "",
            )
        },
        "multiple choice needs 2 to 6 choices",
    );
}

#[test]
fn structured_solution_needs_three_distinct_aspects() {
    expect_error(
        |f| {
            f.replace(
                CH2,
                "            - aspect: control-flow\n              body: The `break` ends the loop at the first negative value.\n",
                "",
            )
        },
        "a structured solution needs at least three analysis aspects",
    );
    expect_error(
        |f| f.replace(CH2, "aspect: control-flow", "aspect: purpose"),
        "analysis aspect `Purpose` appears twice",
    );
}

#[test]
fn structured_solution_must_separate_guarantees_semantics_and_interpretation() {
    expect_error(
        |f| {
            f.replace(
                CH2,
                "          semantics: The slice is borrowed, so the caller keeps ownership.\n",
                "",
            )
        },
        "missing field `semantics`",
    );
}

#[test]
fn self_assessment_without_checklist_is_rejected_in_any_stage() {
    expect_error(
        |f| f.replace(CH1, "exercise_type: explain-line", "exercise_type: self-assessment"),
        "exercise `basics-01-q2`: independent and self-assessment exercises must reference a checklist",
    );
}

#[test]
fn independent_lesson_without_exercise_is_rejected() {
    let d = expect_error(
        |f| {
            f.replace(
                CH2,
                "      - type: exercise\n        id: deeper-03-q1\n        exercise_type: read-function\n        prompt: Read `eval` and write down what it does.\n        snippets: [eval-word]\n        checklist: independent-reading\n",
                "      - type: code\n        snippet: eval-word\n",
            )
        },
        "lesson `deeper-03`: independent lessons must contain a reading exercise",
    );
    assert!(!d.has_error_containing("solution"), "{}", d.report());
}

// ---------------------------------------------------------------------------------------------
// Snippet structure and the lock file

#[test]
fn snippet_without_segments_is_rejected() {
    expect_error(
        |f| {
            let text = f.read(GREET);
            let cut = text.find("segments:").unwrap();
            f.write(GREET, &format!("{}segments: []\n", &text[..cut]));
        },
        "snippet `greet` has no segments",
    );
}

#[test]
fn overlapping_segments_are_rejected() {
    let f = Fixture::new();
    let text = std::fs::read_to_string(support::ono_source().join(LIB_RS)).unwrap();
    let s = snippet::build_snippet(
        "greet",
        LIB_RS,
        PLACEHOLDER_COMMIT,
        &text,
        &[(3, 6, "pub fn greet".into())],
    )
    .unwrap();
    let mut s2 = s.clone();
    s2.segments.push(s.segments[0].clone());
    f.write(GREET, &serde_yaml::to_string(&s2).unwrap());
    assert_error(
        &f.validate(),
        "segments must be ascending and non-overlapping",
    );
}

#[test]
fn empty_repository_in_lock_is_rejected() {
    expect_error(
        |f| {
            let text = f.read(LOCK);
            let line = text
                .lines()
                .find(|l| l.trim_start().starts_with("repository:"))
                .unwrap()
                .to_string();
            f.replace(LOCK, &line, "  repository: \"\"");
        },
        "ono_sendai.repository must not be empty",
    );
}

#[test]
fn checklist_without_items_is_rejected() {
    expect_error(
        |f| {
            f.replace(
                CURRICULUM,
                "    items:\n      - I can summarize the purpose of this code.\n      - I can name its inputs and outputs.\n",
                "    items: []\n",
            )
        },
        "checklist `independent-reading` has no items",
    );
}

#[test]
fn unused_snippet_is_a_warning_not_an_error() {
    let f = Fixture::new();
    add_snippet(&f, "crate-doc", LIB_RS, 1, 1, "demo crate");
    let d = f.validate();
    assert!(!d.has_errors(), "{}", d.report());
    assert!(
        d.items.iter().any(|i| i
            .message
            .contains("snippet `crate-doc` is not used by any lesson")),
        "{}",
        d.report()
    );
}

#[test]
fn links_in_plain_text_titles_and_summaries_are_rejected() {
    expect_error(
        |f| {
            f.replace(
                CH1,
                "summary: A function that borrows a string.",
                "summary: A function that [borrows](concept:borrowing) a string.",
            )
        },
        "lesson `basics-01`: summary: links are not supported in plain-text fields",
    );
    expect_error(
        |f| {
            f.replace(
                CH2,
                "title: Deeper\n",
                "title: \"[Deeper](chapter:basics)\"\n",
            )
        },
        "chapter `deeper` title: links are not supported in plain-text fields",
    );
}
