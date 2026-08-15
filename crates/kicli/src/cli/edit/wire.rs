//! The `wire` noun: the verbs that draw and remove the segments joining pins.
//!
//! `wire draw` takes the corners and does no searching. `wire delete` removes
//! one segment and nothing else. Both write through [`crate::edit::wire`],
//! which is the only path to disk, and both report what changed the way every
//! other mutating command does.
//!
//! A drawn wire also answers in the router's own output contract, which
//! [`contract`] renders: the status, the cost in parts, what the route crossed
//! and what it added. That answer is what an agent decides on, so it is printed
//! first and carried under the noun's key in JSON.
//!
//! **One resolver names the segment a delete removes.**
//! [`crate::cli::edit::address`] already turns what a caller typed into the
//! object it names — the whole identifier, or a prefix of at least eight
//! characters, refusing an ambiguous one with the matches listed. The verb
//! resolves the text here and hands the library an identifier that names
//! exactly one segment. [`crate::edit::wire::delete`] states the same rule
//! again for a caller that did not come through the command line; on this path
//! it never fires.

pub mod contract;

use serde_json::json;

use crate::cli::args::{DrawArgs, Global, WireVerb};
use crate::cli::edit::{Editing, Note, code_for, code_for_snapshot, report};
use crate::cli::exit::ExitCode;
use crate::cli::output::{Failure, Report};
use crate::edit::wire::{DeletedWire, DrawnWire, Polyline, Stranded, WireError, delete, draw};

use super::address;

/// Run one verb of the `wire` noun.
///
/// # Errors
///
/// Returns a [`Failure`] carrying the row of the exit-code table the command
/// ended on. A request no drawing can hold and a way that is barred are both
/// [`ExitCode::Operation`], and neither writes a byte.
pub fn run(global: &Global, verb: &WireVerb) -> Result<Report, Failure> {
    match verb {
        WireVerb::Draw(args) => draw_wire(global, args),
        WireVerb::Delete { target } => delete_wire(global, target),
    }
}

/// Draw a wire through the corners the caller gave.
fn draw_wire(global: &Global, args: &DrawArgs) -> Result<Report, Failure> {
    let request = Polyline {
        from: args
            .start()
            .map_err(|why| Failure::new(ExitCode::Usage, why))?,
        to: args
            .finish()
            .map_err(|why| Failure::new(ExitCode::Usage, why))?,
        via: args.corners(),
    };

    let mut editing = Editing::open(global)?;
    let routing = editing.loaded.config.routing;
    let grid = editing.place.grid();
    let (hierarchy, target, taken) = editing.tree_parts();
    let DrawnWire {
        report: route,
        mutation,
    } = draw(hierarchy, &request, &routing, &target, taken).map_err(|error| refused(&error))?;

    // The route's own answer comes first: it is what the caller asked for, and
    // the delta below it says what reaching that answer changed in the file.
    let rendered = contract::render(&route, grid);
    let mut result = report(&mutation, Some(("wire", rendered.json)), &[]);
    result.text = format!("{}{}", rendered.text, result.text);
    Ok(result)
}

/// Delete the one segment the caller named.
fn delete_wire(global: &Global, target: &str) -> Result<Report, Failure> {
    let mut editing = Editing::open(global)?;
    let named = address::wire(editing.schematic(), target)?;

    let (hierarchy, target, taken) = editing.tree_parts();
    let DeletedWire {
        removed: uuid,
        from,
        to,
        stranded,
        mutation,
    } = delete(hierarchy, &named.0, &target, taken).map_err(|error| refused(&error))?;
    let notes: Vec<Note> = stranded.iter().map(stranded_note).collect();
    Ok(report(
        &mutation,
        Some((
            "wire",
            json!({
                "uuid": uuid.0,
                "from": from.to_string(),
                "to": to.to_string(),
                "stranded": stranded
                    .iter()
                    .map(|left| json!({
                        "junction": left.junction.0,
                        "at": left.at.to_string(),
                        "joins": left.ends.len(),
                    }))
                    .collect::<Vec<serde_json::Value>>(),
            }),
        )),
        &notes,
    ))
}

/// What a caller must feel about a junction the delete left behind.
///
/// The junction is still in the file. Removing it is a second decision, and it
/// belongs to whoever asks for it.
fn stranded_note(left: &Stranded) -> Note {
    Note::new(
        "stranded-junction",
        format!(
            "the junction {} at ({}) now joins {} wire end(s), and is still there. \
             Run junction delete --at {} to take it away.",
            handle(&left.junction.0),
            left.at,
            left.ends.len(),
            left.at
        ),
    )
}

/// The first characters of an identifier, as a view prints them.
fn handle(uuid: &str) -> &str {
    uuid.get(..8).unwrap_or(uuid)
}

/// Which row of the table a refused wire command is.
///
/// The shape of the request decides: a request no drawing can hold and a way
/// that is barred are both well-formed requests kicli could not complete, and
/// [`ExitCode::for_route`] is the one place that turns the status into a row.
/// A file that will not read or write is the file's row instead, because the
/// request was never the problem.
fn refused(error: &WireError) -> Failure {
    let code = match error {
        WireError::Mutation(inner) => code_for(inner),
        WireError::Snapshot(inner) => code_for_snapshot(inner),
        WireError::Read(_) | WireError::Sexpr(_) | WireError::Empty => ExitCode::File,
        WireError::UnknownFile { .. } => ExitCode::Usage,
        other => ExitCode::for_route(other.status()),
    };
    Failure::new(code, error.to_string())
}

#[cfg(test)]
mod tests {
    use super::{handle, refused};
    use crate::cli::exit::ExitCode;
    use crate::edit::wire::WireError;
    use crate::geometry::Point;

    #[test]
    fn a_refusal_takes_the_row_its_status_names() {
        // Both kinds of refusal are the same row, and they get there through
        // the status rather than through a second table.
        let blocked = WireError::Blocked {
            handle: "U1".to_owned(),
            at: Point::new(0, 0),
            from: Point::new(0, 0),
        };
        assert_eq!(refused(&blocked).code, ExitCode::Operation);
        let invalid = WireError::OffGrid {
            at: Point::new(1, 1),
        };
        assert_eq!(refused(&invalid).code, ExitCode::Operation);
        // A file kicli cannot use is the file's row, not the request's.
        assert_eq!(refused(&WireError::Empty).code, ExitCode::File);
    }

    #[test]
    fn a_handle_is_the_first_characters_a_view_prints() {
        assert_eq!(handle("da5aa983-0000-4000-8000-000000000001"), "da5aa983");
        assert_eq!(handle("short"), "short");
    }
}
