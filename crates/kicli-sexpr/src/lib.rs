//! Tokens, the token-preserving tree, and KiCad's pretty-printer.
//!
//! This crate parses KiCad s-expression files into a tree. The tree keeps the
//! exact source text of every atom. The crate emits that tree back through a
//! port of `KICAD_FORMAT::Prettify`. The P1 and P2 round-trip properties live
//! here.
//!
//! This crate knows nothing about schematics. It handles s-expressions only.
//! Schematic meaning lives in the `kicli` crate. The crate boundary enforces
//! that direction. See `spec/SPEC.md` §5 and `research/sexpr-strategy.md`.
//!
//! # Status
//!
//! Milestone M1 is in progress. This crate is empty. See `tasks/M1.md` T3, T4
//! and T6.

// Crate lints, per ENGINEERING.md "Machine-enforced gates".
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
