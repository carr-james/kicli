//! Wires drawn through the vertices a caller states, with no search.
//!
//! The caller gives the corners. kicli decides whether they are drawable and
//! refuses rather than drawing something illegal. Four rules decide, and every
//! refusal names the vertex it is about:
//!
//! - every vertex sits on the placement grid;
//! - every segment runs along one axis;
//! - no segment passes through anything a wire may not cross;
//! - the wire leaves each end the way that end must be left.
//!
//! The last is the escape rule, which is a hard constraint and not a cost. A
//! wire that approaches a pin from the side reads wrong, and on a pin whose
//! graphic carries a marker it draws over the marker. For a **port** the same
//! rule runs off the edge the port sits on, which
//! [`Terminal::of_sheet_pin`](crate::route::Terminal::of_sheet_pin) records as
//! measured against KiCad rather than read from its source.
//!
//! **A vertex off the grid is refused rather than snapped.** Every other verb
//! snaps a position it is given, because a symbol an agent asked for at an
//! approximate point belongs at the nearest grid point. A polyline is not a
//! position: moving one corner changes the shape of the wire the caller asked
//! for, and can turn a legal path into one that runs along another net. The
//! router already refuses an off-grid terminal for the same reason, rather
//! than moving somebody's pin.
//!
//! **A drawn wire owns none of the sheet's existing wires.** The caller chose
//! the path, so no wire on the sheet is this route's own: another net's wire
//! blocks along its own axis and is costed across it. That is why a request
//! here needs no net extraction, and why a reported crossing names the wire it
//! crossed and leaves the net unattributed — whose a wire is, is connectivity's
//! answer and not geometry's.
//!
//! Every segment becomes one two-point `wire` record, because a KiCad wire is
//! always two points. The write goes through [`crate::model::mutate`], which is
//! the only path to disk: the invariants run on what was built, and a file is
//! written only when all of them hold.

use std::path::Path;

use kicli_sexpr::{SexprError, quote};

use crate::edit::insert::{Identifiers, insertion_index};
use crate::edit::mark::PinAddress;
use crate::geometry::{Iu, Point, on_segment, resolve_pins};
use crate::model::config::Routing;
use crate::model::hierarchy::{Hierarchy, LoadedFile};
use crate::model::items::{ReadError, Schematic, SheetPath, Uuid};
use crate::model::library::{definition_of, read_library};
use crate::model::mutate::{Mutation, MutationError, Target, commit, state_before};
use crate::route::cost::{Cost, Tally, Uncostable};
use crate::route::obstacles::{Axis, Feature, Obstacles};
use crate::route::report::{Added, Crossing, Report, Status};
use crate::route::sheet::{Routed, SheetObjects};
use crate::route::terminal::{Heading, Terminal};
use crate::route::window::Window;
use crate::view::snapshot::{Snapshot, SnapshotError};

/// Where a drawn wire starts or finishes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum End {
    /// One pin of one placed symbol, such as `R11.2`.
    Pin(PinAddress),
    /// One port of one child sheet, by the name the port carries.
    Port(String),
    /// A point of the drawing, which fixes no direction.
    At(Point),
}

/// The wire a caller asked for: two ends, and the corners between them.
#[derive(Clone, Debug)]
pub struct Polyline {
    /// Where the wire starts.
    pub from: End,
    /// Where it finishes.
    pub to: End,
    /// The corners between the two ends, in the order the wire meets them.
    pub via: Vec<Point>,
}

/// What one drawn wire produced.
#[derive(Clone, Debug)]
pub struct DrawnWire {
    /// The result, in the shape the output contract prints.
    pub report: Report,
    /// What the write changed, and what kicli checked afterwards.
    pub mutation: Mutation,
}

/// Why a wire was not drawn.
///
/// Every variant refuses before anything is written, so a refusal leaves the
/// file exactly as it was.
#[derive(Debug, thiserror::Error)]
pub enum WireError {
    /// A vertex is not on the placement grid.
    #[error(
        "the vertex at ({at}) is off the grid. Connectable geometry sits on it, \
         and kicli will not move a corner you chose. Give a grid point."
    )]
    OffGrid {
        /// The vertex that misses the grid.
        at: Point,
    },

    /// A segment runs along neither axis.
    #[error("the segment from ({from}) to ({to}) is diagonal. A wire runs along one axis.")]
    Diagonal {
        /// Where the segment starts.
        from: Point,
        /// Where it ends.
        to: Point,
    },

    /// Two vertices in a row are the same point.
    #[error("the vertex at ({at}) is given twice in a row, so one segment draws nothing.")]
    Stationary {
        /// The repeated vertex.
        at: Point,
    },

    /// The request names fewer than two points.
    #[error("a wire needs two ends, and this request gives fewer.")]
    TooShort,

    /// The wire does not leave a terminal the way that terminal is left.
    #[error(
        "a wire leaves {terminal} through ({escape}). This one goes to ({at}) instead. \
         Put ({escape}) on the wire."
    )]
    Escape {
        /// The terminal, as a report names it.
        terminal: String,
        /// The point the wire must reach before it turns.
        escape: Point,
        /// The vertex the wire goes to instead.
        at: Point,
    },

    /// Something a wire may not pass through is in the way.
    #[error("{handle} blocks the wire at ({at}), on the segment from the vertex at ({from}).")]
    Blocked {
        /// What is in the way, as a report names it.
        handle: String,
        /// Where the wire would have had to pass.
        at: Point,
        /// The vertex the blocked segment starts at.
        from: Point,
    },

    /// The sheet holds no symbol of that name on this sheet path.
    #[error("this sheet has no symbol called {reference} on sheet path {path}.")]
    NoSuchSymbol {
        /// The reference designator the caller named.
        reference: String,
        /// The sheet path kicli looked on.
        path: String,
    },

    /// The symbol has no pin of that number.
    #[error("{pin} does not exist: the symbol has no pin of that number.")]
    NoSuchPin {
        /// The pin the caller named.
        pin: PinAddress,
    },

    /// The symbol's definition is not embedded, so its pins have no positions.
    #[error(
        "the definition of {reference} is not in this file, so kicli cannot place its pins. \
         The symbol was placed from {lib_id}."
    )]
    NoDefinition {
        /// The reference designator of the symbol.
        reference: String,
        /// The library identifier the symbol was placed from.
        lib_id: String,
    },

    /// No child sheet of this file carries a port of that name.
    #[error("this sheet has no port called {name}.")]
    NoSuchPort {
        /// The port name the caller gave.
        name: String,
    },

    /// The file to edit is not one of the project's.
    #[error("{path} is not one of the files of this project.")]
    UnknownFile {
        /// The file the caller asked for.
        path: String,
    },

    /// The file holds no outermost list, so nothing can be added to it.
    #[error("this file is empty, so kicli cannot add anything to it.")]
    Empty,

    /// Every derived identifier is already taken.
    #[error("this file leaves no identifier free for a new wire.")]
    NoIdentifier,

    /// A fragment kicli built did not parse.
    #[error(transparent)]
    Sexpr(#[from] SexprError),

    /// The edited file did not read back as a schematic.
    #[error(transparent)]
    Read(#[from] ReadError),

    /// The state to compare the change against could not be taken.
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),

    /// The change did not survive its own checks, or could not be written.
    #[error(transparent)]
    Mutation(#[from] MutationError),
}

impl WireError {
    /// The word the output contract writes for this refusal.
    ///
    /// A request whose shape is wrong is `invalid`: the caller asked for
    /// something no drawing can hold. A request whose shape is right and whose
    /// way is barred is `blocked`, and the message names what barred it.
    #[must_use]
    pub const fn status(&self) -> Status {
        match self {
            Self::Blocked { .. } => Status::Blocked,
            _ => Status::Invalid,
        }
    }
}

/// Draw a wire through the vertices a caller gives.
///
/// The vertex list is the first end, then `request.via`, then the second end.
/// Every check runs before anything is written, so a refusal leaves the file as
/// it was. A valid polyline writes one two-point `wire` record per segment.
///
/// `routing` supplies the window margin and the cost weights. The margin only
/// decides how far outside the wire kicli looks for obstacles, because the
/// caller has already chosen the path.
///
/// # Errors
///
/// Returns [`WireError`] when an end cannot be resolved, when a vertex is off
/// the grid, when a segment is diagonal or stands still, when the wire does not
/// leave an end the way that end is left, or when something blocks it. None of
/// those writes a byte.
pub fn draw(
    hierarchy: &mut Hierarchy,
    request: &Polyline,
    routing: &Routing,
    target: &Target<'_>,
    taken: &str,
) -> Result<DrawnWire, WireError> {
    let file = file_of(hierarchy, target.path)?;
    let from = terminal_of(&hierarchy.files[file], target.sheet_path, &request.from)?;
    let to = terminal_of(&hierarchy.files[file], target.sheet_path, &request.to)?;

    let mut vertices = Vec::with_capacity(request.via.len() + 2);
    vertices.push(from.at);
    vertices.extend(request.via.iter().copied());
    vertices.push(to.at);
    if vertices.len() < 2 {
        return Err(WireError::TooShort);
    }
    for at in &vertices {
        if !on_grid(*at, target.grid) {
            return Err(WireError::OffGrid { at: *at });
        }
    }
    escapes_are_honoured(&from, &to, &vertices, target.grid)?;

    let (tally, crossings) = {
        let owned = [from.name.clone(), to.name.clone()];
        let routed = Routed {
            wires: &[],
            terminals: &owned,
        };
        let objects = SheetObjects::read(&hierarchy.files[file], target.sheet_path, &routed);
        let (least, most) = bounds(&vertices);
        let window = Window::around(least, most, routing.margin, objects.page(), target.grid);
        let obstacles = Obstacles::build(window, &objects.geometry());
        let tally =
            Tally::of_path(&vertices, &obstacles).map_err(|why| refusal(&why, &vertices))?;
        (tally, crossings_on(&vertices, &obstacles))
    };

    // One seed per request over one design, so a command run twice writes one
    // file rather than two that differ only in their identifiers.
    let seed = format!(
        "{} wire {} to {}",
        target.path.display(),
        from.name,
        to.name
    );
    let (wires, mutation) =
        write_segments(&mut hierarchy.files[file], &vertices, &seed, target, taken)?;

    let mut report = Report::of(Status::Routed, &from.name, &to.name);
    report.path = vertices;
    report.tally = tally;
    report.cost = Cost::of(tally, routing);
    report.crossings = crossings;
    report.added = Added {
        wires,
        junctions: Vec::new(),
    };
    Ok(DrawnWire { report, mutation })
}

/// Write one record per segment, then check the file and write it.
fn write_segments(
    loaded: &mut LoadedFile,
    vertices: &[Point],
    seed: &str,
    target: &Target<'_>,
    taken: &str,
) -> Result<(Vec<Uuid>, Mutation), WireError> {
    let segments = vertices.len() - 1;
    let uuids: Vec<Uuid> = Identifiers::for_document(&loaded.doc, seed)
        .take(segments)
        .collect();
    if uuids.len() != segments {
        return Err(WireError::NoIdentifier);
    }

    let before: Snapshot = state_before(&loaded.doc, &loaded.schematic, target.sheet_path, taken)?;
    let root = loaded.doc.root().ok_or(WireError::Empty)?;
    // The records go in in the order the wire meets its corners, each after
    // the last, so the file reads the way the wire is drawn.
    let first = insertion_index(&loaded.doc, root);
    for (offset, (pair, uuid)) in vertices.windows(2).zip(&uuids).enumerate() {
        let fragment = loaded
            .doc
            .add_fragment(&segment_fragment(pair[0], pair[1], uuid))?;
        loaded.doc.insert_child(root, first + offset, fragment);
    }

    let mutation = commit(&loaded.doc, target, &before, taken)?;
    loaded.schematic = Schematic::read(&loaded.doc)?;
    Ok((uuids, mutation))
}

/// One two-point wire record, in the shape KiCad writes.
fn segment_fragment(from: Point, to: Point, uuid: &Uuid) -> String {
    format!(
        "(wire (pts (xy {} {}) (xy {} {})) (stroke (width 0) (type default)) (uuid {}))",
        from.x,
        from.y,
        to.x,
        to.y,
        quote(&uuid.0)
    )
}

/// The terminal one end of a request names.
fn terminal_of(loaded: &LoadedFile, sheet: &SheetPath, end: &End) -> Result<Terminal, WireError> {
    match end {
        End::At(at) => Ok(Terminal::of_point(*at, &at.to_string())),
        End::Port(name) => loaded
            .schematic
            .sheets()
            .flat_map(|child| &child.pins)
            .find(|port| port.name == *name)
            .map(Terminal::of_sheet_pin)
            .ok_or_else(|| WireError::NoSuchPort { name: name.clone() }),
        End::Pin(pin) => pin_terminal(loaded, sheet, pin),
    }
}

/// The terminal one pin of one placed symbol makes.
///
/// The unit is a property of the sheet path rather than of the cache beside the
/// `lib_id`. A sheet placed twice draws a different unit on each placement, and
/// resolving from the cache would answer for the other placement's pin.
fn pin_terminal(
    loaded: &LoadedFile,
    sheet: &SheetPath,
    pin: &PinAddress,
) -> Result<Terminal, WireError> {
    let symbol = loaded
        .schematic
        .symbols()
        .find(|symbol| symbol.reference_on(sheet) == Some(&pin.reference))
        .ok_or_else(|| WireError::NoSuchSymbol {
            reference: pin.reference.0.clone(),
            path: sheet.0.clone(),
        })?;
    let library = read_library(
        &loaded.doc,
        &loaded.schematic.library_symbols,
        loaded.schematic.version,
    );
    let definition = definition_of(&library, symbol).ok_or_else(|| WireError::NoDefinition {
        reference: pin.reference.0.clone(),
        lib_id: symbol.lib_id.0.clone(),
    })?;
    resolve_pins(&symbol.drawn_on(sheet), definition)
        .iter()
        .find(|resolved| resolved.number == pin.number)
        .map(|resolved| Terminal::of_pin(&pin.reference.0, resolved))
        .ok_or_else(|| WireError::NoSuchPin { pin: pin.clone() })
}

/// Does the wire leave each end the way that end is left?
///
/// A terminal that fixes no direction may be met from any side. A segment that
/// runs along neither axis names no heading, which is the diagonal rule's
/// business rather than this one's, so it is left to the walk: two refusals for
/// one fault would name the same vertex twice and answer neither question well.
fn escapes_are_honoured(
    from: &Terminal,
    to: &Terminal,
    vertices: &[Point],
    grid: Iu,
) -> Result<(), WireError> {
    let last = vertices.len() - 1;
    for (terminal, at, next) in [
        (from, vertices[0], vertices[1]),
        (to, vertices[last], vertices[last - 1]),
    ] {
        let Some(escape) = terminal.escape else {
            continue;
        };
        if Heading::between(at, next).is_some_and(|taken| taken != escape) {
            return Err(WireError::Escape {
                terminal: terminal.name.clone(),
                escape: terminal.escape_point(grid),
                at: next,
            });
        }
    }
    Ok(())
}

/// Every crossing of another net's wire, in the order the wire makes them.
///
/// The walk that costs the path counts crossings; this one names the wires. A
/// count with no names tells an agent that a route is dear and not what to
/// move, which is what the breakdown is for.
fn crossings_on(vertices: &[Point], obstacles: &Obstacles) -> Vec<Crossing> {
    let window = obstacles.window();
    let grid = window.grid();
    let mut found = Vec::new();
    for pair in vertices.windows(2) {
        let (from, to) = (pair[0], pair[1]);
        let Some(heading) = Heading::between(from, to) else {
            continue;
        };
        let span = (to.x.0 - from.x.0).abs() + (to.y.0 - from.y.0).abs();
        for step in 1..=(span / grid.0) {
            let at = heading.step(from, Iu(step * grid.0));
            let Some(cell) = window.cell(at) else {
                continue;
            };
            for feature in obstacles.features(cell) {
                if let Feature::ForeignWire { handle, axis } = feature {
                    if *axis != Axis::of(heading) {
                        found.push(Crossing {
                            at,
                            wire: handle.clone(),
                            net: None,
                        });
                    }
                }
            }
        }
    }
    found
}

/// Which refusal a walk that could not cost the path is.
fn refusal(why: &Uncostable, vertices: &[Point]) -> WireError {
    match why {
        Uncostable::TooShort(_) => WireError::TooShort,
        Uncostable::Stationary { at } => WireError::Stationary { at: *at },
        Uncostable::Diagonal { from, to } => WireError::Diagonal {
            from: *from,
            to: *to,
        },
        Uncostable::OffGrid { from, .. } => WireError::OffGrid { at: *from },
        Uncostable::Blocked { handle, at } => WireError::Blocked {
            handle: handle.clone(),
            at: *at,
            from: vertex_before(vertices, *at),
        },
    }
}

/// The vertex the segment holding a point starts at.
fn vertex_before(vertices: &[Point], at: Point) -> Point {
    vertices
        .windows(2)
        .find(|pair| on_segment(pair[0], pair[1], at))
        .map_or(at, |pair| pair[0])
}

/// The two corners of the box the vertices sit in.
fn bounds(vertices: &[Point]) -> (Point, Point) {
    let mut least = vertices[0];
    let mut most = vertices[0];
    for at in vertices {
        least = Point::new(least.x.0.min(at.x.0), least.y.0.min(at.y.0));
        most = Point::new(most.x.0.max(at.x.0), most.y.0.max(at.y.0));
    }
    (least, most)
}

/// Is a vertex on the placement grid?
fn on_grid(at: Point, grid: Iu) -> bool {
    grid.0 != 0 && at.x.0 % grid.0 == 0 && at.y.0 % grid.0 == 0
}

/// Which file of the hierarchy the target names.
fn file_of(hierarchy: &Hierarchy, path: &Path) -> Result<usize, WireError> {
    let wanted = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_owned());
    hierarchy
        .files
        .iter()
        .position(|file| {
            std::fs::canonicalize(&file.path).unwrap_or_else(|_| file.path.clone()) == wanted
        })
        .ok_or_else(|| WireError::UnknownFile {
            path: path.display().to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::{Point, Uuid, WireError, bounds, on_grid, segment_fragment, vertex_before};
    use crate::geometry::GRID;
    use crate::route::report::Status;
    use kicli_sexpr::Doc;

    #[test]
    fn a_segment_is_one_two_point_record() {
        let text = segment_fragment(
            Point::new(508_000, 889_000),
            Point::new(762_000, 889_000),
            &Uuid("abc".to_owned()),
        );
        assert_eq!(
            text,
            "(wire (pts (xy 50.8 88.9) (xy 76.2 88.9)) \
             (stroke (width 0) (type default)) (uuid \"abc\"))"
        );
        assert!(Doc::parse(&text).is_ok(), "the record parses: {text}");
    }

    #[test]
    fn a_refusal_carries_the_word_the_contract_prints() {
        // A request no drawing can hold is invalid; a way that is barred is
        // blocked. The command layer reads this to choose its exit code.
        let blocked = WireError::Blocked {
            handle: "U1".to_owned(),
            at: Point::new(0, 0),
            from: Point::new(0, 0),
        };
        assert_eq!(blocked.status(), Status::Blocked);
        for invalid in [
            WireError::OffGrid {
                at: Point::new(1, 1),
            },
            WireError::Diagonal {
                from: Point::new(0, 0),
                to: Point::new(1, 1),
            },
            WireError::TooShort,
        ] {
            assert_eq!(invalid.status(), Status::Invalid, "{invalid}");
        }
    }

    #[test]
    fn a_blocked_point_names_the_segment_it_is_on() {
        let path = [Point::new(0, 0), Point::new(0, 100), Point::new(100, 100)];
        assert_eq!(vertex_before(&path, Point::new(0, 50)), path[0]);
        assert_eq!(vertex_before(&path, Point::new(50, 100)), path[1]);
        // A point on no segment names itself rather than a vertex it is not on.
        let stray = Point::new(500, 500);
        assert_eq!(vertex_before(&path, stray), stray);
    }

    #[test]
    fn the_box_holds_every_vertex() {
        let path = [Point::new(30, 10), Point::new(30, -40), Point::new(-5, -40)];
        assert_eq!(bounds(&path), (Point::new(-5, -40), Point::new(30, 10)));
    }

    #[test]
    fn a_grid_point_is_a_whole_number_of_steps_on_both_axes() {
        assert!(on_grid(Point::new(508_000, 457_200), GRID));
        assert!(!on_grid(Point::new(508_000, 457_201), GRID));
        assert!(!on_grid(Point::new(508_001, 457_200), GRID));
        // A grid of nothing has no points, and answering yes would divide by it
        // one call later.
        assert!(!on_grid(Point::default(), crate::geometry::Iu(0)));
    }
}
