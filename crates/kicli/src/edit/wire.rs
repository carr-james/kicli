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
//!
//! **A delete removes what it was asked for and nothing else.** It does not
//! cascade into junctions. A junction the removal leaves sitting on two ends is
//! a legal drawing, and taking it away is a second decision that belongs to
//! whoever asks for it: a delete that tidies up after itself is a delete an
//! agent cannot predict. So [`delete`] **reports** every junction on the
//! segment it removed that is now left joining fewer than [`JOINING`] wire
//! ends, and the caller decides what to do about each.

use std::path::Path;

use kicli_sexpr::{NodeId, SexprError, quote};

use crate::connectivity::{Net, NetPin, Nets, extract};
use crate::edit::insert::{Identifiers, document_name, insertion_index};
use crate::edit::mark::{PinAddress, WireEnd, wire_ends_at};
use crate::geometry::{Iu, Point, on_segment, resolve_pins};
use crate::model::config::Routing;
use crate::model::hierarchy::{Hierarchy, LoadedFile};
use crate::model::items::{Line, LineKind, ReadError, Schematic, SheetPath, Uuid};
use crate::model::library::{definition_of, read_library};
use crate::model::mutate::{Mutation, MutationError, Target, commit, state_before};
use crate::route::cost::{Cost, Tally, Uncostable};
use crate::route::obstacles::{Axis, Feature, Obstacles};
use crate::route::propose::walked;
use crate::route::report::{Added, Adjusted, Crossing, Report, Status};
use crate::route::search::Search;
use crate::route::shapes::Shapes;
use crate::route::sheet::{Routed, SheetObjects};
use crate::route::terminal::{Heading, Terminal, has_room, settled};
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

/// What a connection is asked to reach.
///
/// One end and a whole net are different requests rather than two spellings of
/// one. An end names a point; a net names everything a route could join it at,
/// and choosing between those is the router's decision rather than the
/// caller's.
#[derive(Clone, Debug)]
pub enum Destination {
    /// One end: a pin, a port, or a point of the drawing.
    End(End),
    /// A whole net, by the name the drawing gives it or by the handle kicli
    /// gives one the drawing does not name.
    Net(String),
}

/// What one drawn wire produced.
#[derive(Clone, Debug)]
pub struct DrawnWire {
    /// The result, in the shape the output contract prints.
    pub report: Report,
    /// What the write changed, and what kicli checked afterwards.
    pub mutation: Mutation,
}

/// How many wire ends a junction joins before it is doing a junction's work.
///
/// Three. Two wire ends that meet are joined without a dot, so a junction on
/// two ends draws something the drawing already said; on fewer, it says nothing
/// at all. This measures the same quantity as [`crate::edit::mark`]'s refusal
/// boundary — how many wire ends meet at a point — and both boundaries call the
/// same implementation. They are two different thresholds on one measurement,
/// approached from opposite directions.
pub const JOINING: usize = 3;

/// How much of an identifier a caller must type to name a segment.
///
/// The same floor the command layer addresses every other object by. Eight
/// characters are what the views print.
const HANDLE: usize = 8;

/// A junction a delete left joining fewer than [`JOINING`] wire ends.
///
/// It is still in the file. This is the report that lets a caller decide.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stranded {
    /// The junction's identifier.
    pub junction: Uuid,
    /// Where it sits.
    pub at: Point,
    /// The wire ends it is left joining, which is fewer than [`JOINING`].
    pub ends: Vec<WireEnd>,
}

/// What one deleted segment produced.
#[derive(Clone, Debug)]
pub struct DeletedWire {
    /// The identifier of the record that was removed.
    pub removed: Uuid,
    /// One end of the segment that was removed.
    pub from: Point,
    /// The other end.
    pub to: Point,
    /// The junctions the removal left joining fewer than [`JOINING`] ends.
    ///
    /// Every one of them is still in the file: the command reports, and never
    /// cascades.
    pub stranded: Vec<Stranded>,
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

    /// The two ends of a connection are one point.
    #[error(
        "both ends of this connection are at ({at}), so there is nothing to join. \
         Name two different ends."
    )]
    SamePoint {
        /// The point both ends are at.
        at: Point,
    },

    /// No route joins the two ends.
    #[error(
        "no route joins {from} to {to}{}. \
         Move a symbol out of the way, or connect them with a pair of labels.",
        if .blocked_by.is_empty() {
            String::new()
        } else {
            format!(": {} is in the way", .blocked_by.join(", "))
        }
    )]
    NoRoute {
        /// The end the route was to leave.
        from: String,
        /// The end it was to reach.
        to: String,
        /// What stood in the way, named once each.
        blocked_by: Vec<String>,
    },

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

    /// No net of this project answers to the name given.
    #[error(
        "this project has no net called {name}. \
         Run sch view to list the nets, which names an unnamed one #n1, #n2 and so on."
    )]
    NoSuchNet {
        /// The name the caller gave.
        name: String,
    },

    /// More than one net of this project answers to the name given.
    #[error(
        "{name} names {} nets of this project: {}. \
         A local label names one net per sheet, so the same text can name several. \
         Name a pin of the net you mean with --to-pin, or a point on it with --to-at.",
        .candidates.len(),
        .candidates.join("; ")
    )]
    AmbiguousNet {
        /// The name the caller gave.
        name: String,
        /// Every net it named, each with the name KiCad gives it and its pins.
        candidates: Vec<String>,
    },

    /// The net is a net of this project, and nothing of it is on this sheet.
    #[error(
        "the net {name} is not drawn on this sheet, so a wire on this sheet cannot reach it. \
         Draw it on the sheet the net is on, or join the two with a pair of labels."
    )]
    NetOffSheet {
        /// The net the caller named.
        name: String,
    },

    /// The net is on this sheet and offers no point a route may end on.
    #[error(
        "the net {name} offers no point on this sheet that a wire may join. \
         Every point of it already carries as many wire ends as one point may."
    )]
    NetHasNoRoom {
        /// The net the caller named.
        name: String,
    },

    /// No child sheet of this file carries a port of that name.
    #[error("this sheet has no port called {name}.")]
    NoSuchPort {
        /// The port name the caller gave.
        name: String,
    },

    /// No wire of this sheet carries the identifier given.
    #[error(
        "this sheet has no wire called {identifier}. \
         Name at least {HANDLE} characters of an identifier. \
         Run sch view --uuids to list them."
    )]
    NoSuchWire {
        /// The identifier the caller gave.
        identifier: String,
    },

    /// More than one segment answers to the identifier given.
    #[error(
        "{identifier} names {} segments of this sheet: {}. Name more of the identifier.",
        .matched.len(),
        .matched.join(", ")
    )]
    AmbiguousWire {
        /// The identifier the caller gave.
        identifier: String,
        /// Every segment it named, by its whole identifier.
        matched: Vec<String>,
    },

    /// The identifier names a bundle rather than a wire.
    #[error(
        "{identifier} is a bus, not a wire. A bundle carries several nets, and \
         removing one is not this verb's decision."
    )]
    NotAWire {
        /// The identifier the caller gave.
        identifier: String,
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
            Self::Blocked { .. } | Self::NoRoute { .. } => Status::Blocked,
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
        document_name(target.path, target.project),
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

/// The connection a caller asked for: two ends, and no path between them.
///
/// The difference from [`Polyline`] is the whole of the difference between the
/// two verbs. A polyline says where the wire goes; a connection says only what
/// must end up joined, and the router chooses the path.
#[derive(Clone, Debug)]
pub struct Connection {
    /// Where the connection starts.
    pub from: End,
    /// What it must reach.
    pub to: Destination,
}

/// The route the router found, before anything is written.
///
/// Planning and writing are two calls because the decision between them is not
/// the router's: a route longer than `routing.label_threshold` is proposed as a
/// pair of labels, and the name that pair carries comes from connectivity and
/// from the pins, which is the command layer's own vocabulary. So the router
/// answers with what it found and the caller decides what to do about it.
#[derive(Clone, Debug)]
pub struct Plan {
    /// The end the route leaves, after any four-way adjustment.
    pub source: Terminal,
    /// The end it arrives at, after any four-way adjustment.
    pub target: Terminal,
    /// The terminals the router moved, empty when it moved neither.
    pub adjusted: Vec<Adjusted>,
    /// The route, when one was found.
    pub route: Option<Planned>,
    /// What stood in the way, when none was.
    pub blocked_by: Vec<String>,
    /// How many candidates were tried before this one was chosen.
    pub considered: u32,
}

/// One route the router would draw, and the junctions drawing it needs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Planned {
    /// The vertices, from the source terminal to the target terminal.
    pub path: Vec<Point>,
    /// What walking it meets.
    pub tally: Tally,
    /// What that costs, in parts.
    pub cost: Cost,
    /// The crossings, in the order the route makes them.
    pub crossings: Vec<Crossing>,
    /// Where the route needs a junction, in the order the route meets them.
    ///
    /// A junction goes where an existing wire of the same net has the route's
    /// own end in its **interior**, and nowhere else: a KiCad wire's connection
    /// points are its two ends and nothing between them, so a route that ends
    /// where an old one ends is already joined and a dot there says nothing.
    /// Empty is the common answer.
    pub junctions: Vec<Point>,
}

impl Planned {
    /// How long the route is, walked leg by leg.
    #[must_use]
    pub fn length(&self) -> Iu {
        walked(&self.path)
    }
}

impl Plan {
    /// The report of a route this plan would draw, before it is drawn.
    ///
    /// `added` is left empty: nothing has been written yet, and a report that
    /// claimed otherwise would be a second answer waiting to disagree with the
    /// file.
    #[must_use]
    pub fn report(&self, routing: &Routing) -> Report {
        let status = if self.route.is_some() {
            Status::Routed
        } else {
            Status::Blocked
        };
        let mut report = Report::of(status, &self.source.name, &self.target.name);
        report.adjusted.clone_from(&self.adjusted);
        report.alternatives_considered = self.considered;
        report.blocked_by.clone_from(&self.blocked_by);
        if let Some(route) = &self.route {
            report.path.clone_from(&route.path);
            report.tally = route.tally;
            report.cost = Cost::of(route.tally, routing);
            report.crossings.clone_from(&route.crossings);
        }
        report
    }
}

/// Route one connection, and answer with what would be drawn.
///
/// Nothing is written and nothing is decided about whether to write: this is
/// the router run over one sheet, in the composition
/// `research/wire-routing.md` §4 fixes. [`settled`] settles every terminal
/// against the drawing first, so everything after it is asked about the
/// terminals it answered with; the silhouettes are tried before the search,
/// because half of all real segments are drawn as an I or an L.
///
/// **A whole net is one request, not many.** A connection that names a net is
/// given every terminal of it at once: the silhouettes are tried against the
/// nearest first and the cheapest candidate wins, and the search — when no
/// silhouette fits — runs once over the whole set rather than once per
/// terminal. The route that comes back is the cheapest route to the net, and
/// the plan names the terminal it ends on.
///
/// **Whose a wire is, is answered here.** A wire that already passes through
/// either end of the route is the route's own: the route is joining it rather
/// than avoiding it, and a route refused by the wire already on its own pin
/// would be refused at the very point it was asked to leave. Every other wire
/// blocks along its own axis and is costed across it.
///
/// # Errors
///
/// Returns [`WireError`] when an end cannot be resolved, when an end is off the
/// placement grid, or when the two ends are one point. None of those writes a
/// byte, and neither does a route the router could not find: that comes back as
/// a [`Plan`] with no route and the list of what stood in the way.
pub fn plan(
    hierarchy: &Hierarchy,
    request: &Connection,
    routing: &Routing,
    target: &Target<'_>,
) -> Result<Plan, WireError> {
    let file = file_of(hierarchy, target.path)?;
    let loaded = &hierarchy.files[file];
    let asked_from = terminal_of(loaded, target.sheet_path, &request.from)?;
    if !asked_from.is_on_grid(target.grid) {
        return Err(WireError::OffGrid { at: asked_from.at });
    }
    let (source, moved) = settled(&asked_from, &loaded.schematic, target.grid);
    let mut adjusted: Vec<Adjusted> = moved.into_iter().collect();
    let aim = aim_of(hierarchy, loaded, request, target, &source, &mut adjusted)?;

    let objects = SheetObjects::read(
        loaded,
        target.sheet_path,
        &Routed {
            wires: &aim.own,
            terminals: &aim.named,
        },
    );
    let (least, most) = bounds(&aim.reach(&source));
    let window = Window::around(least, most, routing.margin, objects.page(), target.grid);
    let obstacles = Obstacles::build(window, &objects.geometry());

    let found = cheapest(&source, &aim.targets, &obstacles, routing);
    let route = found.walked.map(|(path, tally)| Planned {
        crossings: crossings_on(&path, &obstacles),
        junctions: junctions_of(&loaded.schematic, &aim.own, &path),
        cost: Cost::of(tally, routing),
        tally,
        path,
    });
    // A route that was found stood in nobody's way in the end: the handles
    // collected on the way to it named candidates that were dropped rather
    // than a refusal, and reporting them would tell an agent to move a symbol
    // that is not in the way of the wire it just got.
    let blocked_by = if route.is_some() {
        Vec::new()
    } else {
        found.blocked_by
    };
    Ok(Plan {
        source,
        target: found.target,
        adjusted,
        route,
        blocked_by,
        considered: found.considered,
    })
}

/// What one request aims at: one settled terminal, or a whole net's worth.
///
/// The `adjusted` list grows here rather than being returned, because a
/// terminal that moved is reported whichever kind of request moved it.
fn aim_of(
    hierarchy: &Hierarchy,
    loaded: &LoadedFile,
    request: &Connection,
    target: &Target<'_>,
    source: &Terminal,
    adjusted: &mut Vec<Adjusted>,
) -> Result<Aim, WireError> {
    match &request.to {
        Destination::End(end) => {
            let asked_to = terminal_of(loaded, target.sheet_path, end)?;
            if !asked_to.is_on_grid(target.grid) {
                return Err(WireError::OffGrid { at: asked_to.at });
            }
            if source.at == asked_to.at {
                return Err(WireError::SamePoint { at: source.at });
            }
            let (settled_to, moved) = settled(&asked_to, &loaded.schematic, target.grid);
            adjusted.extend(moved);
            Ok(Aim {
                own: wires_through(&loaded.schematic, &[source.at, settled_to.at]),
                named: vec![source.name.clone(), settled_to.name.clone()],
                targets: vec![settled_to],
            })
        }
        Destination::Net(name) => {
            let net = net_named(&extract(hierarchy), name)?.clone();
            aim_at_net(loaded, target, source, &net)
        }
    }
}

/// What a planned route is aimed at, and what it owns on the way.
///
/// One terminal and a whole net differ only in how many terminals are in the
/// list. Everything after this point asks the same questions of both, which is
/// what makes the net-addressed form one search rather than a composition of
/// several.
struct Aim {
    /// Every terminal the route may end on, cheapest-to-reach first.
    targets: Vec<Terminal>,
    /// The wires the route joins rather than avoids.
    own: Vec<Uuid>,
    /// The pins and ports the route may end on, as the obstacle map names them.
    named: Vec<String>,
}

impl Aim {
    /// The source and every target, which is the area the window must hold.
    fn reach(&self, source: &Terminal) -> Vec<Point> {
        let mut points = vec![source.at];
        points.extend(self.targets.iter().map(|terminal| terminal.at));
        points
    }
}

/// The net one name or handle addresses.
///
/// The rule is the one `net rename` already addresses a net by: the name the
/// drawing gives it, or the `#n` handle kicli gives one the drawing does not
/// name. A name that answers for more than one net is refused with the
/// candidates, because a local label names one net **per sheet** and the same
/// text on two sheets is two nets. Choosing one of them for the caller is
/// guessing at which net they meant to join.
fn net_named<'a>(nets: &'a Nets, name: &str) -> Result<&'a Net, WireError> {
    let matched: Vec<&Net> = nets.nets().iter().filter(|net| net.name == name).collect();
    match matched.as_slice() {
        [only] => Ok(only),
        [] => Err(WireError::NoSuchNet {
            name: name.to_owned(),
        }),
        many => Err(WireError::AmbiguousNet {
            name: name.to_owned(),
            candidates: many.iter().map(|net| candidate(net)).collect(),
        }),
    }
}

/// One candidate of an ambiguous name, as the refusal lists it.
///
/// KiCad's own name for the net and the pins on it. The name is what
/// distinguishes two nets that share a label text, because KiCad prefixes a
/// local name with the sheet it is on; the pins are what let a caller pick one
/// with `--to-pin`.
fn candidate(net: &Net) -> String {
    let pins: Vec<String> = net.pins.iter().map(NetPin::label).collect();
    format!("{} on pins {}", net.kicad_name, pins.join(", "))
}

/// The pins of one net that are drawn on one sheet, as terminals.
///
/// A pin the sheet does not draw, and one whose library definition the file
/// does not carry, contribute nothing: there is no point on the drawing for a
/// route to end at.
fn net_pins(loaded: &LoadedFile, sheet: &SheetPath, net: &Net) -> Vec<Terminal> {
    net.pins
        .iter()
        .filter(|pin| pin.sheet == *sheet)
        .filter_map(|pin| {
            crate::route::sheet::pin_terminal(loaded, sheet, &pin.reference.0, &pin.number)
                .map(|(terminal, _)| terminal)
        })
        .collect()
}

/// Every terminal of one net on one sheet, in the order the router tries them.
///
/// **The target set is every grid point of the net's wires, plus its pins.**
/// A route joining a net may end anywhere the net already is, so the search is
/// given all of it at once rather than one point somebody chose in advance.
///
/// The wires are found from the net's **pins**, which the extractor answered
/// for: a wire with an end at a pin of the net is on the net, a junction on a
/// point of the net joins every line through that point, and both rules carry
/// on from each wire they admit. The walk therefore only ever adds a wire that
/// is on the net — it is sound rather than complete, and a wire it does not
/// reach is one terminal the router is not offered rather than a wrong one it
/// is.
///
/// A point that already carries as many wire ends as one point may is left
/// out. The route's own end would be one too many, and there is another point
/// of the same net one grid step away.
fn aim_at_net(
    loaded: &LoadedFile,
    target: &Target<'_>,
    source: &Terminal,
    net: &Net,
) -> Result<Aim, WireError> {
    let schematic = &loaded.schematic;
    let pinned = net_pins(loaded, target.sheet_path, net);
    let anchors: Vec<Point> = pinned.iter().map(|terminal| terminal.at).collect();
    let mut named: Vec<String> = vec![source.name.clone()];
    named.extend(pinned.iter().map(|terminal| terminal.name.clone()));
    // The pin keeps its escape: a wire drawn to a pin must still leave it the
    // way a pin is left, whichever net asked for the wire.
    let mut terminals: Vec<Terminal> = pinned
        .into_iter()
        .map(|terminal| Terminal {
            name: joint(&net.name, &terminal.name),
            ..terminal
        })
        .collect();
    if anchors.is_empty() {
        return Err(WireError::NetOffSheet {
            name: net.name.clone(),
        });
    }

    let own = net_wires(schematic, &anchors);
    for at in grid_points(schematic, &own, target.grid) {
        if terminals.iter().any(|held| held.at == at) {
            continue;
        }
        terminals.push(Terminal::of_point(at, &joint(&net.name, &at.to_string())));
    }

    // A terminal with no room, and a terminal on the source's own point, are
    // both points no route may end on.
    terminals.retain(|terminal| terminal.at != source.at && has_room(terminal.at, schematic));
    if terminals.is_empty() {
        return Err(WireError::NetHasNoRoom {
            name: net.name.clone(),
        });
    }
    // Nearest first. No orthogonal route is shorter than the Manhattan
    // distance, so this is the order of a lower bound on what each terminal
    // costs to reach — which is what lets the search stop early.
    terminals.sort_by_key(|terminal| {
        (
            span(source.at, terminal.at),
            terminal.at.x.0,
            terminal.at.y.0,
        )
    });

    let mut own_and_ends = wires_through(schematic, &[source.at]);
    for uuid in own {
        if !own_and_ends.contains(&uuid) {
            own_and_ends.push(uuid);
        }
    }
    Ok(Aim {
        targets: terminals,
        own: own_and_ends,
        named,
    })
}

/// What to call a terminal that is one point of a net.
///
/// The net and the point, because both are the answer: a caller asked to join
/// a net and needs to read back **where** the route joined it.
fn joint(net: &str, at: &str) -> String {
    format!("{net}@{at}")
}

/// The wires of one net on one sheet, from the points its pins stand on.
///
/// Two of KiCad's merge rules, and no more. A wire whose **end** is a point of
/// the net is on the net, because a wire's connection points are its two ends
/// (`SCH_LINE::GetConnectionPoints`). A junction at a point of the net joins
/// every line through that point, interior included
/// (`CONNECTION_GRAPH::updateItemConnectivity`). Each wire admitted brings its
/// own two ends, and every junction on it, into the set of points — so the two
/// rules carry the net out along the drawing until nothing more joins.
///
/// A bus is never admitted: a bundle carries several nets and joining one to a
/// route is not this verb's decision.
fn net_wires(schematic: &Schematic, anchors: &[Point]) -> Vec<Uuid> {
    let mut points: Vec<Point> = anchors.to_vec();
    let mut found: Vec<Uuid> = Vec::new();
    let mut growing = true;
    while growing {
        growing = false;
        for line in schematic.lines().filter(|line| line.kind == LineKind::Wire) {
            if found.contains(&line.uuid) {
                continue;
            }
            let joined = points.contains(&line.from)
                || points.contains(&line.to)
                || schematic.junctions().any(|junction| {
                    points.contains(&junction.at) && on_segment(line.from, line.to, junction.at)
                });
            if !joined {
                continue;
            }
            found.push(line.uuid.clone());
            growing = true;
            for at in [line.from, line.to] {
                if !points.contains(&at) {
                    points.push(at);
                }
            }
            for junction in schematic.junctions() {
                if on_segment(line.from, line.to, junction.at) && !points.contains(&junction.at) {
                    points.push(junction.at);
                }
            }
        }
    }
    found
}

/// Every grid point on the given wires, in file order then along each wire.
///
/// A wire that runs along neither axis, or whose length is not a whole number
/// of grid steps, contributes only the points the lattice holds — which for a
/// diagonal is none. The router works on a lattice, so a point off it is a
/// point no route can end on.
fn grid_points(schematic: &Schematic, wires: &[Uuid], grid: Iu) -> Vec<Point> {
    let mut found: Vec<Point> = Vec::new();
    for line in schematic.lines().filter(|line| wires.contains(&line.uuid)) {
        let Some(heading) = Heading::between(line.from, line.to) else {
            continue;
        };
        let span = span(line.from, line.to);
        if grid.0 == 0 || span % i64::from(grid.0) != 0 {
            continue;
        }
        let steps = span / i64::from(grid.0);
        for step in 0..=steps {
            let Ok(distance) = i32::try_from(step * i64::from(grid.0)) else {
                continue;
            };
            let at = heading.step(line.from, Iu(distance));
            if !found.contains(&at) {
                found.push(at);
            }
        }
    }
    found
}

/// The Manhattan distance between two points, in internal units.
fn span(from: Point, to: Point) -> i64 {
    i64::from(to.x.0 - from.x.0).abs() + i64::from(to.y.0 - from.y.0).abs()
}

/// What the two searches between them found.
struct Found {
    /// The vertices and what walking them meets, when a route was found.
    walked: Option<(Vec<Point>, Tally)>,
    /// The terminal the route ends on, or the nearest one when none was found.
    target: Terminal,
    /// How many candidates were tried, feasible or not.
    considered: u32,
    /// What stood in the way, named once each, in the order first met.
    blocked_by: Vec<String>,
}

/// The cheapest route from one terminal to any one of many.
///
/// The silhouettes first and the search only when none of them fits, which is
/// the order `research/wire-routing.md` §4 fixes: A\* with a corner penalty
/// finds *a* minimal route and often not the one a person would draw, and half
/// of all real segments are drawn as an I or an L.
///
/// **The silhouettes are tried against the nearest terminal first, and the
/// cheapest candidate wins.** Nearest is by Manhattan distance, which prices a
/// lower bound on any route to that terminal: no orthogonal path is shorter,
/// and every step costs at least `w_len`. So once a candidate is held, the
/// first terminal whose lower bound reaches its cost ends the enumeration —
/// and every terminal after that one is further still. A tie is kept by the
/// nearer terminal, because the enumeration is in that order and only a
/// strictly cheaper candidate replaces the one held.
///
/// **The search, when it runs, runs once over every terminal at once.** One
/// search per terminal would expand the same states again for each, and would
/// answer with the cheapest of several separate answers rather than with the
/// cheapest route.
fn cheapest(
    source: &Terminal,
    targets: &[Terminal],
    obstacles: &Obstacles,
    routing: &Routing,
) -> Found {
    let nearest = targets.first().cloned().unwrap_or_else(|| source.clone());
    let Silhouettes {
        best,
        considered,
        mut blocked_by,
    } = silhouettes(source, targets, obstacles, routing);
    if let Some((path, tally, _, target)) = best {
        return Found {
            walked: Some((path, tally)),
            target,
            considered,
            blocked_by: Vec::new(),
        };
    }

    let search = Search::to_any(source, targets, obstacles, routing);
    let considered = considered.saturating_add(search.expanded());
    if let Some(route) = search.route() {
        // The terminal the route ends on is the one its last vertex stands on.
        // A route that ends where no terminal is would be a route to nowhere,
        // and the nearest terminal is then the honest thing to name.
        let reached = route
            .path
            .last()
            .and_then(|at| targets.iter().find(|terminal| terminal.at == *at))
            .cloned()
            .unwrap_or(nearest);
        return Found {
            walked: Some((route.path.clone(), route.tally)),
            target: reached,
            considered,
            blocked_by: Vec::new(),
        };
    }
    // Both refused, so both lists are worth having: a silhouette meets things
    // the search never steps on, and the search meets things no silhouette
    // reaches.
    for handle in search.blocked_by() {
        if !blocked_by.contains(handle) {
            blocked_by.push(handle.clone());
        }
    }
    Found {
        walked: None,
        target: nearest,
        considered,
        blocked_by,
    }
}

/// What the silhouettes offered over every terminal.
struct Silhouettes {
    /// The cheapest candidate, with its cost and the terminal it ends on.
    best: Option<(Vec<Point>, Tally, i64, Terminal)>,
    /// How many candidates were tried, feasible or not.
    considered: u32,
    /// What stood in the way, named once each, in the order first met.
    blocked_by: Vec<String>,
}

/// Try the silhouettes against each terminal, nearest first, and keep the
/// cheapest candidate any of them offered.
fn silhouettes(
    source: &Terminal,
    targets: &[Terminal],
    obstacles: &Obstacles,
    routing: &Routing,
) -> Silhouettes {
    let grid = obstacles.window().grid();
    let mut found = Silhouettes {
        best: None,
        considered: 0,
        blocked_by: Vec::new(),
    };
    for target in targets {
        // Nothing from here on can beat what is already held, and every
        // terminal after this one is further still.
        if let Some((.., held, _)) = &found.best {
            if floor(source, target, routing, grid) >= *held {
                break;
            }
        }
        let shapes = Shapes::of(source, target, obstacles, routing);
        found.considered = found.considered.saturating_add(shapes.considered());
        let Some(candidate) = shapes.best() else {
            for handle in shapes.blocked_by() {
                if !found.blocked_by.contains(handle) {
                    found.blocked_by.push(handle.clone());
                }
            }
            continue;
        };
        let total = candidate.cost.total();
        if found
            .best
            .as_ref()
            .is_none_or(|(.., held, _)| total < *held)
        {
            found.best = Some((
                candidate.path.clone(),
                candidate.tally,
                total,
                target.clone(),
            ));
        }
    }
    found
}

/// The least a route from one terminal to another can cost.
///
/// One step per grid step of the Manhattan distance, at the length weight. An
/// orthogonal route is never shorter than that, and every other weight is
/// non-negative — so a candidate already held that costs this much or less
/// cannot be beaten by anything this far away.
fn floor(source: &Terminal, target: &Terminal, routing: &Routing, grid: Iu) -> i64 {
    if grid.0 <= 0 {
        return 0;
    }
    routing.w_len * (span(source.at, target.at) / i64::from(grid.0))
}

/// Draw a route the router planned, with the junctions it needs.
///
/// One mutation over the whole route: the wire records and the junction
/// records go in together, the invariants run once over the result, and one
/// delta says what the command did. Two mutations would leave a window in
/// which the file holds a wire that joins nothing.
///
/// # Errors
///
/// Returns [`WireError`] when the plan found no route, when the file holds no
/// outermost list, when no identifier is free, or when the change does not
/// survive its own checks. Nothing is written unless every invariant holds.
pub fn draw_plan(
    hierarchy: &mut Hierarchy,
    plan: &Plan,
    routing: &Routing,
    target: &Target<'_>,
    taken: &str,
) -> Result<DrawnWire, WireError> {
    let file = file_of(hierarchy, target.path)?;
    let route = plan.route.as_ref().ok_or_else(|| WireError::NoRoute {
        from: plan.source.name.clone(),
        to: plan.target.name.clone(),
        blocked_by: plan.blocked_by.clone(),
    })?;

    // One seed per request over one design, so a command run twice writes one
    // file rather than two that differ only in their identifiers.
    let seed = format!(
        "{} connect {} to {}",
        document_name(target.path, target.project),
        plan.source.name,
        plan.target.name
    );
    let (wires, junctions, mutation) = write_route(
        &mut hierarchy.files[file],
        &route.path,
        &route.junctions,
        &seed,
        target,
        taken,
    )?;

    let mut report = plan.report(routing);
    report.added = Added { wires, junctions };
    Ok(DrawnWire { report, mutation })
}

/// Every wire that already passes through one of the route's own ends.
///
/// A bus is left out: a bundle carries several nets and joining one to a route
/// is not this verb's decision.
fn wires_through(schematic: &Schematic, points: &[Point]) -> Vec<Uuid> {
    schematic
        .lines()
        .filter(|line| line.kind == LineKind::Wire)
        .filter(|line| points.iter().any(|at| on_segment(line.from, line.to, *at)))
        .map(|line| line.uuid.clone())
        .collect()
}

/// Where a route needs a junction, in the order the route meets them.
///
/// Only the two ends are asked, because only an end of the route is a new wire
/// end: a route that merely crosses a wire is a crossing, and a crossing with
/// a dot on it is a connection nobody asked for.
fn junctions_of(schematic: &Schematic, own: &[Uuid], path: &[Point]) -> Vec<Point> {
    let mut found = Vec::new();
    for at in [path.first(), path.last()].into_iter().flatten() {
        if !found.contains(at) && junction_needed(schematic, own, *at) {
            found.push(*at);
        }
    }
    found
}

/// Does a route ending at this point need a junction there?
///
/// **It does when an existing wire of the same net has the point in its
/// interior, and does not when the wire ends there.** A KiCad wire's
/// connection points are its two ends and nothing between them
/// (`SCH_LINE::GetConnectionPoints`), so a new wire that ends where an old one
/// ends is joined to it with no dot at all — KiCad renders a corner — while one
/// that ends in the middle of an old one is joined to nothing until a junction
/// says so (`CONNECTION_GRAPH::updateItemConnectivity`).
///
/// **The wire has to be the route's own**, which is what
/// [`wires_touching`] decides. A junction joins *every* wire through its
/// position, so putting one on a foreign wire's interior would merge a net the
/// caller never named.
///
/// The ends at the point are counted by [`wire_ends_at`], which is the one
/// implementation of that question; a line through the point that is not among
/// them has the point in its interior.
fn junction_needed(schematic: &Schematic, own: &[Uuid], at: Point) -> bool {
    let ends = wire_ends_at(schematic, at);
    schematic
        .lines()
        .filter(|line| line.kind == LineKind::Wire)
        .filter(|line| own.contains(&line.uuid))
        .filter(|line| on_segment(line.from, line.to, at))
        .any(|line| !ends.iter().any(|end| end.handle == line.uuid.short()))
}

/// Write one record per segment and one per junction, then check and write.
fn write_route(
    loaded: &mut LoadedFile,
    vertices: &[Point],
    junctions: &[Point],
    seed: &str,
    target: &Target<'_>,
    taken: &str,
) -> Result<(Vec<Uuid>, Vec<Uuid>, Mutation), WireError> {
    let segments = vertices.len() - 1;
    let wanted = segments + junctions.len();
    let uuids: Vec<Uuid> = Identifiers::for_document(&loaded.doc, seed)
        .take(wanted)
        .collect();
    if uuids.len() != wanted {
        return Err(WireError::NoIdentifier);
    }
    let (wires, dots) = uuids.split_at(segments);

    let before: Snapshot = state_before(&loaded.doc, &loaded.schematic, target.sheet_path, taken)?;
    let root = loaded.doc.root().ok_or(WireError::Empty)?;
    let first = insertion_index(&loaded.doc, root);
    let mut offset = 0;
    for (pair, uuid) in vertices.windows(2).zip(wires) {
        let fragment = loaded
            .doc
            .add_fragment(&segment_fragment(pair[0], pair[1], uuid))?;
        loaded.doc.insert_child(root, first + offset, fragment);
        offset += 1;
    }
    for (at, uuid) in junctions.iter().zip(dots) {
        let fragment = loaded.doc.add_fragment(&junction_fragment(*at, uuid))?;
        loaded.doc.insert_child(root, first + offset, fragment);
        offset += 1;
    }

    let mutation = commit(&loaded.doc, target, &before, taken)?;
    loaded.schematic = Schematic::read(&loaded.doc)?;
    Ok((wires.to_vec(), dots.to_vec(), mutation))
}

/// One junction record, in the shape KiCad writes.
///
/// The `junction add` verb builds the same record for itself, and deliberately:
/// its builder is private to a verb that commits a mutation of its own, and a
/// route writes its wires and its dots in **one** mutation. The two are one
/// record shape and no rule, so the rule that would have to be shared is not
/// there to share.
fn junction_fragment(at: Point, uuid: &Uuid) -> String {
    format!(
        "(junction (at {} {}) (diameter 0) (color 0 0 0 0) (uuid {}))",
        at.x,
        at.y,
        quote(&uuid.0)
    )
}

/// Delete one wire segment, by the identifier a report writes.
///
/// `identifier` is either a segment's whole identifier or the eight-character
/// handle a view prints for it. A handle that names more than one segment is
/// refused with the list, because choosing one of them for the caller is
/// guessing at which wire they meant to lose.
///
/// The record named is removed and nothing else is. Every junction that sits
/// on the removed segment and is now left joining fewer than [`JOINING`] wire
/// ends comes back in [`DeletedWire::stranded`], **still in the file**.
///
/// # Errors
///
/// Returns [`WireError::NoSuchWire`] when no segment answers to the
/// identifier, [`WireError::AmbiguousWire`] when more than one does, and
/// [`WireError::NotAWire`] when the one that does is a bus. None of those
/// writes a byte, and neither does a failed invariant.
pub fn delete(
    hierarchy: &mut Hierarchy,
    identifier: &str,
    target: &Target<'_>,
    taken: &str,
) -> Result<DeletedWire, WireError> {
    let file = file_of(hierarchy, target.path)?;
    let named = segment_named(&hierarchy.files[file].schematic, identifier)?;

    let loaded = &mut hierarchy.files[file];
    let before: Snapshot = state_before(&loaded.doc, &loaded.schematic, target.sheet_path, taken)?;
    loaded.doc.remove(named.node);
    let mutation = commit(&loaded.doc, target, &before, taken)?;
    loaded.schematic = Schematic::read(&loaded.doc)?;

    // Read after the write, so what is reported is what the file now says
    // rather than what the arithmetic expected it to say.
    let stranded = stranded_by(&loaded.schematic, named.from, named.to);
    Ok(DeletedWire {
        removed: named.uuid,
        from: named.from,
        to: named.to,
        stranded,
        mutation,
    })
}

/// The one segment an identifier names.
struct Named {
    uuid: Uuid,
    node: NodeId,
    from: Point,
    to: Point,
}

/// Which segment an identifier names, or why none does.
///
/// The rule is the one the command layer already addresses every other object
/// by, in [`crate::cli::edit::address`]: the whole identifier, or a prefix of
/// at least [`HANDLE`] characters, which is what a view prints. It is stated
/// again here rather than borrowed because that module answers in the command
/// layer's own failure type, and nothing below the command layer may depend on
/// it. Two statements of one rule is one too many, and the seam belongs to
/// whoever wires the verb.
///
/// Ambiguity is judged over the segments alone, because they are the objects
/// this verb can act on: a handle shared with a symbol says nothing about
/// which wire is meant.
fn segment_named(schematic: &Schematic, identifier: &str) -> Result<Named, WireError> {
    let matched: Vec<&Line> = schematic
        .lines()
        .filter(|line| names(&line.uuid, identifier))
        .collect();
    let [line] = matched[..] else {
        if matched.is_empty() {
            return Err(WireError::NoSuchWire {
                identifier: identifier.to_owned(),
            });
        }
        return Err(WireError::AmbiguousWire {
            identifier: identifier.to_owned(),
            matched: matched.iter().map(|line| line.uuid.0.clone()).collect(),
        });
    };
    if line.kind == LineKind::Bus {
        return Err(WireError::NotAWire {
            identifier: identifier.to_owned(),
        });
    }
    Ok(Named {
        uuid: line.uuid.clone(),
        node: line.node,
        from: line.from,
        to: line.to,
    })
}

/// Does this identifier name that object, whole or by a handle?
///
/// Eight characters are what a view prints, so eight is the floor. Fewer would
/// let a typo address a segment the caller never saw named.
fn names(uuid: &Uuid, identifier: &str) -> bool {
    uuid.0 == identifier || (identifier.len() >= HANDLE && uuid.0.starts_with(identifier))
}

/// The junctions on a removed segment that are left joining too few ends.
///
/// Only junctions the segment touched can have changed, and a junction on the
/// segment's interior loses as much as one on its end. Everything else on the
/// sheet joins exactly what it joined before, so naming it would be noise the
/// caller has to read past.
fn stranded_by(schematic: &Schematic, from: Point, to: Point) -> Vec<Stranded> {
    schematic
        .junctions()
        .filter(|junction| on_segment(from, to, junction.at))
        .filter_map(|junction| {
            let ends = wire_ends_at(schematic, junction.at);
            (ends.len() < JOINING).then(|| Stranded {
                junction: junction.uuid.clone(),
                at: junction.at,
                ends,
            })
        })
        .collect()
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
