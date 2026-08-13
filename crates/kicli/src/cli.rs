//! Command surface, output formatting, and exit codes.
//!
//! This module parses arguments and renders results as text or JSON. It owns
//! the exit-code table. It translates `kicad-cli`'s exit codes into kicli's own,
//! because the two schemes give different meanings to the same numbers. It
//! depends on the other modules. No other module depends on it.
