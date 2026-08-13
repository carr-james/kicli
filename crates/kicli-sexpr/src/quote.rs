//! Quoting, exactly as KiCad's output formatter does it.
//!
//! KiCad escapes four characters and no others (`common/richio.cpp`). A tab, a
//! parenthesis, or any UTF-8 byte goes into the file raw. A tool that also
//! escapes tabs, or that emits `\uXXXX`, writes a file KiCad reads back with
//! different text in it.

/// Wrap `value` in quotes and escape what KiCad escapes.
///
/// # Examples
///
/// ```
/// use kicli_sexpr::quote;
/// assert_eq!(quote("plain"), "\"plain\"");
/// assert_eq!(quote("say \"hi\""), "\"say \\\"hi\\\"\"");
/// assert_eq!(quote("a\tb"), "\"a\tb\"");
/// ```
#[must_use]
pub fn quote(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for character in value.chars() {
        match character {
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

/// Read the text out of a quoted token.
///
/// `raw` includes both quote characters. An unknown escape keeps its backslash,
/// which is what KiCad's reader does.
///
/// # Examples
///
/// ```
/// use kicli_sexpr::unquote;
/// assert_eq!(unquote("\"say \\\"hi\\\"\""), "say \"hi\"");
/// assert_eq!(unquote("\"a\\nb\""), "a\nb");
/// ```
#[must_use]
pub fn unquote(raw: &str) -> String {
    let inner = raw
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(raw);

    let mut out = String::with_capacity(inner.len());
    let mut characters = inner.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            out.push(character);
            continue;
        }
        match characters.next() {
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            // KiCad's reader leaves an unknown escape alone, and a trailing
            // backslash is just a backslash.
            other => {
                out.push('\\');
                out.extend(other);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_four_characters_are_escaped() {
        assert_eq!(quote("tab\there"), "\"tab\there\"");
        assert_eq!(quote("paren ( )"), "\"paren ( )\"");
        assert_eq!(quote("µΩ±°C"), "\"µΩ±°C\"");
        assert_eq!(quote("line\nbreak"), "\"line\\nbreak\"");
        assert_eq!(quote("carriage\rreturn"), "\"carriage\\rreturn\"");
        assert_eq!(quote("back\\slash"), "\"back\\\\slash\"");
    }

    #[test]
    fn quoting_round_trips_awkward_text() {
        for value in [
            "",
            "~",
            "#PWR01",
            "a\tb",
            "a\nb",
            "a\\b",
            "a\"b",
            "a\\\"b",
            "µ Ω ± °C — ✓",
            "((()))",
            "trailing backslash \\",
        ] {
            assert_eq!(unquote(&quote(value)), value, "value {value:?}");
        }
    }

    #[test]
    fn an_unknown_escape_keeps_its_backslash() {
        assert_eq!(unquote("\"a\\tb\""), "a\\tb");
    }
}
