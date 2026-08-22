//! Deterministic style rules and the readability score.
//!
//! This module scores how a schematic is drawn. It layers on KiCad's own
//! electrical rule check and repeats none of those checks, because the
//! electrical layer already owns them. Detection uses integer geometry only, so
//! two runs over one file always agree.
//!
//! # The seam
//!
//! A rule is one implementation of [`Rule`]. Rules live one family to a file
//! under `src/lint/rules/`. The build script reads that directory and writes
//! the module list and the registry, so a new rule is a new file and nothing
//! else. [`registry`] holds the generated list.
//!
//! # What this module may not do
//!
//! The module knows nothing of the command line, files on disk, or
//! `kicad-cli`. It never writes. A rule suggests a command as text and stops
//! there. `cargo test --test the_linter_holds_no_write_path` is the
//! enforcement.

pub mod drawing;

pub mod engine;

pub mod finding;

pub mod registry;

pub mod rule;

pub use drawing::Drawing;
pub use engine::Engine;
pub use finding::{Finding, Penalty, RuleId, Severity, Tier};
pub use rule::{Findings, Rule};
