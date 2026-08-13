//! Tokens, the token-preserving tree, and KiCad's pretty-printer.
//!
//! This crate parses KiCad s-expression files into a tree. The tree keeps the
//! exact source text of every atom. The crate emits that tree back through a
//! port of `KICAD_FORMAT::Prettify`. Two round-trip properties live here. An
//! emit of an unedited KiCad-authored file reproduces its bytes exactly. A
//! parse of any emitted file reproduces the tree it came from.
//!
//! This crate knows nothing about schematics. It handles s-expressions only.
//! Schematic meaning lives in the `kicli` crate. The crate boundary enforces
//! that direction. See `spec/SPEC.md` §5 and `research/sexpr-strategy.md`.
//!
//! # Status
//!
//! This crate is not implemented yet. It exposes no items.

// Crate lints, per ENGINEERING.md "Machine-enforced gates".
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
