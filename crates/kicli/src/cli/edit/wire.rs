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
use crate::connectivity::extract;
use crate::edit::label::{LabelError, NewLabel, PortShape, add as add_label};
use crate::edit::wire::{DeletedWire, DrawnWire, End, Polyline, Stranded, WireError, delete, draw};
use crate::geometry::pins::ResolvedPin;
use crate::geometry::{Angle, Iu};
use crate::model::LoadedFile;
use crate::model::config::Routing;
use crate::model::items::{LabelKind, SheetPath};
use crate::route::propose::{Proposal, label_name, walked};
use crate::route::report::Added;
use crate::route::{Terminal, pin_terminal};

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

    // The proposal comes first, because performing it draws no wire at all.
    // Without the flag nothing here runs and the verb draws what it was asked
    // for: a connection kicli would rather see labelled is still a connection
    // an agent may draw.
    if args.auto_labels {
        if let Some(proposed) = proposed(&editing, &request, &routing, grid) {
            return perform(&mut editing, &request, &proposed, &routing, grid);
        }
    }

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

/// A connection kicli proposes as a pair of labels, and the ends it joins.
struct Proposed {
    /// The pair, and why it is proposed.
    proposal: Proposal,
    /// The end the connection leaves.
    source: Terminal,
    /// The end it reaches.
    target: Terminal,
}

/// Is this request one kicli proposes as a pair of labels?
///
/// Nothing, when the connection is short enough to draw — and nothing when an
/// end cannot be resolved or names no pin. An end that does not resolve is a
/// refusal [`draw`] states far better than this could, so the request goes on
/// to it; and a pair of ends with no pin between them gives no name to fall
/// back to when the drawing names no net.
///
/// The length judged is the vertices the caller gave. Two ends and no corners
/// measure the Manhattan distance between them, which no orthogonal route can
/// beat, so a request that names no path is still judged against the shortest
/// wire that could join it.
fn proposed(
    editing: &Editing,
    request: &Polyline,
    routing: &Routing,
    grid: Iu,
) -> Option<Proposed> {
    let file = &editing.loaded.hierarchy.files[editing.file];
    let sheet = editing.place.sheet_path();
    let (source, source_pin) = end_terminal(file, sheet, &request.from)?;
    let (target, target_pin) = end_terminal(file, sheet, &request.to)?;

    let mut vertices = Vec::with_capacity(request.via.len() + 2);
    vertices.push(source.at);
    vertices.extend(request.via.iter().copied());
    vertices.push(target.at);

    let name = name_for(editing, &request.from, source_pin.as_ref())
        .or_else(|| name_for(editing, &request.to, target_pin.as_ref()))?;
    Proposal::of(
        &source,
        &target,
        Some(walked(&vertices)),
        &name,
        routing,
        grid,
    )
    .map(|proposal| Proposed {
        proposal,
        source,
        target,
    })
}

/// The terminal one end of a request names, with the pin when it names one.
///
/// The pin comes back because a proposed label is named after it when the
/// drawing names no net. [`crate::edit::wire`] resolves the same three forms
/// for the wire it draws; the pin arm is the one with a rule in it, and both
/// paths read it from [`pin_terminal`].
fn end_terminal(
    loaded: &LoadedFile,
    sheet: &SheetPath,
    end: &End,
) -> Option<(Terminal, Option<ResolvedPin>)> {
    match end {
        End::At(at) => Some((Terminal::of_point(*at, &at.to_string()), None)),
        End::Port(name) => loaded
            .schematic
            .sheets()
            .flat_map(|child| &child.pins)
            .find(|port| port.name == *name)
            .map(|port| (Terminal::of_sheet_pin(port), None)),
        End::Pin(pin) => pin_terminal(loaded, sheet, &pin.reference.0, &pin.number)
            .map(|(terminal, resolved)| (terminal, Some(resolved))),
    }
}

/// What to call the pair, from the end a name can be taken from.
///
/// The net's own name when the drawing gives the pin one, and
/// `<reference>_<pin name>` when it does not. A synthetic name is one kicli
/// invented for a net the drawing does not name, so it is not a name to label
/// with — it would freeze a handle that renumbers when another net gains a
/// label.
fn name_for(editing: &Editing, end: &End, pin: Option<&ResolvedPin>) -> Option<String> {
    let (End::Pin(address), Some(resolved)) = (end, pin) else {
        return None;
    };
    let nets = extract(&editing.loaded.hierarchy);
    let named = nets
        .net_of(&address.reference.0, &address.number)
        .filter(|net| !net.synthetic)
        .map(|net| net.name.clone());
    Some(label_name(
        named.as_deref(),
        &address.reference.0,
        &resolved.name,
        &resolved.number,
    ))
}

/// Write the pair of labels the router proposed, and no wire between them.
///
/// Each label sits [`crate::route::propose::REACH`] grid steps along its own
/// pin's direction, on a stub drawn from the pin to it. The stub is what makes
/// the label electrically the pin's: a label standing two grid steps off a pin
/// with nothing between them names a net the pin is not on.
///
/// Every write goes through the mutation path and is verified on its own. The
/// result reports **one** delta over all of them, taken against the state
/// before the first, because "what did this command do" has one answer.
fn perform(
    editing: &mut Editing,
    request: &Polyline,
    proposed: &Proposed,
    routing: &Routing,
    grid: Iu,
) -> Result<Report, Failure> {
    let before = editing
        .state()
        .map_err(|why| Failure::new(code_for_snapshot(&why), why.to_string()))?;
    let pair = &proposed.proposal.labels;
    let ends = [
        (&request.from, &proposed.source, pair.at[0]),
        (&request.to, &proposed.target, pair.at[1]),
    ];

    let mut wires = Vec::new();
    for (end, terminal, at) in ends {
        // An end that fixes no direction is its own anchor, and a stub from a
        // point to itself is no wire at all.
        if at == terminal.at {
            continue;
        }
        let stub = Polyline {
            from: end.clone(),
            to: End::At(at),
            via: Vec::new(),
        };
        let (hierarchy, target, taken) = editing.tree_parts();
        let DrawnWire { report: stub, .. } =
            draw(hierarchy, &stub, routing, &target, taken).map_err(|error| refused(&error))?;
        wires.extend(stub.added.wires);
    }

    for at in pair.at {
        let request = NewLabel {
            kind: LabelKind::Local,
            text: pair.name.clone(),
            at,
            angle: Angle(0),
            shape: PortShape::default(),
        };
        let root = editing.root();
        let (doc, target, taken) = editing.parts();
        add_label(doc, &target, &root, &request, taken).map_err(|error| label_refused(&error))?;
    }

    let mutation = editing
        .commit(&before)
        .map_err(|why| Failure::new(code_for(&why), why.to_string()))?;
    let mut route = proposed.proposal.report(&proposed.source, &proposed.target);
    route.added = Added {
        wires,
        junctions: Vec::new(),
    };

    let rendered = contract::render(&route, grid);
    let notes = vec![Note::new(
        "auto-labels",
        format!(
            "kicli wrote the label {:?} at each end instead of a wire. \
             Each label sits on a short stub from its own pin. \
             Nothing joins the two ends but the name they share.",
            pair.name
        ),
    )];
    let mut result = report(&mutation, Some(("wire", rendered.json)), &notes);
    result.text = format!("{}{}", rendered.text, result.text);
    Ok(result)
}

/// Which row of the table a refused label write is.
///
/// The same reading as the `label` noun's own, because it is the same library
/// call. Two homes for one mapping is one more than the rule wants; the noun's
/// is private to its module, so this states it rather than reaching into it.
fn label_refused(error: &LabelError) -> Failure {
    let code = match error {
        LabelError::Mutation(inner) => code_for(inner),
        LabelError::Snapshot(inner) => code_for_snapshot(inner),
        LabelError::Read(_) | LabelError::Load(_) => ExitCode::File,
        _ => ExitCode::Operation,
    };
    Failure::new(code, error.to_string())
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
            left.junction.short(),
            left.at,
            left.ends.len(),
            left.at
        ),
    )
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
    use super::refused;
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
}
