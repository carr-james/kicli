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
//! # The electrical layer is KiCad's, and this module layers on it
//!
//! KiCad's own rule check already implements 47 electrical checks. kicli runs
//! it, maps what it says into kicli findings, and repeats none of it
//! (`spec/SPEC.md` §11.1). [`erc`] is the seam that arrives on: pure data,
//! handed in by [`crate::kicad`], read by a rule off its [`Drawing`].
//!
//! # What this module may not do
//!
//! The module knows nothing of the command line, files on disk, or
//! `kicad-cli`. It never writes. A rule suggests a command as text and stops
//! there. `cargo test --test the_linter_holds_no_write_path` is the
//! enforcement.

pub mod drawing;

pub mod engine;

pub mod erc;

pub mod finding;

pub mod gate;

pub mod registry;

pub mod rule;

pub mod score;

pub use drawing::Drawing;
pub use engine::Engine;
pub use erc::{KicadSeverity, RuleCheck};
pub use finding::{Finding, Penalty, RuleId, Severity, Tier};
pub use gate::{Blocker, Counted, Gate, Report, Saturation};
pub use rule::{Findings, Rule};
pub use score::{Density, Normaliser, RawPenalty, SheetScore};
