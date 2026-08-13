//! Command surface, output formatting, and exit codes.
//!
//! This module parses arguments and renders results as text or JSON. It owns
//! the exit-code table and the translation of `kicad-cli`'s codes. It depends on
//! the other modules. No other module depends on it. See `spec/SPEC.md` §6.
