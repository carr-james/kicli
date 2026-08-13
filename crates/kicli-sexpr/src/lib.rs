//! Tokens, the token-preserving tree, and KiCad's pretty-printer.
//!
//! This crate parses KiCad s-expression files into a tree. The tree keeps the
//! exact source text of every atom. The crate emits that tree back through a
//! port of KiCad's `KICAD_FORMAT::Prettify`
//! (`common/io/kicad/kicad_io_utils.cpp`). Two round-trip properties live here.
//! An emit of an unedited KiCad-authored file reproduces its bytes exactly. A
//! parse of any emitted file reproduces the tree it came from.
//!
//! KiCad writes a flat token stream, then reformats the whole buffer, so
//! whitespace carries no information. Keeping each atom's source text and
//! replaying the prettifier therefore reproduces the file exactly. A tree that
//! stored whitespace would be larger and would buy nothing.
//!
//! This crate knows nothing about schematics. It handles s-expressions only.
//! Schematic meaning lives in the `kicli` crate, and the crate boundary keeps
//! that direction one-way.
//!
//! # Status
//!
//! This crate is not implemented yet. It exposes no items.

// Undocumented public items and unsafe code are errors. This problem domain
// never needs unsafe. Pedantic lints warn; allow one only with a reason beside
// it.
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
