//! A port of KiCad's whole-file pretty-printer.
//!
//! KiCad's writers emit a flat token stream with no layout, then run
//! `KICAD_FORMAT::Prettify` over the whole buffer
//! (`common/io/kicad/kicad_io_utils.cpp`). Layout is therefore a pure function
//! of the token stream, and reproducing this function is what makes
//! byte-identical output possible.
//!
//! This is a literal port, including the four rules the upstream doc comment
//! leaves out: `(xy ...)` run packing, the long-token wrap, the short-form and
//! library-row special cases, and the backslash-parity quote tracking.

/// Which layout KiCad applies.
///
/// KiCad picks the mode from what it is writing and from one advanced setting,
/// so a file's mode must be detected and preserved. Writing a compact file back
/// as normal would reformat every line of somebody's project.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FormatMode {
    /// Schematics, boards and symbol libraries.
    #[default]
    Normal,
    /// The clipboard, local history, and every save when `CompactSave` is on.
    ///
    /// Keeps `font`, `stroke`, `fill`, `teardrop`, `offset`, `rotate` and
    /// `scale` lists on one line.
    CompactTextProperties,
    /// Library tables. Keeps one `lib` row per line.
    LibraryTable,
}

/// Consecutive `(xy ...)` lists share a line until this column.
const XY_COLUMN_LIMIT: usize = 99;

/// Whitespace inside a list becomes a newline once the column reaches this.
const TOKEN_WRAP_THRESHOLD: usize = 72;

/// Tokens that stay on one line in [`FormatMode::CompactTextProperties`].
const SHORT_FORM_TOKENS: [&str; 7] = [
    "font", "stroke", "fill", "teardrop", "offset", "rotate", "scale",
];

fn is_whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\r')
}

/// Read the alphabetic token that starts one byte after `at`.
fn token_after(bytes: &[u8], at: usize) -> &str {
    let start = at + 1;
    let mut end = start;
    while end < bytes.len() && bytes[end].is_ascii_alphabetic() {
        end += 1;
    }
    std::str::from_utf8(&bytes[start..end]).unwrap_or("")
}

/// Does an `(xy ` list start at `at`?
fn is_xy(bytes: &[u8], at: usize) -> bool {
    bytes.get(at + 1) == Some(&b'x')
        && bytes.get(at + 2) == Some(&b'y')
        && bytes.get(at + 3) == Some(&b' ')
}

/// The next non-whitespace byte at or after `from`.
fn next_non_whitespace(bytes: &[u8], from: usize) -> u8 {
    let mut seek = from;
    while seek < bytes.len() && is_whitespace(bytes[seek]) {
        seek += 1;
    }
    bytes.get(seek).copied().unwrap_or(0)
}

/// Lay out a token stream exactly as KiCad would.
///
/// The input may carry any whitespace, or none. Only the tokens matter.
///
/// # Panics
///
/// Cannot panic in practice. Every byte written comes either from `source`,
/// which is valid UTF-8, or from this function, which writes only ASCII.
#[must_use]
#[allow(clippy::too_many_lines)] // A literal port. Splitting it would hide the
// correspondence with the upstream function, which is the only thing that makes
// this code checkable.
pub fn prettify(source: &str, mode: FormatMode) -> String {
    let bytes = source.as_bytes();
    let mut formatted: Vec<u8> = Vec::with_capacity(source.len());

    let text_special_case = mode == FormatMode::CompactTextProperties;
    let lib_special_case = mode == FormatMode::LibraryTable;

    let mut list_depth: usize = 0;
    let mut lib_depth: usize = 0;
    let mut last_non_whitespace = 0u8;
    let mut in_quote = false;
    let mut has_inserted_space = false;
    let mut in_multi_line_list = false;
    let mut in_xy = false;
    let mut in_short_form = false;
    let mut in_lib_row = false;
    let mut short_form_depth: usize = 0;
    let mut column: usize = 0;
    let mut backslash_count: usize = 0;

    let mut cursor = 0;
    while cursor < bytes.len() {
        let byte = bytes[cursor];

        if is_whitespace(byte) && !in_quote {
            let next = next_non_whitespace(bytes, cursor);

            if !has_inserted_space
                && list_depth > 0
                && last_non_whitespace != b'('
                && next != b')'
                && next != b'('
            {
                if in_xy || column < TOKEN_WRAP_THRESHOLD {
                    formatted.push(b' ');
                    column += 1;
                } else if in_short_form || in_lib_row {
                    formatted.push(b' ');
                } else {
                    formatted.push(b'\n');
                    formatted.extend(std::iter::repeat_n(b'\t', list_depth));
                    column = list_depth;
                    in_multi_line_list = true;
                }
                has_inserted_space = true;
            }
        } else {
            has_inserted_space = false;

            if byte == b'(' && !in_quote {
                let current_is_xy = is_xy(bytes, cursor);
                let current_is_short_form =
                    text_special_case && SHORT_FORM_TOKENS.contains(&token_after(bytes, cursor));
                let current_is_lib = lib_special_case && token_after(bytes, cursor) == "lib";

                if formatted.is_empty() {
                    formatted.push(b'(');
                    column += 1;
                } else if (in_xy && current_is_xy && column < XY_COLUMN_LIMIT)
                    || in_short_form
                    || in_lib_row
                {
                    // A run of points, a short form, or a library row keeps
                    // sharing its line.
                    formatted.extend_from_slice(b" (");
                    column += 2;
                } else {
                    formatted.push(b'\n');
                    formatted.extend(std::iter::repeat_n(b'\t', list_depth));
                    formatted.push(b'(');
                    column = list_depth + 1;
                }

                in_xy = current_is_xy;

                if current_is_short_form {
                    in_short_form = true;
                    short_form_depth = list_depth;
                } else if current_is_lib {
                    in_lib_row = true;
                    lib_depth = list_depth;
                }

                list_depth += 1;
            } else if byte == b')' && !in_quote {
                list_depth = list_depth.saturating_sub(1);

                if in_short_form {
                    formatted.push(b')');
                    column += 1;
                } else if in_lib_row && list_depth == lib_depth {
                    formatted.push(b')');
                    in_lib_row = false;
                } else if last_non_whitespace == b')' || in_multi_line_list {
                    formatted.push(b'\n');
                    formatted.extend(std::iter::repeat_n(b'\t', list_depth));
                    formatted.push(b')');
                    column = list_depth + 1;
                    in_multi_line_list = false;
                } else {
                    formatted.push(b')');
                    column += 1;
                }

                if short_form_depth == list_depth {
                    in_short_form = false;
                    short_form_depth = 0;
                }
            } else {
                // KiCad escapes a double quote as \". The corner case is \\",
                // where the backslash is escaped and the quote is not, so a
                // quote only toggles when an even number of backslashes
                // precedes it.
                if byte == b'\\' {
                    backslash_count += 1;
                } else if byte == b'"' && backslash_count % 2 == 0 {
                    in_quote = !in_quote;
                }
                if byte != b'\\' {
                    backslash_count = 0;
                }

                formatted.push(byte);
                column += 1;
            }

            last_non_whitespace = byte;
        }

        cursor += 1;
    }

    // POSIX wants a newline at the end of a file, and it keeps git diffs clean.
    formatted.push(b'\n');

    // Every byte came from `source`, which is valid UTF-8, and every byte this
    // function inserts is ASCII, so the result is still UTF-8.
    String::from_utf8(formatted).expect("prettify moves whole bytes of valid UTF-8")
}

/// Strip layout while keeping every token.
///
/// Whitespace outside a quoted string collapses to one space. The newline that
/// ends a comment survives, because without it the rest of the file would be
/// swallowed by the comment.
///
/// # Panics
///
/// Cannot panic in practice, for the same reason [`prettify`] cannot.
#[must_use]
pub fn flatten(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(source.len());
    let mut in_quote = false;
    let mut in_comment = false;
    let mut backslash_count = 0usize;
    let mut at_line_start = true;
    let mut cursor = 0;

    while cursor < bytes.len() {
        let byte = bytes[cursor];

        if in_comment {
            if byte == b'\n' {
                in_comment = false;
                at_line_start = true;
                out.push(b'\n');
            } else {
                out.push(byte);
            }
            cursor += 1;
            continue;
        }

        if !in_quote && byte == b'#' && at_line_start {
            open_comment_line(&mut out);
            in_comment = true;
            cursor += 1;
            continue;
        }

        if !in_quote && is_whitespace(byte) {
            let mut seek = cursor;
            while seek < bytes.len() && is_whitespace(bytes[seek]) {
                if bytes[seek] == b'\n' {
                    at_line_start = true;
                }
                seek += 1;
            }
            if !out.is_empty() && !matches!(out.last(), Some(b' ' | b'\n')) {
                out.push(b' ');
            }
            cursor = seek;
            continue;
        }

        at_line_start = false;

        if byte == b'\\' {
            backslash_count += 1;
        } else if byte == b'"' && backslash_count % 2 == 0 {
            in_quote = !in_quote;
        }
        if byte != b'\\' {
            backslash_count = 0;
        }

        out.push(byte);
        cursor += 1;
    }

    // Trailing whitespace carries no token, so it is not worth keeping.
    while matches!(out.last(), Some(b' ' | b'\n')) {
        out.pop();
    }
    String::from_utf8(out).expect("flatten moves whole bytes of valid UTF-8")
}

/// Put a comment back at the start of its own line.
///
/// A comment only counts as one when it opens a line. Collapsing the newline
/// before it would turn it into a bare atom and swallow the rest of the line.
fn open_comment_line(out: &mut Vec<u8>) {
    while matches!(out.last(), Some(b' ')) {
        out.pop();
    }
    if !out.is_empty() && !matches!(out.last(), Some(b'\n')) {
        out.push(b'\n');
    }
    out.push(b'#');
}

/// The mode in which `source` is already laid out, when there is one.
///
/// A file is canonical in exactly the mode whose prettifier leaves it
/// unchanged. Detecting the mode this way costs one pass per candidate and
/// needs no guessing from file names.
#[must_use]
pub fn detect_mode(source: &str) -> Option<FormatMode> {
    [
        FormatMode::Normal,
        FormatMode::CompactTextProperties,
        FormatMode::LibraryTable,
    ]
    .into_iter()
    .find(|&mode| prettify(source, mode) == source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_list_of_atoms_stays_on_one_line() {
        assert_eq!(
            prettify("(version 20260306)", FormatMode::Normal),
            "(version 20260306)\n"
        );
    }

    #[test]
    fn a_list_holding_a_list_breaks_over_lines() {
        assert_eq!(
            prettify("(a (b c))", FormatMode::Normal),
            "(a\n\t(b c)\n)\n"
        );
    }

    #[test]
    fn quotes_hide_parentheses_and_whitespace() {
        let source = r#"(text "a ( b )  c")"#;
        assert_eq!(
            prettify(source, FormatMode::Normal),
            "(text \"a ( b )  c\")\n"
        );
    }

    #[test]
    fn an_escaped_backslash_does_not_open_a_quote() {
        // The string ends with a literal backslash, so the quote after it
        // closes the string rather than opening a new one.
        let source = r#"(a "b\\" (c d))"#;
        assert_eq!(
            prettify(source, FormatMode::Normal),
            "(a \"b\\\\\"\n\t(c d)\n)\n"
        );
    }

    #[test]
    fn prettify_is_idempotent() {
        let source = "(a (b c) (d (e f)))";
        let once = prettify(source, FormatMode::Normal);
        assert_eq!(prettify(&once, FormatMode::Normal), once);
    }

    #[test]
    fn compact_mode_keeps_short_forms_on_one_line() {
        let source = "(effects (font (size 1.27 1.27)) (justify left))";
        let normal = prettify(source, FormatMode::Normal);
        let compact = prettify(source, FormatMode::CompactTextProperties);
        assert!(normal.contains("(font\n"));
        assert!(compact.contains("(font (size 1.27 1.27))"));
    }

    #[test]
    fn flatten_keeps_tokens_and_drops_layout() {
        let source = "(a\n\t(b   c)\n)\n";
        assert_eq!(flatten(source), "(a (b c) )");
    }

    #[test]
    fn flatten_keeps_whitespace_inside_a_string() {
        let source = "(a \"b   c\")";
        assert_eq!(flatten(source), "(a \"b   c\")");
    }
}
