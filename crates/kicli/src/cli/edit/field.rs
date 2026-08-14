//! The `field` noun: the text a symbol, sheet or label owns.
//!
//! Every object that owns fields answers to these verbs, which is what
//! "anything a human can move, kicli can move" requires. A symbol answers to
//! its reference designator and everything else to its identifier, because that
//! is what the views print for each.

use serde_json::json;

use crate::cli::args::{FieldVerb, Global, HorizontalArg, VerticalArg};
use crate::cli::edit::{Editing, address, code_for, report};
use crate::cli::exit::ExitCode;
use crate::cli::output::{Failure, Report};
use crate::edit::field::{
    FieldAddress, FieldError, Justification, hide, justify, locate, move_to, rotate_to, set_value,
    show,
};
use crate::model::items::Uuid;
use crate::model::mutate::Mutation;

/// Run one verb of the `field` noun.
///
/// # Errors
///
/// Returns a [`Failure`] carrying the row of the exit-code table the command
/// ended on.
pub fn run(global: &Global, verb: &FieldVerb) -> Result<Report, Failure> {
    match verb {
        FieldVerb::Move { owner, name, to } => {
            let to = to.point();
            change(global, owner, name, |doc, target, address, taken| {
                move_to(doc, target, address, to, taken)
            })
        }
        FieldVerb::Rotate { owner, name, to } => {
            let angle = to.angle();
            change(global, owner, name, |doc, target, address, taken| {
                rotate_to(doc, target, address, angle, taken)
            })
        }
        FieldVerb::Justify {
            owner,
            name,
            horizontal,
            vertical,
        } => align(global, owner, name, *horizontal, *vertical),
        FieldVerb::Show { owner, name } => {
            change(global, owner, name, |doc, target, address, taken| {
                show(doc, target, address, taken)
            })
        }
        FieldVerb::Hide { owner, name } => {
            change(global, owner, name, |doc, target, address, taken| {
                hide(doc, target, address, taken)
            })
        }
    }
}

/// Set the text of one field, which is what `sym set-field` does.
///
/// # Errors
///
/// Returns a [`Failure`] carrying the row of the exit-code table the command
/// ended on.
pub fn set_field(global: &Global, owner: &str, name: &str, value: &str) -> Result<Report, Failure> {
    change(global, owner, name, |doc, target, address, taken| {
        set_value(doc, target, address, value, taken)
    })
}

/// Set which part of a field's text sits at its position.
///
/// An axis the caller did not name keeps the alignment it has, so a command
/// that changes one axis does not quietly centre the other.
fn align(
    global: &Global,
    owner: &str,
    name: &str,
    horizontal: Option<HorizontalArg>,
    vertical: Option<VerticalArg>,
) -> Result<Report, Failure> {
    let mut editing = Editing::open(global)?;
    let owner_uuid = address::owner(editing.schematic(), editing.place.sheet_path(), owner)?;
    let field = FieldAddress {
        owner: owner_uuid.clone(),
        name: name.to_owned(),
    };
    let located = locate(editing.schematic(), &field).map_err(|error| refused(&error))?;
    let current = Justification::read(editing.doc(), located.property);
    let wanted = Justification {
        horizontal: horizontal.map_or(current.horizontal, HorizontalArg::alignment),
        vertical: vertical.map_or(current.vertical, VerticalArg::alignment),
    };

    let (doc, target, taken) = editing.parts();
    let mutation = justify(doc, &target, &field, wanted, taken).map_err(|error| refused(&error))?;
    Ok(reported(&mutation, &owner_uuid, name))
}

/// One change to one field, written and reported.
fn change(
    global: &Global,
    owner: &str,
    name: &str,
    edit: impl FnOnce(
        &mut kicli_sexpr::Doc,
        &crate::model::mutate::Target<'_>,
        &FieldAddress,
        &str,
    ) -> Result<Mutation, FieldError>,
) -> Result<Report, Failure> {
    let mut editing = Editing::open(global)?;
    let owner_uuid = address::owner(editing.schematic(), editing.place.sheet_path(), owner)?;
    let field = FieldAddress {
        owner: owner_uuid.clone(),
        name: name.to_owned(),
    };

    let (doc, target, taken) = editing.parts();
    let mutation = edit(doc, &target, &field, taken).map_err(|error| refused(&error))?;
    Ok(reported(&mutation, &owner_uuid, name))
}

/// The report of one field command.
fn reported(mutation: &Mutation, owner: &Uuid, name: &str) -> Report {
    report(
        mutation,
        Some(("field", json!({ "owner": owner.0, "name": name }))),
        &[],
    )
}

/// Which row of the table a refused field command is.
fn refused(error: &FieldError) -> Failure {
    let code = match error {
        FieldError::Write(inner) => code_for(inner),
        FieldError::Read(_) | FieldError::Snapshot(_) => ExitCode::File,
        _ => ExitCode::Operation,
    };
    Failure::new(code, error.to_string())
}
