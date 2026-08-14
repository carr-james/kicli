//! The `junction` and `noconnect` nouns: the marks that decide what is joined.
//!
//! Both marks are one point in the file and both change what the netlist says.
//! Both carry a refusal, and a refusal writes no byte: a junction where four
//! wire ends already meet, and a no-connect on a pin something already joins.
//!
//! A junction is addressed by a point or by a pin. The pin form exists because
//! "join this pin to the wire it sits on" is the common request, and working the
//! point out by hand is where a caller makes an arithmetic mistake.

use serde_json::json;

use crate::cli::args::{Global, JunctionVerb, NoconnectVerb, PinArg, PointArg};
use crate::cli::edit::{Editing, code_for, code_for_snapshot, report};
use crate::cli::exit::ExitCode;
use crate::cli::output::{Failure, Report};
use crate::edit::insert::fresh_uuid;
use crate::edit::mark::{
    MarkError, add_junction, add_no_connect, delete_junction, delete_no_connect, pin_point,
};
use crate::geometry::Point;
use crate::model::items::Uuid;

/// Run one verb of the `junction` noun.
///
/// # Errors
///
/// Returns a [`Failure`] carrying the row of the exit-code table the command
/// ended on. A four-way junction is [`ExitCode::Operation`] and writes nothing.
pub fn junction(global: &Global, verb: &JunctionVerb) -> Result<Report, Failure> {
    match verb {
        JunctionVerb::Add { at, pin } => {
            let mut editing = Editing::open(global)?;
            let point = point_of(&mut editing, at.as_ref(), pin.as_ref())?;
            let uuid = fresh(&editing, &format!("junction {point}"));
            let (hierarchy, target, taken) = editing.tree_parts();
            let mutation = add_junction(hierarchy, point, &uuid, &target, taken)
                .map_err(|error| refused(&error))?;
            Ok(report(
                &mutation,
                Some((
                    "junction",
                    json!({ "uuid": uuid.0, "at": point.to_string() }),
                )),
                &[],
            ))
        }

        JunctionVerb::Delete { at, pin } => {
            let mut editing = Editing::open(global)?;
            let point = point_of(&mut editing, at.as_ref(), pin.as_ref())?;
            let (hierarchy, target, taken) = editing.tree_parts();
            let mutation = delete_junction(hierarchy, point, &target, taken)
                .map_err(|error| refused(&error))?;
            Ok(report(
                &mutation,
                Some(("junction", json!({ "at": point.to_string() }))),
                &[],
            ))
        }
    }
}

/// Run one verb of the `noconnect` noun.
///
/// # Errors
///
/// Returns a [`Failure`] carrying the row of the exit-code table the command
/// ended on. A no-connect on a connected pin is [`ExitCode::Operation`] and
/// writes nothing.
pub fn no_connect(global: &Global, verb: &NoconnectVerb) -> Result<Report, Failure> {
    match verb {
        NoconnectVerb::Add { pin } => {
            let mut editing = Editing::open(global)?;
            let address = pin.address();
            let uuid = fresh(&editing, &format!("no-connect {address}"));
            let (hierarchy, target, taken) = editing.tree_parts();
            let mutation = add_no_connect(hierarchy, &address, &uuid, &target, taken)
                .map_err(|error| refused(&error))?;
            Ok(report(
                &mutation,
                Some((
                    "no_connect",
                    json!({ "uuid": uuid.0, "pin": address.to_string() }),
                )),
                &[],
            ))
        }

        NoconnectVerb::Delete { pin } => {
            let mut editing = Editing::open(global)?;
            let address = pin.address();
            let (hierarchy, target, taken) = editing.tree_parts();
            let mutation = delete_no_connect(hierarchy, &address, &target, taken)
                .map_err(|error| refused(&error))?;
            Ok(report(
                &mutation,
                Some(("no_connect", json!({ "pin": address.to_string() }))),
                &[],
            ))
        }
    }
}

/// The point a junction command works on, given either form of address.
fn point_of(
    editing: &mut Editing,
    at: Option<&PointArg>,
    pin: Option<&PinArg>,
) -> Result<Point, Failure> {
    if let Some(point) = at {
        return Ok(point.point());
    }
    let Some(pin) = pin else {
        // The argument parser requires one of the two.
        return Err(Failure::new(
            ExitCode::Usage,
            "a junction needs --at or --pin.",
        ));
    };
    let address = pin.address();
    let (hierarchy, target, _) = editing.tree_parts();
    pin_point(hierarchy, &target, &address).map_err(|error| refused(&error))
}

/// An identifier for the mark this command makes.
fn fresh(editing: &Editing, seed: &str) -> Uuid {
    fresh_uuid(
        &editing.loaded.hierarchy.files[editing.file].doc,
        &format!("{seed} {}", editing.taken),
    )
}

/// Which row of the table a refused mark command is.
fn refused(error: &MarkError) -> Failure {
    let code = match error {
        MarkError::Mutation(inner) => code_for(inner),
        MarkError::Snapshot(inner) => code_for_snapshot(inner),
        MarkError::Read(_) => ExitCode::File,
        _ => ExitCode::Operation,
    };
    Failure::new(code, error.to_string())
}
