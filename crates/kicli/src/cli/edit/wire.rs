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
use std::fmt::Write as _;
use std::path::Path;

use crate::cli::args::{ConnectArgs, DrawArgs, Global, WireVerb};
use crate::cli::edit::{Editing, Note, code_for, code_for_snapshot, report};
use crate::cli::exit::ExitCode;
use crate::cli::output::{Failure, Report};
use crate::connectivity::extract;
use crate::edit::label::{LabelError, NewLabel, PortShape, add as add_label};
use crate::edit::wire::{
    Connection, DeletedWire, DrawnWire, End, Plan, Planned, Polyline, Stranded, WireError, delete,
    draw, draw_plan, plan,
};
use crate::geometry::pins::ResolvedPin;
use crate::geometry::{Angle, Iu};
use crate::model::config::Routing;
use crate::model::items::{LabelKind, SheetPath};
use crate::model::{Hierarchy, LoadedFile};
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
        WireVerb::Connect(args) => connect_wire(global, args),
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

/// Route a connection between two ends, and draw it.
///
/// Three answers, and only one of them writes a wire. A route short enough to
/// draw is drawn, with the junctions it needs. A route longer than
/// `routing.label_threshold`, and a pair of ends nothing routes between, are
/// **proposed** as a pair of labels — reported and not written, unless
/// `--auto-labels` says to perform the proposal. A pair of ends that cannot be
/// resolved is refused before anything is planned.
///
/// **The net is read back after the write, never predicted.** What the two ends
/// are joined into is a property of the file kicli has just written, so it is
/// taken from the written file rather than from the arithmetic that produced
/// it.
fn connect_wire(global: &Global, args: &ConnectArgs) -> Result<Report, Failure> {
    let request = Connection {
        from: args
            .start()
            .map_err(|why| Failure::new(ExitCode::Usage, why))?,
        to: args
            .finish()
            .map_err(|why| Failure::new(ExitCode::Usage, why))?,
    };

    let mut editing = Editing::open(global)?;
    let routing = editing.loaded.config.routing;
    let grid = editing.place.grid();
    let root = editing.root();

    let planned = {
        let (hierarchy, target, _) = editing.tree_parts();
        plan(hierarchy, &request, &routing, &target).map_err(|error| refused(&error))?
    };

    if let Some(proposed) = proposed_for(&editing, &request, &planned, &routing, grid) {
        if !args.auto_labels {
            // A proposal is an answer rather than a failure, so it succeeds and
            // writes nothing. The caller decides whether to perform it.
            let mut route = proposed.proposal.report(&proposed.source, &proposed.target);
            route.blocked_by.clone_from(&planned.blocked_by);
            return Ok(unwritten(
                &contract::render(&route, grid),
                &[proposal_note()],
            ));
        }
        // A proposal is performed on its two ends alone: the labels sit on
        // stubs from the ends, and the path between them is exactly what the
        // proposal declines to draw.
        let ends = Polyline {
            from: request.from.clone(),
            to: request.to.clone(),
            via: Vec::new(),
        };
        let performed = perform(&mut editing, &ends, &proposed, &routing, grid)?;
        return Ok(joined(performed, &root, &request));
    }

    let drawn = {
        let (hierarchy, target, taken) = editing.tree_parts();
        draw_plan(hierarchy, &planned, &routing, &target, taken).map_err(|error| refused(&error))?
    };
    let rendered = contract::render(&drawn.report, grid);
    let mut result = report(&drawn.mutation, Some(("wire", rendered.json)), &[]);
    result.text = format!("{}{}", rendered.text, result.text);
    Ok(joined(result, &root, &request))
}

/// Is this planned connection one kicli proposes as a pair of labels?
///
/// The length judged is the route the router found, and nothing when it found
/// none — which `research/wire-routing.md` §5.5 makes the second trigger. The
/// pair needs a name, so a connection between two ends that name no pin is
/// never proposed: there is nothing to call it, and a connection kicli cannot
/// name is one it draws or refuses rather than one it proposes badly.
fn proposed_for(
    editing: &Editing,
    request: &Connection,
    planned: &Plan,
    routing: &Routing,
    grid: Iu,
) -> Option<Proposed> {
    let file = &editing.loaded.hierarchy.files[editing.file];
    let sheet = editing.place.sheet_path();
    let source_pin = end_terminal(file, sheet, &request.from).and_then(|(_, pin)| pin);
    let target_pin = end_terminal(file, sheet, &request.to).and_then(|(_, pin)| pin);
    let name = name_for(editing, &request.from, source_pin.as_ref())
        .or_else(|| name_for(editing, &request.to, target_pin.as_ref()))?;

    Proposal::of(
        &planned.source,
        &planned.target,
        planned.route.as_ref().map(Planned::length),
        &name,
        routing,
        grid,
    )
    .map(|proposal| Proposed {
        proposal,
        source: planned.source.clone(),
        target: planned.target.clone(),
    })
}

/// What a caller must feel about a proposal kicli did not perform.
fn proposal_note() -> Note {
    Note::new(
        "proposal",
        "kicli drew nothing. Run the same command with --auto-labels to write \
         the pair, or choose the path yourself with wire draw.",
    )
}

/// The result of a command that answered without writing anything.
///
/// It carries the same keys a written answer does, minus the mutation's own:
/// the noun's key, the notes, and the net — null, because nothing was joined.
/// A caller parses one shape whichever answer it got.
fn unwritten(rendered: &Report, notes: &[Note]) -> Report {
    let mut text = rendered.text.clone();
    for note in notes {
        let _ = writeln!(text, "note: {}  {}", note.name, note.message);
    }
    Report {
        text,
        json: with_net(
            json!({
                "wire": rendered.json.clone(),
                "notes": notes
                    .iter()
                    .map(|note| json!({ "name": note.name, "message": note.message }))
                    .collect::<Vec<serde_json::Value>>(),
            }),
            None,
        ),
    }
}

/// The net the two ends are on now, added to a result that wrote something.
///
/// Read from the file on disk rather than from the tree in memory, because the
/// claim is about what the drawing now says. A connection between two ends that
/// name no pin has no net to report and says so with a null rather than by
/// leaving the key out, so one parse covers both.
fn joined(mut result: Report, root: &Path, request: &Connection) -> Report {
    let net = joined_net(root, request);
    if let Some(name) = &net {
        result.text = format!("joined: net {name}\n{}", result.text);
    }
    result.json = with_net(result.json, net);
    result
}

/// The net an end of the connection is on, read back from the written file.
fn joined_net(root: &Path, request: &Connection) -> Option<String> {
    let hierarchy = Hierarchy::load(root).ok()?;
    let nets = extract(&hierarchy);
    [&request.from, &request.to]
        .into_iter()
        .filter_map(|end| match end {
            End::Pin(pin) => Some(pin),
            _ => None,
        })
        .find_map(|pin| {
            nets.net_of(&pin.reference.0, &pin.number)
                .map(|net| net.name.clone())
        })
}

/// One result, with the net it joined beside it.
///
/// The key is a sibling of the noun's own rather than a field inside the route
/// contract: `crate::route::report` is frozen, and the shape
/// [`contract`] renders is that contract. What net a connection produced is the
/// command's answer, not the router's.
fn with_net(mut json: serde_json::Value, net: Option<String>) -> serde_json::Value {
    if let Some(fields) = json.as_object_mut() {
        fields.insert(
            "net".to_owned(),
            net.map_or(serde_json::Value::Null, serde_json::Value::String),
        );
    }
    json
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
