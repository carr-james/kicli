//! The `text` noun: free text and text boxes.
//!
//! Graphic text is a drawing and not a conductor, so nothing here snaps to the
//! grid. `edit` changes what the text says, the size of its box, or both, and a
//! request that names neither is a usage error rather than a write that does
//! nothing.

use serde_json::json;

use crate::cli::args::{Global, TextVerb};
use crate::cli::edit::{Editing, address, code_for, report};
use crate::cli::exit::ExitCode;
use crate::cli::output::{Failure, Report};
use crate::edit::text::{NewText, TextError, add, delete, edit, move_to, resize};
use crate::model::items::Uuid;
use crate::model::mutate::Mutation;

/// Run one verb of the `text` noun.
///
/// # Errors
///
/// Returns a [`Failure`] carrying the row of the exit-code table the command
/// ended on.
pub fn run(global: &Global, verb: &TextVerb) -> Result<Report, Failure> {
    match verb {
        TextVerb::Add {
            text,
            at,
            angle,
            size,
        } => {
            let request = NewText {
                text: text.clone(),
                at: at.point(),
                angle: angle.angle(),
                size: size.map(crate::cli::args::SizeArg::size),
            };
            let mut editing = Editing::open(global)?;
            let (doc, target, taken) = editing.parts();
            let change = add(doc, &target, &request, taken).map_err(|error| refused(&error))?;
            Ok(reported(&change.mutation, &change.uuid))
        }

        TextVerb::Move { target, to } => {
            let to = to.point();
            change(global, target, |doc, where_to, uuid, taken| {
                move_to(doc, where_to, uuid, to, taken)
            })
        }

        TextVerb::Delete { target } => change(global, target, |doc, where_to, uuid, taken| {
            delete(doc, where_to, uuid, taken)
        }),

        // The argument parser takes exactly one of the two, because one command
        // is one write and one delta.
        TextVerb::Edit {
            target,
            text: Some(text),
            ..
        } => change(global, target, |doc, where_to, uuid, taken| {
            edit(doc, where_to, uuid, text, taken)
        }),

        TextVerb::Edit {
            target,
            size: Some(size),
            ..
        } => {
            let size = size.size();
            change(global, target, |doc, where_to, uuid, taken| {
                resize(doc, where_to, uuid, size, taken)
            })
        }

        TextVerb::Edit { .. } => Err(Failure::new(
            ExitCode::Usage,
            "an edit needs --text or --size.",
        )),
    }
}

/// One change to one existing text object, written and reported.
fn change(
    global: &Global,
    target: &str,
    apply: impl FnOnce(
        &mut kicli_sexpr::Doc,
        &crate::model::mutate::Target<'_>,
        &Uuid,
        &str,
    ) -> Result<crate::edit::text::TextChange, TextError>,
) -> Result<Report, Failure> {
    let mut editing = Editing::open(global)?;
    let uuid = address::uuid(editing.schematic(), target)?;

    let (doc, where_to, taken) = editing.parts();
    let change = apply(doc, &where_to, &uuid, taken).map_err(|error| refused(&error))?;
    Ok(reported(&change.mutation, &change.uuid))
}

/// The report of one text command.
fn reported(mutation: &Mutation, uuid: &Uuid) -> Report {
    report(mutation, Some(("text", json!({ "uuid": uuid.0 }))), &[])
}

/// Which row of the table a refused text command is.
fn refused(error: &TextError) -> Failure {
    let code = match error {
        TextError::Mutation(inner) => code_for(inner),
        TextError::Read(_) | TextError::Snapshot(_) => ExitCode::File,
        _ => ExitCode::Operation,
    };
    Failure::new(code, error.to_string())
}
