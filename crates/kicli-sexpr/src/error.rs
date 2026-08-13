//! Typed errors for parsing and emitting.

use std::ops::Range;

/// Something a KiCad s-expression file can be wrong about.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SexprError {
    /// A quoted string ran to the end of the file with no closing quote.
    #[error("unterminated string starting at byte {0}")]
    UnterminatedString(usize),

    /// A closing parenthesis had no matching opening parenthesis.
    #[error("unmatched closing parenthesis at byte {0}")]
    UnmatchedClose(usize),

    /// The file ended while a list was still open.
    #[error("unclosed list opened at byte {0}")]
    UnclosedList(usize),

    /// The file held no list at all.
    #[error("no s-expression found")]
    Empty,

    /// An atom was expected but a list was found, or the reverse.
    #[error("expected {expected} at byte {at}")]
    Unexpected {
        /// What the caller wanted.
        expected: &'static str,
        /// Where it looked.
        at: usize,
    },
}

impl SexprError {
    /// The byte offset the error points at, when it has one.
    #[must_use]
    pub fn offset(&self) -> Option<usize> {
        match self {
            Self::UnterminatedString(at)
            | Self::UnmatchedClose(at)
            | Self::UnclosedList(at)
            | Self::Unexpected { at, .. } => Some(*at),
            Self::Empty => None,
        }
    }
}

/// A byte range in the source text.
pub type Span = Range<usize>;
