//! Snippet identity: exact text, content hashes, extraction from a checkout and relocation.
//!
//! Line numbers are orientation, not identity (spec §19). A segment is identified by the exact
//! text of its lines, hashed with SHA-256. When the recorded line range no longer holds that text,
//! the text is searched for in the whole file: found once means "moved but identical", found more
//! than once means "no longer uniquely identifiable", not found means "changed".

use crate::model::{Segment, Snippet, SnippetSource};
use sha2::{Digest, Sha256};

pub const HASH_PREFIX: &str = "sha256:";

/// Split source text into lines without their terminators. A trailing newline does not create an
/// extra empty line, and `\r` is kept so CRLF sources hash exactly as they are.
pub fn source_lines(text: &str) -> Vec<&str> {
    let mut lines: Vec<&str> = text.split('\n').collect();
    if text.ends_with('\n') {
        lines.pop();
    }
    lines
}

/// The canonical segment text: each line followed by `\n`.
pub fn segment_text(lines: &[&str]) -> String {
    let mut out = String::new();
    for l in lines {
        out.push_str(l);
        out.push('\n');
    }
    out
}

pub fn content_hash(code: &str) -> String {
    let digest = Sha256::digest(code.as_bytes());
    format!("{HASH_PREFIX}{}", hex(&digest))
}

pub fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(DIGITS[(b >> 4) as usize] as char);
        s.push(DIGITS[(b & 0xf) as usize] as char);
    }
    s
}

/// Extract lines `start..=end` (1-based) of `text` as canonical segment text.
pub fn extract(text: &str, start: u32, end: u32) -> Result<String, String> {
    let lines = source_lines(text);
    if start == 0 || end < start {
        return Err(format!("invalid line range {start}-{end}"));
    }
    if end as usize > lines.len() {
        return Err(format!(
            "line range {start}-{end} exceeds the file's {} lines",
            lines.len()
        ));
    }
    Ok(segment_text(&lines[start as usize - 1..end as usize]))
}

/// Count non-overlapping-start occurrences of `needle` in `hay` (overlapping allowed).
pub fn count_occurrences(hay: &str, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }
    let mut count = 0;
    let mut from = 0;
    while let Some(pos) = hay[from..].find(needle) {
        count += 1;
        from += pos + needle.chars().next().map_or(1, char::len_utf8);
    }
    count
}

/// Build a snippet from a checkout file's text.
pub fn build_snippet(
    id: &str,
    file: &str,
    commit: &str,
    text: &str,
    ranges: &[(u32, u32, String)],
) -> Result<Snippet, String> {
    if ranges.is_empty() {
        return Err("at least one line range is required".into());
    }
    let mut segments = Vec::new();
    let mut last_end = 0;
    for (start, end, anchor) in ranges {
        if *start <= last_end {
            return Err(format!(
                "line ranges must be ascending and non-overlapping ({start}-{end})"
            ));
        }
        let code = extract(text, *start, *end)?;
        let n = count_occurrences(&code, anchor);
        if n != 1 {
            return Err(format!(
                "anchor `{anchor}` must occur exactly once in lines {start}-{end}, found {n}"
            ));
        }
        segments.push(Segment {
            start_line: *start,
            end_line: *end,
            anchor: anchor.clone(),
            content_hash: content_hash(&code),
            code,
        });
        last_end = *end;
    }
    Ok(Snippet {
        id: id.to_string(),
        source: SnippetSource {
            file: file.to_string(),
            commit: commit.to_string(),
        },
        segments,
    })
}

/// Where a segment's recorded text is found in a (possibly newer) version of its file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Location {
    /// The recorded line range still holds exactly the recorded text.
    Exact,
    /// The text exists exactly once, at another start line.
    Moved { start_line: u32 },
    /// The text exists more than once and the recorded range no longer holds it.
    Ambiguous { occurrences: usize },
    /// The text does not occur in the file.
    Changed,
    /// The recorded range lies outside the file and the text is not found.
    OutOfRange { file_lines: usize },
}

/// Locate a segment's recorded text in `file_text`.
pub fn locate(segment: &Segment, file_text: &str) -> Location {
    let lines = source_lines(file_text);
    let want: Vec<&str> = source_lines(&segment.code);
    let in_range = segment.end_line as usize <= lines.len() && segment.start_line >= 1;
    if in_range {
        let got = &lines[segment.start_line as usize - 1..segment.end_line as usize];
        if got == want.as_slice() {
            return Location::Exact;
        }
    }
    let starts = find_line_sequence(&lines, &want);
    match starts.len() {
        0 if !in_range => Location::OutOfRange {
            file_lines: lines.len(),
        },
        0 => Location::Changed,
        1 => Location::Moved {
            start_line: starts[0] as u32 + 1,
        },
        n => Location::Ambiguous { occurrences: n },
    }
}

/// All 0-based start indices where `needle` occurs as a contiguous run of whole lines.
pub fn find_line_sequence(lines: &[&str], needle: &[&str]) -> Vec<usize> {
    if needle.is_empty() || needle.len() > lines.len() {
        return Vec::new();
    }
    (0..=lines.len() - needle.len())
        .filter(|&i| lines[i..i + needle.len()] == *needle)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE: &str = "fn a() {}\nfn b() {\n    1\n}\nfn c() {}\n";

    #[test]
    fn lines_ignore_trailing_newline() {
        assert_eq!(source_lines(FILE).len(), 5);
        assert_eq!(source_lines("x").len(), 1);
        assert_eq!(source_lines("x\n\n").len(), 2);
    }

    #[test]
    fn hash_is_stable_sha256() {
        assert_eq!(
            content_hash(""),
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(content_hash("abc\n"), content_hash("abc\n"));
        assert_ne!(content_hash("abc\n"), content_hash("abc \n"));
    }

    #[test]
    fn extract_is_exact_and_bounded() {
        assert_eq!(extract(FILE, 2, 4).unwrap(), "fn b() {\n    1\n}\n");
        assert!(extract(FILE, 4, 9).is_err());
        assert!(extract(FILE, 0, 1).is_err());
        assert!(extract(FILE, 3, 2).is_err());
    }

    #[test]
    fn build_requires_unique_anchor_and_ordered_ranges() {
        let s = build_snippet("x", "f.rs", "c", FILE, &[(2, 4, "fn b".into())]).unwrap();
        assert_eq!(
            s.segments[0].content_hash,
            content_hash("fn b() {\n    1\n}\n")
        );
        assert!(build_snippet("x", "f.rs", "c", FILE, &[(1, 5, "fn".into())]).is_err());
        assert!(build_snippet("x", "f.rs", "c", FILE, &[(1, 5, "nope".into())]).is_err());
        assert!(build_snippet(
            "x",
            "f.rs",
            "c",
            FILE,
            &[(2, 3, "fn b".into()), (3, 5, "fn c".into())]
        )
        .is_err());
    }

    fn seg(start: u32, end: u32, code: &str) -> Segment {
        Segment {
            start_line: start,
            end_line: end,
            anchor: String::new(),
            content_hash: content_hash(code),
            code: code.into(),
        }
    }

    #[test]
    fn locate_classifies_exact_moved_ambiguous_changed() {
        let s = seg(2, 4, "fn b() {\n    1\n}\n");
        assert_eq!(locate(&s, FILE), Location::Exact);
        let moved = format!("// new\n{FILE}");
        assert_eq!(locate(&s, &moved), Location::Moved { start_line: 3 });
        let twice = format!("{FILE}fn b() {{\n    1\n}}\n");
        assert_eq!(locate(&s, &twice), Location::Exact);
        let twice_shifted = format!("//\n{twice}");
        assert_eq!(
            locate(&s, &twice_shifted),
            Location::Ambiguous { occurrences: 2 }
        );
        let changed = FILE.replace("    1", "    2");
        assert_eq!(locate(&s, &changed), Location::Changed);
        assert_eq!(
            locate(&seg(10, 12, "zzz\n"), FILE),
            Location::OutOfRange { file_lines: 5 }
        );
    }

    #[test]
    fn occurrences_count_overlaps() {
        assert_eq!(count_occurrences("aaa", "aa"), 2);
        assert_eq!(count_occurrences("abc", "d"), 0);
        assert_eq!(count_occurrences("abc", ""), 0);
    }
}
