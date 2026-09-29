//! Build-time Rust syntax highlighting (spec §59): a small lexer that classifies tokens and emits
//! escaped HTML, one string per source line, so line numbers and line highlights stay exact.
//! Multi-line tokens (block comments, raw strings) are closed at each line end and reopened on
//! the next line, so every line is well-formed HTML on its own.

use crate::html::escape;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    Keyword,
    Type,
    Str,
    Num,
    Comment,
    Doc,
    Macro,
    Attr,
    Lifetime,
}

impl Class {
    fn css(self) -> &'static str {
        match self {
            Class::Keyword => "tok-kw",
            Class::Type => "tok-ty",
            Class::Str => "tok-str",
            Class::Num => "tok-num",
            Class::Comment => "tok-com",
            Class::Doc => "tok-doc",
            Class::Macro => "tok-mac",
            Class::Attr => "tok-attr",
            Class::Lifetime => "tok-life",
        }
    }
}

const KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while", "yield", "union",
];

const PRIMITIVES: &[&str] = &[
    "bool", "char", "str", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64",
    "i128", "isize", "f32", "f64",
];

/// Tokenize into `(class, text)` runs; unclassified text has `None`.
pub fn tokenize(src: &str) -> Vec<(Option<Class>, &str)> {
    let b = src.as_bytes();
    let mut out: Vec<(Option<Class>, &str)> = Vec::new();
    let mut i = 0;
    let mut plain_start = 0;
    macro_rules! emit {
        ($class:expr, $end:expr) => {{
            if plain_start < i {
                out.push((None, &src[plain_start..i]));
            }
            let end = $end;
            out.push((Some($class), &src[i..end]));
            i = end;
            plain_start = i;
            continue;
        }};
    }
    while i < b.len() {
        let c = b[i];
        // Comments.
        if c == b'/' && b.get(i + 1) == Some(&b'/') {
            let end = src[i..].find('\n').map_or(b.len(), |p| i + p);
            let t = &src[i..end];
            let doc = (t.starts_with("///") && !t.starts_with("////")) || t.starts_with("//!");
            emit!(if doc { Class::Doc } else { Class::Comment }, end);
        }
        if c == b'/' && b.get(i + 1) == Some(&b'*') {
            let mut depth = 0;
            let mut j = i;
            while j < b.len() {
                if b[j] == b'/' && b.get(j + 1) == Some(&b'*') {
                    depth += 1;
                    j += 2;
                } else if b[j] == b'*' && b.get(j + 1) == Some(&b'/') {
                    depth -= 1;
                    j += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    j += 1;
                }
            }
            let t = &src[i..j.min(b.len())];
            let doc = (t.starts_with("/**") && !t.starts_with("/***") && t != "/**/")
                || t.starts_with("/*!");
            emit!(
                if doc { Class::Doc } else { Class::Comment },
                j.min(b.len())
            );
        }
        // Attributes: #[...] and #![...], bracket-balanced, string-aware.
        if c == b'#'
            && (b.get(i + 1) == Some(&b'[')
                || (b.get(i + 1) == Some(&b'!') && b.get(i + 2) == Some(&b'[')))
        {
            let mut j = i + 1;
            let mut depth = 0;
            let mut in_str = false;
            while j < b.len() {
                let d = b[j];
                if in_str {
                    if d == b'\\' {
                        j += 1;
                    } else if d == b'"' {
                        in_str = false;
                    }
                } else if d == b'"' {
                    in_str = true;
                } else if d == b'[' {
                    depth += 1;
                } else if d == b']' {
                    depth -= 1;
                    if depth == 0 {
                        j += 1;
                        break;
                    }
                }
                j += 1;
            }
            emit!(Class::Attr, j.min(b.len()));
        }
        // Raw strings: r"..", r#".."#, br"..", cr"..".
        if let Some(end) = raw_string_end(b, i) {
            emit!(Class::Str, end);
        }
        // Strings: "..", b"..", c"..".
        if c == b'"'
            || ((c == b'b' || c == b'c') && b.get(i + 1) == Some(&b'"') && !prev_is_ident(b, i))
        {
            let mut j = if c == b'"' { i + 1 } else { i + 2 };
            while j < b.len() {
                if b[j] == b'\\' {
                    j += 2;
                    continue;
                }
                if b[j] == b'"' {
                    j += 1;
                    break;
                }
                j += 1;
            }
            emit!(Class::Str, j.min(b.len()));
        }
        // Char literals and lifetimes.
        if c == b'\'' || (c == b'b' && b.get(i + 1) == Some(&b'\'') && !prev_is_ident(b, i)) {
            let q = if c == b'b' { i + 1 } else { i };
            if let Some(end) = char_literal_end(src, q) {
                emit!(Class::Str, end);
            }
            if c == b'\'' {
                let mut j = i + 1;
                while j < b.len() && is_ident_byte(b[j]) {
                    j += 1;
                }
                if j > i + 1 {
                    emit!(Class::Lifetime, j);
                }
            }
        }
        // Numbers.
        if c.is_ascii_digit() && !prev_is_ident(b, i) {
            let mut j = i + 1;
            while j < b.len() {
                let d = b[j];
                let fraction = d == b'.' && b.get(j + 1).is_some_and(|n| n.is_ascii_digit());
                if d.is_ascii_alphanumeric() || d == b'_' || fraction {
                    j += 1;
                } else {
                    break;
                }
            }
            emit!(Class::Num, j);
        }
        // Identifiers, keywords, macros, types.
        if is_ident_start(c) && !prev_is_ident(b, i) {
            let mut j = i;
            while j < b.len() && is_ident_byte(b[j]) {
                j += 1;
            }
            let word = &src[i..j];
            if b.get(j) == Some(&b'!') && b.get(j + 1) != Some(&b'=') && !KEYWORDS.contains(&word) {
                emit!(Class::Macro, j + 1);
            }
            if KEYWORDS.contains(&word) {
                emit!(Class::Keyword, j);
            }
            if PRIMITIVES.contains(&word) || word.as_bytes()[0].is_ascii_uppercase() {
                emit!(Class::Type, j);
            }
            i = j;
            continue;
        }
        // Advance one full character.
        i += src[i..].chars().next().map_or(1, char::len_utf8);
    }
    if plain_start < b.len() {
        out.push((None, &src[plain_start..]));
    }
    out
}

fn is_ident_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_'
}

fn is_ident_byte(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

fn prev_is_ident(b: &[u8], i: usize) -> bool {
    i > 0 && is_ident_byte(b[i - 1])
}

fn raw_string_end(b: &[u8], i: usize) -> Option<usize> {
    if prev_is_ident(b, i) {
        return None;
    }
    let mut j = i;
    if b.get(j) == Some(&b'b') || b.get(j) == Some(&b'c') {
        j += 1;
    }
    if b.get(j) != Some(&b'r') {
        return None;
    }
    j += 1;
    let mut hashes = 0;
    while b.get(j) == Some(&b'#') {
        hashes += 1;
        j += 1;
    }
    if b.get(j) != Some(&b'"') {
        return None;
    }
    j += 1;
    while j < b.len() {
        if b[j] == b'"' && (0..hashes).all(|k| b.get(j + 1 + k) == Some(&b'#')) {
            return Some(j + 1 + hashes);
        }
        j += 1;
    }
    Some(b.len())
}

/// End of a char literal starting at the quote `q`, or `None` if it is a lifetime.
fn char_literal_end(src: &str, q: usize) -> Option<usize> {
    let rest = &src[q + 1..];
    let mut chars = rest.char_indices();
    let (_, first) = chars.next()?;
    if first == '\\' {
        // Escape: find the closing quote within a short distance (e.g. '\u{1F600}').
        let close = rest[1..].find('\'')?;
        if close <= 10 {
            return Some(q + 1 + 1 + close + 1);
        }
        return None;
    }
    let (idx, next) = chars.next()?;
    if next == '\'' && first != '\'' {
        return Some(q + 1 + idx + 1);
    }
    None
}

/// Highlight Rust source into one HTML string per line.
pub fn highlight_rust(src: &str) -> Vec<String> {
    let src = src.strip_suffix('\n').unwrap_or(src);
    let mut lines = vec![String::new()];
    for (class, text) in tokenize(src) {
        let mut first = true;
        for part in text.split('\n') {
            if !first {
                lines.push(String::new());
            }
            first = false;
            if part.is_empty() {
                continue;
            }
            let line = lines.last_mut().expect("at least one line");
            match class {
                Some(c) => {
                    line.push_str("<span class=\"");
                    line.push_str(c.css());
                    line.push_str("\">");
                    line.push_str(&escape(part));
                    line.push_str("</span>");
                }
                None => line.push_str(&escape(part)),
            }
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn classes(src: &str) -> Vec<(Option<Class>, String)> {
        tokenize(src)
            .into_iter()
            .filter(|(_, t)| !t.trim().is_empty())
            .map(|(c, t)| (c, t.trim().to_string()))
            .collect()
    }

    #[test]
    fn keywords_types_and_macros() {
        let t = classes("pub fn f(x: u32) -> Option<String> { println!(\"{x}\") }");
        assert!(t.contains(&(Some(Class::Keyword), "pub".into())));
        assert!(t.contains(&(Some(Class::Type), "u32".into())));
        assert!(t.contains(&(Some(Class::Type), "Option".into())));
        assert!(t.contains(&(Some(Class::Macro), "println!".into())));
        assert!(t.contains(&(Some(Class::Str), "\"{x}\"".into())));
    }

    #[test]
    fn lifetimes_versus_chars() {
        let t = classes("fn f<'a>(c: char) -> &'a str { let q = 'x'; let e = '\\n'; }");
        assert!(t.contains(&(Some(Class::Lifetime), "'a".into())));
        assert!(t.contains(&(Some(Class::Str), "'x'".into())));
        assert!(t.contains(&(Some(Class::Str), "'\\n'".into())));
    }

    #[test]
    fn comments_docs_and_attributes() {
        let t = classes(
            "//! crate doc\n/// item doc\n// plain\n#[derive(Debug)]\n#![forbid(unsafe_code)]",
        );
        assert_eq!(t[0].0, Some(Class::Doc));
        assert_eq!(t[1].0, Some(Class::Doc));
        assert_eq!(t[2].0, Some(Class::Comment));
        assert_eq!(t[3], (Some(Class::Attr), "#[derive(Debug)]".into()));
        assert_eq!(t[4], (Some(Class::Attr), "#![forbid(unsafe_code)]".into()));
    }

    #[test]
    fn raw_strings_and_ranges() {
        let t = classes("let s = r#\"a \"quoted\" b\"#; for i in 0..10 {}");
        assert!(t.contains(&(Some(Class::Str), "r#\"a \"quoted\" b\"#".into())));
        assert!(t.contains(&(Some(Class::Num), "0".into())));
        assert!(t.contains(&(Some(Class::Num), "10".into())));
    }

    #[test]
    fn multiline_tokens_are_split_per_line() {
        let lines = highlight_rust("/* a\n b */ x\n");
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "<span class=\"tok-com\">/* a</span>");
        assert_eq!(lines[1], "<span class=\"tok-com\"> b */</span> x");
    }

    #[test]
    fn output_is_escaped_and_preserves_text() {
        let src = "if a < b && c > d { \"<&>\" }\n\tlet x = 1;\n";
        let lines = highlight_rust(src);
        assert!(lines[0].contains("&lt;"));
        // Text content with tags removed equals the input.
        let mut text = String::new();
        let joined = lines.join("\n");
        let mut tag = false;
        for ch in joined.chars() {
            if ch == '<' {
                tag = true;
                continue;
            }
            if ch == '>' && tag {
                tag = false;
                continue;
            }
            if !tag {
                text.push(ch);
            }
        }
        let text = text
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&amp;", "&");
        assert_eq!(text, src.trim_end_matches('\n'));
    }
}
