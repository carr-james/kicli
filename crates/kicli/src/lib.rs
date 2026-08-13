//! kicli gives LLM agents eyes and hands in KiCad 10.0 projects.
//!
//! The crate reads and mutates KiCad schematics at full human parity. It scores
//! them with deterministic geometry rules. It never calls a model or a network
//! service, so a score is reproducible offline and comparable across runs.
//!
//! # Structure
//!
//! The workspace is three crates. `kicli-sexpr` holds the s-expression layer
//! and knows nothing about schematics. This crate holds the schematic meaning.
//! `xtask` holds the workspace automation.
//!
//! The modules below each own one concern. [`cli`] depends on the other
//! modules. No module depends on [`cli`].
//!
//! # Status
//!
//! The modules below are empty. The crate exposes only [`version`].

// Undocumented public items and unsafe code are errors. This problem domain
// never needs unsafe. Pedantic lints warn; allow one only with a reason beside
// it.
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::pedantic)]

pub mod model;

pub mod geometry;

pub mod connectivity;

pub mod view;

pub mod lint;

pub mod render;

pub mod libraries;

pub mod kicad;

pub mod pcb;

pub mod cli;

/// The version of this build of kicli.
///
/// # Examples
///
/// ```
/// assert!(!kicli::version().is_empty());
/// ```
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
