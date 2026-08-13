//! What a mutating command does, and what it reports.
//!
//! Every mutation follows one path: change the tree, run the invariants, write
//! atomically, and report what moved. The report is a delta fragment, so a
//! caller reads one vocabulary whether it asked what changed or caused it.
//!
//! The implicit `@last-write` snapshot is updated by every mutation, which is
//! what lets the next command answer "what did my last command change?" with no
//! bookkeeping from the caller.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use kicli_sexpr::Doc;

use crate::geometry::Iu;
use crate::model::invariant::{Report as InvariantReport, check_invariants};
use crate::model::items::{Schematic, SheetPath};
use crate::model::write::WriteOptions;
use crate::model::write_file::{WriteError, Written, write_document};
use crate::view::delta::Delta;
use crate::view::snapshot::{Snapshot, SnapshotError};

/// The name of the snapshot every mutation leaves behind.
pub const LAST_WRITE: &str = "@last-write";

/// What a mutating command changed, and what kicli checked afterwards.
#[derive(Clone, Debug)]
pub struct Mutation {
    /// The file that changed.
    pub path: PathBuf,
    /// What moved, in the delta vocabulary.
    pub delta: Delta,
    /// What the invariants found.
    pub invariants: InvariantReport,
    /// Was the file laid out again because it did not arrive canonical?
    pub reformatted: bool,
}

impl Mutation {
    /// The text form: one line per changed object, then the invariant line.
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = String::new();
        for line in &self.delta.lines {
            text.push_str(&line.to_string());
            text.push('\n');
        }
        let failed: Vec<&str> = self
            .invariants
            .failures()
            .map(|outcome| outcome.invariant.name())
            .collect();
        if failed.is_empty() {
            text.push_str("checked: every invariant passed\n");
        } else {
            let _ = writeln!(text, "checked: {} failed", failed.join(", "));
        }
        if self.reformatted {
            text.push_str("the file was laid out again, as KiCad's next save would\n");
        }
        text
    }

    /// The JSON form, carrying the same content.
    #[must_use]
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "file": self.path.display().to_string(),
            "changed": self
                .delta
                .lines
                .iter()
                .map(|line| serde_json::json!({
                    "change": line.change.mark().to_string(),
                    "record": line.record.to_string(),
                    "handle": line.handle,
                    "detail": line.detail,
                }))
                .collect::<Vec<_>>(),
            "unchanged": self.delta.unchanged,
            "invariants": self
                .invariants
                .outcomes
                .iter()
                .map(|outcome| serde_json::json!({
                    "name": outcome.invariant.name(),
                    "passed": outcome.passed(),
                    "faults": outcome.faults,
                }))
                .collect::<Vec<_>>(),
            "reformatted": self.reformatted,
        })
    }
}

/// Why a mutation did not happen.
#[derive(Debug, thiserror::Error)]
pub enum MutationError {
    /// An invariant failed, so nothing was written.
    #[error("{0} did not hold after the change, so nothing was written")]
    Invariant(String),
    /// The file could not be written.
    #[error(transparent)]
    Write(#[from] WriteError),
    /// The snapshot cache could not be updated.
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
}

/// Where a mutation lands, and under what rules.
#[derive(Clone, Debug)]
pub struct Target<'a> {
    /// The file to write.
    pub path: &'a Path,
    /// The project directory, which holds the snapshot cache.
    pub project: &'a Path,
    /// The sheet path the file is being edited as.
    pub sheet_path: &'a SheetPath,
    /// The placement grid.
    pub grid: Iu,
    /// What kicli may and may not write.
    pub options: WriteOptions,
}

/// Apply a change to a document, check it, write it, and report it.
///
/// `before` is the state to compare against, taken before the change. `taken`
/// is the timestamp to record, supplied by the caller so that a test is
/// repeatable.
///
/// # Errors
///
/// Returns [`MutationError`] when an invariant fails, when the file cannot be
/// written, or when the snapshot cache cannot be updated. Nothing is written
/// unless every invariant holds.
pub fn commit(
    doc: &Doc,
    target: &Target<'_>,
    before: &Snapshot,
    taken: &str,
) -> Result<Mutation, MutationError> {
    let Target {
        path,
        project,
        sheet_path,
        grid,
        options,
    } = *target;
    let schematic =
        Schematic::read(doc).map_err(|error| MutationError::Invariant(error.to_string()))?;

    // Check before writing. A file is never left holding a change kicli knows
    // is wrong.
    let invariants = check_invariants(doc, &schematic, grid);
    if !invariants.passed() {
        let failed: Vec<String> = invariants
            .failures()
            .map(|outcome| format!("{}: {}", outcome.invariant, outcome.faults.join("; ")))
            .collect();
        return Err(MutationError::Invariant(failed.join(" | ")));
    }

    let after = Snapshot::take(LAST_WRITE, taken, sheet_path, doc, &schematic)?;
    let delta = Delta::between(before, &after);

    let written: Written = write_document(doc, path, options)?;
    after.write_in(project)?;

    Ok(Mutation {
        path: written.path,
        delta,
        invariants,
        reformatted: written.reformatted,
    })
}

/// The state to compare a change against: the file as it is now.
///
/// # Errors
///
/// Returns [`SnapshotError`] when the document cannot be snapshotted.
pub fn state_before(
    doc: &Doc,
    schematic: &Schematic,
    sheet_path: &SheetPath,
    taken: &str,
) -> Result<Snapshot, SnapshotError> {
    Snapshot::take(LAST_WRITE, taken, sheet_path, doc, schematic)
}
