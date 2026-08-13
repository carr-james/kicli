//! A byte lexer for KiCad's s-expression dialect.
//!
//! The lexer classifies bytes and nothing more. It does not turn `41.91` into a
//! number, and it does not resolve `\"` into a quote. Both are queries on the
//! token, so a value the caller never touches can never come back changed.

use crate::error::SexprError;

/// What a token is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    /// An opening parenthesis.
    LParen,
    /// A closing parenthesis.
    RParen,
    /// An unquoted run: a number, a keyword, or a symbol.
    Bare,
    /// A quoted string, including both quote characters.
    Quoted,
    /// A `#` comment running to the end of its line.
    ///
    /// KiCad's lexer accepts these and drops them on save.
    Comment,
}

/// One token, as a range of the source text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token {
    /// What the token is.
    pub kind: TokenKind,
    /// Byte offset of the first byte.
    pub start: usize,
    /// Byte offset one past the last byte.
    pub end: usize,
}

impl Token {
    /// The token's text.
    #[must_use]
    pub fn text<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start..self.end]
    }
}

fn is_whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\r')
}

/// Split `source` into tokens.
///
/// # Errors
///
/// Returns [`SexprError::UnterminatedString`] when a quoted string reaches the
/// end of the file with no closing quote.
pub fn lex(source: &str) -> Result<Vec<Token>, SexprError> {
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut cursor = 0;
    let mut at_line_start = true;

    while cursor < bytes.len() {
        if is_whitespace(bytes[cursor]) {
            at_line_start |= bytes[cursor] == b'\n';
            cursor += 1;
            continue;
        }

        let token = scan_token(bytes, cursor, at_line_start)?;
        at_line_start = false;
        cursor = token.end;
        tokens.push(token);
    }

    Ok(tokens)
}

/// Classify the token starting at `start`.
///
/// KiCad treats `#` as a comment only when it opens a line. Inside a line it is
/// an ordinary character, which is what lets a reference designator like
/// #PWR01 be a bare token.
fn scan_token(bytes: &[u8], start: usize, at_line_start: bool) -> Result<Token, SexprError> {
    let (kind, end) = match bytes[start] {
        b'#' if at_line_start => (TokenKind::Comment, scan_comment(bytes, start)),
        b'(' => (TokenKind::LParen, start + 1),
        b')' => (TokenKind::RParen, start + 1),
        b'"' => (TokenKind::Quoted, scan_quoted(bytes, start)?),
        _ => (TokenKind::Bare, scan_bare(bytes, start)),
    };
    Ok(Token { kind, start, end })
}

/// Scan a comment, returning the offset of its terminating newline.
fn scan_comment(bytes: &[u8], start: usize) -> usize {
    let mut cursor = start;
    while cursor < bytes.len() && bytes[cursor] != b'\n' {
        cursor += 1;
    }
    cursor
}

/// Scan a quoted string, returning the offset one past its closing quote.
fn scan_quoted(bytes: &[u8], start: usize) -> Result<usize, SexprError> {
    let mut cursor = start + 1;
    loop {
        if cursor >= bytes.len() {
            return Err(SexprError::UnterminatedString(start));
        }
        match bytes[cursor] {
            // A backslash escapes whatever follows it, including a quote.
            b'\\' => cursor += 2,
            b'"' => return Ok(cursor + 1),
            _ => cursor += 1,
        }
        // A trailing backslash can step the cursor past the end.
        if cursor > bytes.len() {
            return Err(SexprError::UnterminatedString(start));
        }
    }
}

/// Scan an unquoted run, returning the offset one past its last byte.
fn scan_bare(bytes: &[u8], start: usize) -> usize {
    let mut cursor = start;
    while cursor < bytes.len()
        && !is_whitespace(bytes[cursor])
        && !matches!(bytes[cursor], b'(' | b')' | b'"')
    {
        cursor += 1;
    }
    cursor
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(source: &str) -> Vec<TokenKind> {
        lex(source).expect("lexes").iter().map(|t| t.kind).collect()
    }

    #[test]
    fn parentheses_and_bare_atoms() {
        assert_eq!(
            kinds("(version 20260306)"),
            [
                TokenKind::LParen,
                TokenKind::Bare,
                TokenKind::Bare,
                TokenKind::RParen
            ]
        );
    }

    #[test]
    fn a_quoted_string_keeps_its_quotes() {
        let source = r#"(a "b c")"#;
        let tokens = lex(source).expect("lexes");
        assert_eq!(tokens[2].kind, TokenKind::Quoted);
        assert_eq!(tokens[2].text(source), "\"b c\"");
    }

    #[test]
    fn an_escaped_quote_does_not_end_the_string() {
        let source = r#"(a "b\"c")"#;
        let tokens = lex(source).expect("lexes");
        assert_eq!(tokens[2].text(source), r#""b\"c""#);
    }

    #[test]
    fn a_string_ending_in_an_escaped_backslash_closes() {
        let source = r#"(a "b\\")"#;
        let tokens = lex(source).expect("lexes");
        assert_eq!(tokens[2].text(source), r#""b\\""#);
        assert_eq!(tokens[3].kind, TokenKind::RParen);
    }

    #[test]
    fn a_comment_needs_to_open_its_line() {
        assert_eq!(
            kinds("# note\n(a)"),
            [
                TokenKind::Comment,
                TokenKind::LParen,
                TokenKind::Bare,
                TokenKind::RParen
            ]
        );
        // Mid-line, `#` is just a character.
        assert_eq!(
            kinds("(a #PWR01)"),
            [
                TokenKind::LParen,
                TokenKind::Bare,
                TokenKind::Bare,
                TokenKind::RParen
            ]
        );
    }

    #[test]
    fn an_unterminated_string_is_an_error() {
        assert_eq!(lex("(a \"b"), Err(SexprError::UnterminatedString(3)));
    }

    #[test]
    fn tokens_cover_the_source_minus_whitespace() {
        let source = "(a\n\t(b  \"c d\")\n)";
        let tokens = lex(source).expect("lexes");
        let joined: String = tokens.iter().map(|t| t.text(source)).collect();
        let expected: String = source.split_whitespace().collect::<Vec<_>>().join("");
        // Whitespace inside the string survives, so compare without it.
        assert_eq!(joined.replace(' ', ""), expected.replace(' ', ""));
    }
}
