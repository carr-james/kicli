//! kicli gives LLM agents eyes and hands in KiCad 10.0 projects.
//!
//! The crate reads and mutates KiCad schematics at full human parity. It scores
//! them with deterministic geometry rules. It never calls a model or a network
//! service. See `spec/SPEC.md` for the specification and `CONSTITUTION.md` for
//! the binding principles.
//!
//! # Structure
//!
//! The workspace is three crates (`ENGINEERING.md` "Structure"). `kicli-sexpr`
//! holds the s-expression layer and knows nothing about schematics. This crate
//! holds the schematic meaning. `xtask` holds the workspace automation.
//!
//! The modules below split along the seams in `ENGINEERING.md`. Each module
//! owns one concern. [`cli`] depends on the other modules. No module depends on
//! [`cli`].
//!
//! # Status
//!
//! The modules below are empty. The crate exposes only [`version`].

// Crate lints, per ENGINEERING.md "Machine-enforced gates".
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::pedantic)]

pub mod model;

pub mod geometry;

pub mod connectivity;

pub mod lint;

pub mod render;

pub mod libraries;

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
