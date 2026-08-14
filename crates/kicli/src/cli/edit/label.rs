//! The `label` noun: the objects that name a net.
//!
//! A label anchor is connectable geometry, so every command here snaps it to
//! the grid. Adding one is therefore a change to the netlist as well as to the
//! drawing, and the report names the net the label joined or made.

use crate::cli::args::{Global, LabelVerb};
use crate::cli::edit::{Editing, address, code_for, code_for_snapshot};
use crate::cli::exit::ExitCode;
use crate::cli::output::{Failure, Report};
use crate::edit::label::{LabelChange, LabelError, NewLabel, add, delete, move_to};

/// Run one verb of the `label` noun.
///
/// # Errors
///
/// Returns a [`Failure`] carrying the row of the exit-code table the command
/// ended on.
pub fn run(global: &Global, verb: &LabelVerb) -> Result<Report, Failure> {
    match verb {
        LabelVerb::Add {
            text,
            at,
            kind,
            angle,
            shape,
        } => {
            let request = NewLabel {
                kind: kind.kind(),
                text: text.clone(),
                at: at.point(),
                angle: angle.angle(),
                shape: shape.shape(),
            };
            let mut editing = Editing::open(global)?;
            let root = editing.root();
            let (doc, target, taken) = editing.parts();
            let change =
                add(doc, &target, &root, &request, taken).map_err(|error| refused(&error))?;
            Ok(reported(&change))
        }

        LabelVerb::Move { target, to } => {
            let to = to.point();
            let mut editing = Editing::open(global)?;
            let uuid = address::uuid(editing.schematic(), target)?;
            let root = editing.root();
            let (doc, where_to, taken) = editing.parts();
            let change = move_to(doc, &where_to, &root, &uuid, to, taken)
                .map_err(|error| refused(&error))?;
            Ok(reported(&change))
        }

        LabelVerb::Delete { target } => {
            let mut editing = Editing::open(global)?;
            let uuid = address::uuid(editing.schematic(), target)?;
            let root = editing.root();
            let (doc, where_to, taken) = editing.parts();
            let change =
                delete(doc, &where_to, &root, &uuid, taken).map_err(|error| refused(&error))?;
            Ok(reported(&change))
        }
    }
}

/// The report of one label command, which already carries its own notes.
fn reported(change: &LabelChange) -> Report {
    Report {
        text: change.render(),
        json: change.to_json(),
    }
}

/// Which row of the table a refused label command is.
fn refused(error: &LabelError) -> Failure {
    let code = match error {
        LabelError::Mutation(inner) => code_for(inner),
        LabelError::Snapshot(inner) => code_for_snapshot(inner),
        LabelError::Read(_) | LabelError::Load(_) => ExitCode::File,
        _ => ExitCode::Operation,
    };
    Failure::new(code, error.to_string())
}
