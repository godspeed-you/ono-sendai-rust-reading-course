//! HTML escaping. Every piece of text that reaches a page goes through one of these functions.

/// Escape text for an HTML text node or a double-quoted attribute value.
pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// Fill `{{slot}}` placeholders in a template. Values are inserted verbatim (they are HTML
/// produced by the renderer); an unknown or unfilled slot is a bug and panics in tests.
pub fn fill(template: &str, slots: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len() + 4096);
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let end = after.find("}}").expect("unterminated template slot");
        let name = after[..end].trim();
        let value = slots
            .iter()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| *v)
            .unwrap_or_else(|| panic!("template slot `{name}` not provided"));
        out.push_str(value);
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_all_special_characters() {
        assert_eq!(
            escape(r#"<a href="x">'&'</a>"#),
            "&lt;a href=&quot;x&quot;&gt;&#39;&amp;&#39;&lt;/a&gt;"
        );
    }

    #[test]
    fn fills_slots() {
        assert_eq!(
            fill("a {{x}} b {{ y }}", &[("x", "1"), ("y", "<2>")]),
            "a 1 b <2>"
        );
    }

    #[test]
    #[should_panic(expected = "not provided")]
    fn missing_slot_panics() {
        fill("{{nope}}", &[]);
    }
}
