//! Tokens, the token-preserving tree, and KiCad's pretty-printer.
//!
//! This module parses KiCad s-expression files into a tree that keeps the exact
//! source text of every atom. It emits that tree back through a port of
//! `KICAD_FORMAT::Prettify`. This is where the P1 and P2 round-trip properties
//! live. See `spec/SPEC.md` §5 and `research/sexpr-strategy.md`.
