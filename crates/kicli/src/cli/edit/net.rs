//! The `net` noun: an edit to a whole net.
//!
//! kicli does not write the project file, so a net's name is only what its
//! labels say. Renaming a net renames every one of them, across every sheet the
//! net reaches, in one pass that checks all the files before it writes any. A
//! net with no label has no name to change, and the refusal points at
//! `label add` rather than leaving a caller guessing.

use crate::cli::args::{Global, NetVerb};
use crate::cli::edit::{Editing, code_for, code_for_refusal, code_for_snapshot};
use crate::cli::exit::ExitCode;
use crate::cli::output::{Failure, Report};
use crate::edit::net::{NetError, Scope, rename};

/// Run one verb of the `net` noun.
///
/// # Errors
///
/// Returns a [`Failure`] carrying the row of the exit-code table the command
/// ended on. A net with no label is [`ExitCode::Operation`] and writes nothing.
pub fn run(global: &Global, verb: &NetVerb) -> Result<Report, Failure> {
    match verb {
        NetVerb::Rename { from, to } => {
            let mut editing = Editing::open(global)?;
            let project = editing.loaded.directory.clone();
            let scope = Scope {
                project: &project,
                grid: editing.place.grid(),
                options: editing.place.options(),
            };
            let taken = editing.taken.clone();
            let renamed = rename(&mut editing.loaded.hierarchy, from, to, &scope, &taken)
                .map_err(|error| refused(&error))?;
            Ok(Report {
                text: renamed.render(),
                json: renamed.to_json(),
            })
        }
    }
}

/// Which row of the table a refused rename is.
fn refused(error: &NetError) -> Failure {
    let code = match error {
        NetError::Mutation(inner) => code_for(inner),
        NetError::Snapshot(inner) => code_for_snapshot(inner),
        NetError::Refused(inner) => code_for_refusal(inner),
        NetError::WouldNotHold { .. } => ExitCode::Verification,
        NetError::Read(_) => ExitCode::File,
        _ => ExitCode::Operation,
    };
    Failure::new(code, error.to_string())
}
