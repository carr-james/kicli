//! The re-route calibration gate: what the router draws costs what a person drew.
//!
//! `spec/SPEC.md` §9 Q3 rules the objective check on the weights — re-route a
//! known-good sheet from scratch and assert the total cost is within **15 %** of
//! the cost of the wires a human actually drew. This file is that gate, over a
//! purpose-built in-repo fixture and, under `--features corpus`, over the tidy
//! hand-drawn demo sheet `ampli_ht` that `research/wire-routing.md` §2 measured.
//!
//! **It is a measurement, not a tuning exercise.** M4's rules say the weights
//! are not retuned in this milestone: they are shared with M5's linter. A sheet
//! outside 15 % is reported and parked, never fixed by moving a weight.
//!
//! **Cheap is an anomaly, never a win.** The tolerance is two-sided for exactly
//! that reason: a total far under the human's is evidence that the cost
//! function fails to price something the person was avoiding.
//!
//! **The gate must not be able to pass by skipping the hard nets.** Every
//! strand it declines is named with its reason, and the pins it thereby leaves
//! out are asserted to be under a quarter of the sheet's pins.
//!
//! # What is compared, and why it is one function
//!
//! A person's wiring is a **tree of segments**, not a path, so it cannot be fed
//! to `Tally::of_path` as it stands. Both sides are therefore reduced to the
//! same thing — the set of unit grid edges a drawing covers — and costed by one
//! function, [`cost_of_drawing`]. The original strand's segments go in; the
//! re-routed strand's polylines go in; neither side gets its own arithmetic.
//! That is what makes the comparison a comparison rather than two measurements.
//!
//! Constitution §4: no floating point anywhere below this line. Every cost is
//! `i64`, every coordinate an `Iu`, and the deviation is compared as
//! `|reroute − original| · 100 ≤ 15 · original`, in integers.

#![allow(clippy::too_many_lines, reason = "a measurement reads as a procedure")]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use kicli::connectivity::extract;
use kicli::geometry::{GRID, Point, Rect, on_segment, resolve_pins};
use kicli::model::config::Routing;
use kicli::model::{
    Config, Hierarchy, Item, LineKind, LoadedFile, Schematic, SheetPath, Uuid, definition_of,
    read_library,
};
use kicli::route::propose::walked;
use kicli::route::{
    Heading, Obstacles, Routed, Search, Segment, Shapes, SheetGeometry, SheetObjects, Tally,
    Terminal, Trigger, Window,
};
use kicli_probe::drawing::LabelKind;
use kicli_probe::oracle::{Kicad, Netlist, differences, kicli_partition};
use kicli_probe::scratch::Fixtures;
use kicli_probe::{Placed, Probe, millimetres, pin, rectangle, symbol};

/// The tolerance `spec/SPEC.md` §9 Q3 fixes, in per cent, in either direction.
pub const TOLERANCE_PERCENT: i64 = 15;

/// The share of a sheet's pins the gate may leave unmeasured, as a fraction.
///
/// A quarter, from the task entry: "assert the skipped pins are under a quarter
/// of the sheet's pins, so the gate cannot pass by skipping the hard nets."
const SKIP_BUDGET: (usize, usize) = (1, 4);

/// The margin KiCad's default drawing sheet leaves on every side, in internal
/// units.
///
/// 10 mm, from `common/drawing_sheet/ds_data_model.cpp:69` and the
/// `(left_margin 10)(right_margin 10)(top_margin 10)(bottom_margin 10)` of
/// `defaultDrawingSheet` (`drawing_sheet_default_description.cpp:125`), at tag
/// 10.0.5. One millimetre is 10 000 internal units.
const SHEET_MARGIN: i32 = 100_000;

/// The title block, as offsets from the inner bottom-right corner, in internal
/// units: 110 mm to 2 mm left of it, 34 mm to 2 mm above it.
///
/// `(rect (name "") (start 110 34) (end 2 2))` of `defaultDrawingSheet`, whose
/// unnamed corner is `rbcorner` — the bottom-right of the margin rectangle.
const TITLE_BLOCK: (i32, i32, i32, i32) = (1_100_000, 340_000, 20_000, 20_000);

// ---------------------------------------------------------------------------
// One sheet, chosen out of a hierarchy
// ---------------------------------------------------------------------------

/// One placement of one file, which is what a calibration run measures.
pub struct Sheet {
    hierarchy: Hierarchy,
    file: usize,
    path: SheetPath,
    label: String,
}

impl Sheet {
    /// The root sheet of a project.
    pub fn root_of(root: &Path) -> Self {
        let hierarchy = Hierarchy::load(root).expect("the hierarchy loads");
        let path = hierarchy.placements[0].path.clone();
        Self {
            hierarchy,
            file: 0,
            path,
            label: file_label(root),
        }
    }

    /// A named child file of a project, at its first placement.
    ///
    /// A sheet placed twice draws the same geometry twice under two sets of
    /// reference designators, so one placement is the whole drawing and the
    /// second would measure it again under other names. The placement chosen is
    /// the one with the smallest sheet path, which is a property of the file
    /// rather than of the order the walk happened to reach it.
    pub fn child_of(root: &Path, file_name: &str) -> Option<Self> {
        let hierarchy = Hierarchy::load(root).ok()?;
        let mut found: Option<(usize, SheetPath)> = None;
        for placement in &hierarchy.placements {
            let path = &hierarchy.files[placement.file].path;
            if !path.file_name().is_some_and(|name| name == file_name) {
                continue;
            }
            if found
                .as_ref()
                .is_none_or(|(_, held)| placement.path.0 < held.0)
            {
                found = Some((placement.file, placement.path.clone()));
            }
        }
        let (file, path) = found?;
        Some(Self {
            hierarchy,
            file,
            path,
            label: file_name.to_owned(),
        })
    }

    /// The same sheet, with its objects listed in the opposite order.
    ///
    /// **The environment variation this gate is exposed to.** KiCad reorders a
    /// file's objects when it saves, so the order a sheet lists its wires,
    /// junctions and symbols in is not a property of the drawing. Every number
    /// this gate reports, and every row of its table, must be the same either
    /// way — and one of them was not: the strand sort's key left ties to file
    /// order until the run-order break of 2026-08-21 found it.
    #[must_use]
    pub fn with_objects_reversed(mut self) -> Self {
        self.hierarchy.files[self.file].schematic.items.reverse();
        self
    }

    /// The loaded file this placement draws.
    fn file(&self) -> &LoadedFile {
        &self.hierarchy.files[self.file]
    }
}

/// The name a sheet is reported under.
fn file_label(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}

// ---------------------------------------------------------------------------
// The lattice a drawing covers
// ---------------------------------------------------------------------------

/// The unit grid edges one drawing covers.
///
/// Both the wires a person drew and the polylines the router produced are
/// reduced to this, so that one function costs them. An edge is held with its
/// smaller end first, so a segment drawn either way is the same edge and two
/// wires laid over each other are one.
#[derive(Clone, Debug, Default)]
struct Lattice {
    edges: BTreeSet<(Point, Point)>,
}

impl Lattice {
    /// Add one polyline, expanded into unit grid steps.
    ///
    /// Returns false for a polyline that is diagonal or off the lattice, which
    /// is a drawing this gate cannot describe rather than a drawing it prices
    /// wrongly.
    fn add(&mut self, points: &[Point]) -> bool {
        for pair in points.windows(2) {
            let (from, to) = (pair[0], pair[1]);
            let Some(heading) = Heading::between(from, to) else {
                return false;
            };
            let span = (to.x.0 - from.x.0).abs() + (to.y.0 - from.y.0).abs();
            if span % GRID.0 != 0 {
                return false;
            }
            let mut at = from;
            for _ in 0..span / GRID.0 {
                let next = heading.step(at, GRID);
                self.edges.insert(ordered(at, next));
                at = next;
            }
        }
        true
    }

    /// How many unit grid steps of wire the drawing holds.
    fn steps(&self) -> usize {
        self.edges.len()
    }

    /// Every grid point the drawing touches, and its neighbours.
    fn adjacency(&self) -> BTreeMap<Point, Vec<Point>> {
        let mut map: BTreeMap<Point, Vec<Point>> = BTreeMap::new();
        for &(one, other) in &self.edges {
            map.entry(one).or_default().push(other);
            map.entry(other).or_default().push(one);
        }
        map
    }
}

/// The two ends of a unit edge, smaller first.
fn ordered(one: Point, other: Point) -> (Point, Point) {
    if one <= other {
        (one, other)
    } else {
        (other, one)
    }
}

// ---------------------------------------------------------------------------
// Costing one drawing
// ---------------------------------------------------------------------------

/// Why a drawing could not be costed on this map.
#[derive(Clone, Debug)]
struct Refused {
    handle: String,
    at: Point,
}

/// What one drawing costs on one obstacle map.
///
/// **The one place a cost is computed**, for the human's wires and for the
/// router's alike. Length is the number of unit grid edges, so two wires laid
/// over each other count once. A **corner** is a grid point where exactly two
/// edges meet and they are perpendicular — which is what KiCad draws a bend at;
/// a point where three or more meet is a branch and carries a junction dot, and
/// a point where two collinear edges meet is one straight run KiCad happened to
/// split into two records. Crossings, text steps and crowded steps are counted
/// by entering each point once, along the edge that reached it, walking out
/// from the drawing's smallest point — which is `Tally::of_path`'s rule that
/// the point a walk starts from is not entered, generalised from a path to a
/// tree.
///
/// `terminals` are the drawing's own ends. A block there is the wire arriving
/// rather than the wire colliding, which is `research/wire-routing.md` §3.2's
/// "except the grid point at a target pin".
fn cost_of_drawing(
    lattice: &Lattice,
    obstacles: &Obstacles,
    terminals: &BTreeSet<Point>,
) -> Result<Tally, Refused> {
    let adjacency = lattice.adjacency();
    let mut tally = Tally {
        steps: u32::try_from(lattice.steps()).expect("a sheet holds fewer than 4e9 grid steps"),
        ..Tally::default()
    };

    for (&at, neighbours) in &adjacency {
        if neighbours.len() == 2 {
            let (one, other) = (neighbours[0], neighbours[1]);
            // Perpendicular when the two neighbours are not opposite each
            // other, which on a lattice means their midpoint is not the vertex.
            if one.x.0 + other.x.0 != 2 * at.x.0 || one.y.0 + other.y.0 != 2 * at.y.0 {
                tally.corners += 1;
            }
        }
    }

    let mut seen: BTreeSet<Point> = BTreeSet::new();
    let mut entered = 0_u32;
    for &root in adjacency.keys() {
        if seen.contains(&root) {
            continue;
        }
        seen.insert(root);
        let mut queue = std::collections::VecDeque::from([root]);
        while let Some(here) = queue.pop_front() {
            for &next in &adjacency[&here] {
                if !seen.insert(next) {
                    continue;
                }
                let heading = Heading::between(here, next).expect("a unit edge has a heading");
                let verdict = obstacles.entering(next, heading);
                if let Some(handle) = verdict.blocked_by {
                    if !terminals.contains(&next) {
                        return Err(Refused { handle, at: next });
                    }
                }
                entered += 1;
                tally.crossings += verdict.crossings;
                tally.text_steps += verdict.text_steps;
                tally.near_steps += verdict.near_steps;
                queue.push_back(next);
            }
        }
    }
    // A tree of E edges over V points has V − 1 points to enter, one per edge.
    // A drawing that closes a loop enters fewer, and the length term would then
    // price steps no verdict was taken on. Reporting it beats quietly costing
    // it either way.
    if entered != tally.steps {
        return Err(Refused {
            handle: format!(
                "the drawing is not a tree: {entered} points entered over {} steps",
                tally.steps
            ),
            at: *adjacency.keys().next().unwrap_or(&Point::default()),
        });
    }
    Ok(tally)
}

// ---------------------------------------------------------------------------
// The sheet's pins
// ---------------------------------------------------------------------------

/// One pin of one placed symbol on the sheet.
#[derive(Clone, Debug)]
struct PinSite {
    /// What a report calls it: `R12.1`.
    handle: String,
    /// Where it connects.
    at: Point,
    /// The terminal a route to it starts or ends on.
    terminal: Terminal,
}

/// Every pin every placed symbol draws on this sheet.
///
/// A hidden pin is included: it is drawn by nothing and still connects, which
/// is the rule `route::sheet` keeps for the obstacle map.
fn pins_of(sheet: &Sheet) -> Vec<PinSite> {
    let file = sheet.file();
    let library = read_library(
        &file.doc,
        &file.schematic.library_symbols,
        file.schematic.version,
    );
    let mut sites = Vec::new();
    for symbol in file.schematic.symbols() {
        let name = symbol
            .reference_on(&sheet.path)
            .map_or_else(|| symbol.uuid.short().to_owned(), |refdes| refdes.0.clone());
        let Some(definition) = definition_of(&library, symbol) else {
            continue;
        };
        for pin in resolve_pins(&symbol.drawn_on(&sheet.path), definition) {
            sites.push(PinSite {
                handle: format!("{name}.{}", pin.number),
                at: pin.position,
                terminal: Terminal::of_pin(&name, &pin),
            });
        }
    }
    sites.sort_by_key(|site| (site.at, site.handle.clone()));
    sites
}

// ---------------------------------------------------------------------------
// Strands: what a person actually drew, one connected piece at a time
// ---------------------------------------------------------------------------

/// One connected piece of drawn wire, with the pins it joins.
///
/// **A strand rather than a net**, because a net is not always a drawing. A
/// ground net is one net and a dozen separate two-pin strands, joined by the
/// value of a power symbol rather than by any wire; there is nothing to
/// re-route between two strands that were never drawn as one. So the unit of
/// this gate is the connected piece, which is exactly the thing a person sat
/// down and drew.
#[derive(Clone, Debug)]
struct Strand {
    /// The net the extractor puts these pins on, for the report.
    net: String,
    /// The placement the strand is drawn on, which resolves its pins.
    sheet_path: SheetPath,
    /// The wires, as indices into the sheet's wire list.
    wires: Vec<usize>,
    /// The pins the strand reaches, in `(x, y)` order.
    pins: Vec<PinSite>,
}

/// One wire of the sheet, as this gate needs it.
#[derive(Clone, Debug)]
struct Drawn {
    uuid: Uuid,
    from: Point,
    to: Point,
}

/// The disjoint-set forest the strands are found with.
struct Sets(Vec<usize>);

impl Sets {
    fn new(size: usize) -> Self {
        Self((0..size).collect())
    }
    fn root(&mut self, item: usize) -> usize {
        let mut here = item;
        while self.0[here] != here {
            self.0[here] = self.0[self.0[here]];
            here = self.0[here];
        }
        here
    }
    fn join(&mut self, one: usize, other: usize) {
        let (one, other) = (self.root(one), self.root(other));
        if one != other {
            self.0[one.max(other)] = one.min(other);
        }
    }
}

/// Everything the gate reads off one sheet before it measures anything.
struct Drawing {
    wires: Vec<Drawn>,
    buses: Vec<Drawn>,
    /// A bus entry's two ends, which a wire meets to join a bundle.
    bus_entries: Vec<(Point, Point)>,
    junctions: Vec<Point>,
    pins: Vec<PinSite>,
}

/// Read the sheet into the lists the strand walk needs.
fn read_drawing(sheet: &Sheet) -> Drawing {
    let file = sheet.file();
    let mut drawing = Drawing {
        wires: Vec::new(),
        buses: Vec::new(),
        bus_entries: Vec::new(),
        junctions: Vec::new(),
        pins: pins_of(sheet),
    };
    for line in file.schematic.lines() {
        let drawn = Drawn {
            uuid: line.uuid.clone(),
            from: line.from,
            to: line.to,
        };
        match line.kind {
            LineKind::Wire => drawing.wires.push(drawn),
            LineKind::Bus => drawing.buses.push(drawn),
        }
    }
    for item in &file.schematic.items {
        match item {
            Item::Junction(mark) => drawing.junctions.push(mark.at),
            Item::BusEntry(entry) => {
                drawing
                    .bus_entries
                    .push((entry.at, bus_entry_far(file, entry.node, entry.at)));
            }
            _ => {}
        }
    }
    drawing
}

/// Where a bus entry's far end is.
///
/// The record carries `(at x y)` and `(size dx dy)`; the stub runs from the one
/// to the sum of the two. A record with no size answers with its own anchor,
/// which is the conservative reading for a proximity report.
fn bus_entry_far(file: &LoadedFile, node: kicli_sexpr::NodeId, at: Point) -> Point {
    for &child in file.doc.children(node) {
        if !file.doc.head_is(child, "size") {
            continue;
        }
        let values = file.doc.children(child);
        let read = |index: usize| values.get(index).and_then(|&id| file.doc.atom_as_iu(id));
        if let (Some(dx), Some(dy)) = (read(1), read(2)) {
            return Point::new(at.x.0 + dx, at.y.0 + dy);
        }
    }
    at
}

/// The strands of one sheet, in a fixed order.
///
/// The merge rules are the geometric ones KiCad's netlister uses, and only
/// those: items that share a connection point are one conductor, and a junction
/// joins every wire that passes through its position, interior included
/// (`connectivity/graph.rs`, rules 1 and 2). The name-based rules are
/// deliberately left out — a strand is a drawing, and a name is not one.
fn strands_of(sheet: &Sheet, drawing: &Drawing) -> Vec<Strand> {
    let wires = &drawing.wires;
    let pins = &drawing.pins;
    let mut sets = Sets::new(wires.len() + pins.len());

    for (index, wire) in wires.iter().enumerate() {
        for (other_index, other) in wires.iter().enumerate().skip(index + 1) {
            if shares_an_end(wire, other) {
                sets.join(index, other_index);
            }
        }
        for (pin_index, pin) in pins.iter().enumerate() {
            if pin.at == wire.from || pin.at == wire.to {
                sets.join(index, wires.len() + pin_index);
            }
        }
    }
    for &mark in &drawing.junctions {
        let mut members: Vec<usize> = Vec::new();
        for (index, wire) in wires.iter().enumerate() {
            if on_segment(wire.from, wire.to, mark) {
                members.push(index);
            }
        }
        for (pin_index, pin) in pins.iter().enumerate() {
            if pin.at == mark {
                members.push(wires.len() + pin_index);
            }
        }
        for pair in members.windows(2) {
            sets.join(pair[0], pair[1]);
        }
    }

    let nets = extract(&sheet.hierarchy);
    let mut grouped: BTreeMap<usize, (Vec<usize>, Vec<usize>)> = BTreeMap::new();
    for index in 0..wires.len() {
        let root = sets.root(index);
        grouped.entry(root).or_default().0.push(index);
    }
    for pin_index in 0..pins.len() {
        let root = sets.root(wires.len() + pin_index);
        grouped.entry(root).or_default().1.push(pin_index);
    }

    let mut strands: Vec<Strand> = grouped
        .into_values()
        .map(|(wire_indices, pin_indices)| {
            let mut members: Vec<PinSite> = pin_indices.iter().map(|&i| pins[i].clone()).collect();
            // Sorted here rather than inherited from the sheet's own pin list,
            // because `spanning_tree` breaks ties on these indices and the
            // sheet's list is whatever order KiCad last saved its symbols in.
            // This is the sort the order check actually rests on: with it gone
            // and `pins_of`'s gone too, reversing the file's objects changes
            // the report.
            members.sort_by(|one, other| (one.at, &one.handle).cmp(&(other.at, &other.handle)));
            let mut names: BTreeSet<String> = BTreeSet::new();
            for pin in &members {
                let (reference, number) = pin.handle.rsplit_once('.').unwrap_or((&pin.handle, ""));
                if let Some(net) = nets.net_of(reference, number) {
                    names.insert(net.name.clone());
                }
            }
            let net = if names.is_empty() {
                "(unnamed)".to_owned()
            } else {
                names.into_iter().collect::<Vec<_>>().join("+")
            };
            Strand {
                net,
                sheet_path: sheet.path.clone(),
                wires: wire_indices,
                pins: members,
            }
        })
        .collect();
    // **A total order, and it has to be.** Two strands share no pin and no wire,
    // so the whole set of pin positions followed by the whole set of wire
    // endpoints separates any two of them; the net name is only there to make
    // the key readable. The earlier key was "first pin, net, pin count", which
    // is also total on these sheets — no two strands share their smallest pin —
    // but it is total by luck rather than by construction, and a report whose
    // row order can be settled by file order is a report no reader can diff.
    // `the_report_does_not_depend_on_the_order_the_file_lists_its_objects` is
    // the check; removing this sort and the one below makes it fail, measured
    // 2026-08-21.
    strands.sort_by_cached_key(|strand| {
        (
            strand.pins.iter().map(|pin| pin.at).collect::<Vec<Point>>(),
            strand.net.clone(),
            strand
                .wires
                .iter()
                .map(|&index| ordered(drawing.wires[index].from, drawing.wires[index].to))
                .collect::<BTreeSet<(Point, Point)>>(),
        )
    });
    strands
}

/// Do two wires meet at an end?
fn shares_an_end(one: &Drawn, other: &Drawn) -> bool {
    [one.from, one.to]
        .iter()
        .any(|end| *end == other.from || *end == other.to)
}

// ---------------------------------------------------------------------------
// One strand, measured
// ---------------------------------------------------------------------------

/// What one strand cost the person, and what it costs the router.
struct Measured {
    net: String,
    pins: usize,
    segments: usize,
    original: Tally,
    original_cost: i64,
    rerouted: Tally,
    rerouted_cost: i64,
    /// The grid points of each drawn wire record, for the border control.
    original_wires: Vec<Vec<Point>>,
    /// The polylines the router produced, for the two ruling reports.
    paths: Vec<Vec<Point>>,
}

/// A strand the gate declined, and why.
struct Skipped {
    net: String,
    pins: usize,
    why: String,
}

/// The sheet with one strand's own drawing taken out of the typed items.
///
/// The strand's **wires** stay and are named as the route's own through
/// [`Routed`], which the obstacle map treats as free to enter — the same
/// verdict as removing them, and it is the seam the production adapter already
/// has. Its **junctions, no-connects and labels** have no such seam and are
/// removed here, because a junction on the strand's own wire is a hard block
/// standing exactly where the person's wire runs, and the label on that wire is
/// the strand's own name. Leaving either in would charge the person for their
/// own drawing and refuse the router the same ground.
fn without_the_strands_own_marks(
    file: &LoadedFile,
    strand: &Strand,
    drawing: &Drawing,
) -> LoadedFile {
    let touches = |at: Point| {
        strand.pins.iter().any(|pin| pin.at == at)
            || strand
                .wires
                .iter()
                .any(|&index| on_segment(drawing.wires[index].from, drawing.wires[index].to, at))
    };
    let mut schematic: Schematic = file.schematic.clone();
    schematic.items.retain(|item| match item {
        Item::Junction(mark) | Item::NoConnect(mark) => !touches(mark.at),
        Item::Label(label) => !touches(label.at),
        _ => true,
    });
    LoadedFile {
        path: file.path.clone(),
        doc: file.doc.clone(),
        schematic,
    }
}

/// The spanning tree the entry fixes: over the strand's pins, by Manhattan
/// distance, ties broken by `(x, y)`.
///
/// Kruskal over every pair, the pairs sorted by `(distance, first, second)`
/// where the two indices are into the pins **already sorted by `(x, y)`**. So
/// the tree is a function of the pin positions alone: no iteration order, no
/// hash, no file order reaches the answer.
fn spanning_tree(pins: &[PinSite]) -> Vec<(usize, usize)> {
    let mut pairs: Vec<(i64, usize, usize)> = Vec::new();
    for one in 0..pins.len() {
        for other in one + 1..pins.len() {
            let distance = i64::from((pins[other].at.x.0 - pins[one].at.x.0).abs())
                + i64::from((pins[other].at.y.0 - pins[one].at.y.0).abs());
            pairs.push((distance, one, other));
        }
    }
    pairs.sort_unstable();
    let mut sets = Sets::new(pins.len());
    let mut tree = Vec::new();
    for (_, one, other) in pairs {
        if sets.root(one) != sets.root(other) {
            sets.join(one, other);
            tree.push((one, other));
        }
    }
    tree
}

/// Route one tree edge the way the router would: shapes first, A\* second.
///
/// `research/wire-routing.md` §4 fixes that order, and this gate must ask the
/// router the question the verb asks it, not an easier one. The window is the
/// one a route request builds — the bounding box of the two terminals, inflated
/// by `routing.margin`, clipped to the page — rather than a window sized to the
/// whole strand, because a wider window is a different search.
fn route_edge(
    source: &Terminal,
    target: &Terminal,
    base: &SheetGeometry,
    laid: &[Segment],
    page: Rect,
    weights: &Routing,
) -> Option<Vec<Point>> {
    let window = Window::around(source.at, target.at, weights.margin, page, GRID);
    let mut segments: Vec<Segment> = base.segments.to_vec();
    segments.extend_from_slice(laid);
    let geometry = SheetGeometry {
        segments: &segments,
        ..*base
    };
    let obstacles = Obstacles::build(window, &geometry);
    if let Some(best) = Shapes::of(source, target, &obstacles, weights).best() {
        return Some(best.path.clone());
    }
    Search::of(source, target, &obstacles, weights)
        .route()
        .map(|route| route.path.clone())
}

/// Every grid point of a polyline, ends included.
fn points_of(path: &[Point]) -> Vec<Point> {
    let mut lattice = Lattice::default();
    if !lattice.add(path) {
        return path.to_vec();
    }
    lattice.adjacency().into_keys().collect()
}

// ---------------------------------------------------------------------------
// The two T7 rulings this gate reports on, whatever the deviation
// ---------------------------------------------------------------------------

/// Is this point inside the border and title block KiCad draws on the paper?
///
/// T7 ruled that nothing is deducted for either — a wire drawn there is legal
/// in the file, and kicli invents no boundary the editor does not draw — and
/// named this gate as the trigger to revisit. The region is the band outside
/// the drawing sheet's 10 mm margins, plus the title block in the bottom-right
/// corner inside them.
fn in_border_region(page: Rect, at: Point) -> bool {
    let inner = Rect::new(
        Point::new(
            page.start().x.0 + SHEET_MARGIN,
            page.start().y.0 + SHEET_MARGIN,
        ),
        Point::new(page.end().x.0 - SHEET_MARGIN, page.end().y.0 - SHEET_MARGIN),
    );
    if !inner.contains(at) {
        return true;
    }
    let title = Rect::new(
        Point::new(
            inner.end().x.0 - TITLE_BLOCK.0,
            inner.end().y.0 - TITLE_BLOCK.1,
        ),
        Point::new(
            inner.end().x.0 - TITLE_BLOCK.2,
            inner.end().y.0 - TITLE_BLOCK.3,
        ),
    );
    title.contains(at)
}

/// Is this point within one grid step of a bus entry, measured as the obstacle
/// map measures a halo — the ring one step outside a box?
fn within_one_step(at: Point, entry: (Point, Point)) -> bool {
    [entry.0, entry.1]
        .iter()
        .any(|end| (at.x.0 - end.x.0).abs() <= GRID.0 && (at.y.0 - end.y.0).abs() <= GRID.0)
}

// ---------------------------------------------------------------------------
// The gate
// ---------------------------------------------------------------------------

/// One sheet, re-routed and compared.
pub struct Calibration {
    label: String,
    page: Rect,
    measured: Vec<Measured>,
    skipped: Vec<Skipped>,
    sheet_pins: usize,
    skipped_pins: usize,
    bus_entries: usize,
    original_in_border: usize,
    reroute_in_border: usize,
    near_bus_entry: Vec<String>,
}

impl Calibration {
    /// Re-route every eligible strand of a sheet and cost both drawings.
    pub fn of(sheet: &Sheet) -> Self {
        let weights = Config::default().routing;
        let file = sheet.file();
        let drawing = read_drawing(sheet);
        let strands = strands_of(sheet, &drawing);
        let page = kicli::route::page_area(&file.doc);

        let mut run = Self {
            label: sheet.label.clone(),
            page,
            measured: Vec::new(),
            skipped: Vec::new(),
            sheet_pins: drawing.pins.len(),
            skipped_pins: 0,
            bus_entries: drawing.bus_entries.len(),
            original_in_border: 0,
            reroute_in_border: 0,
            near_bus_entry: Vec::new(),
        };

        for strand in &strands {
            match measure(file, &drawing, strand, &weights) {
                Ok(measured) => run.measured.push(measured),
                Err(why) => {
                    run.skipped_pins += strand.pins.len();
                    run.skipped.push(Skipped {
                        net: strand.net.clone(),
                        pins: strand.pins.len(),
                        why,
                    });
                }
            }
        }

        for measured in &run.measured {
            for wire in &measured.original_wires {
                if wire.iter().any(|&at| in_border_region(page, at)) {
                    run.original_in_border += 1;
                }
            }
            for path in &measured.paths {
                for pair in path.windows(2) {
                    let points = points_of(pair);
                    if points.iter().any(|&at| in_border_region(page, at)) {
                        run.reroute_in_border += 1;
                    }
                    for entry in &drawing.bus_entries {
                        if points.iter().any(|&at| within_one_step(at, *entry)) {
                            run.near_bus_entry.push(format!(
                                "{}: the segment {} \u{2192} {} passes within 1 G of the bus entry at {}",
                                measured.net, pair[0], pair[1], entry.0
                            ));
                        }
                    }
                }
            }
        }
        run
    }

    /// What the person's wires cost, over every measured strand.
    fn original(&self) -> i64 {
        self.measured.iter().map(|one| one.original_cost).sum()
    }

    /// What the router's wires cost, over the same strands.
    fn rerouted(&self) -> i64 {
        self.measured.iter().map(|one| one.rerouted_cost).sum()
    }

    /// How many strands were measured.
    pub fn strands(&self) -> usize {
        self.measured.len()
    }

    /// How many drawn wire records the measured strands hold.
    pub fn original_segments(&self) -> usize {
        self.measured.iter().map(|one| one.segments).sum()
    }

    /// The whole measurement, as the next reader needs to see it.
    pub fn report(&self) -> String {
        let mut out = String::new();
        let weights = Config::default().routing;
        let _ = writeln!(
            out,
            "\n=== re-route calibration: {} ===\npage {} × {} iu, {} pins, {} strands measured, \
             {} skipped\nweights: w_len {}, w_turn {}, w_cross {}, w_text {}, w_near {}; \
             margin {} iu, u_max {} iu, label_threshold {} iu",
            self.label,
            self.page.end().x.0 - self.page.start().x.0,
            self.page.end().y.0 - self.page.start().y.0,
            self.sheet_pins,
            self.measured.len(),
            self.skipped.len(),
            weights.w_len,
            weights.w_turn,
            weights.w_cross,
            weights.w_text,
            weights.w_near,
            weights.margin.0,
            weights.u_max.0,
            weights.label_threshold.0
        );
        let _ = writeln!(
            out,
            "\n{:<22} {:>4} {:>4} | {:>6} {:>4} {:>4} {:>4} {:>4} | {:>6} {:>4} {:>4} {:>4} {:>4} | {:>8}",
            "strand",
            "pins",
            "segs",
            "orig",
            "step",
            "turn",
            "xing",
            "text",
            "route",
            "step",
            "turn",
            "xing",
            "text",
            "deviation"
        );
        for one in &self.measured {
            let _ = writeln!(
                out,
                "{:<22} {:>4} {:>4} | {:>6} {:>4} {:>4} {:>4} {:>4} | {:>6} {:>4} {:>4} {:>4} {:>4} | {:>8}",
                elide(&one.net, 22),
                one.pins,
                one.segments,
                one.original_cost,
                one.original.steps,
                one.original.corners,
                one.original.crossings,
                one.original.text_steps,
                one.rerouted_cost,
                one.rerouted.steps,
                one.rerouted.corners,
                one.rerouted.crossings,
                one.rerouted.text_steps,
                percent(one.rerouted_cost - one.original_cost, one.original_cost)
            );
        }
        let _ = writeln!(
            out,
            "\ntotal: original {}, re-routed {}, deviation {} (tolerance ±{TOLERANCE_PERCENT} %)",
            self.original(),
            self.rerouted(),
            percent(self.rerouted() - self.original(), self.original())
        );

        let _ = writeln!(out, "\nskipped strands, with the reason for each:");
        if self.skipped.is_empty() {
            let _ = writeln!(out, "  (none)");
        }
        for one in &self.skipped {
            let _ = writeln!(out, "  {} ({} pins): {}", one.net, one.pins, one.why);
        }
        let _ = writeln!(
            out,
            "skipped pins {} of {} on the sheet; the budget is under {}/{}",
            self.skipped_pins, self.sheet_pins, SKIP_BUDGET.0, SKIP_BUDGET.1
        );

        let _ = writeln!(
            out,
            "\nruling report 1 — the border region (T7: clipped to the paper, nothing deducted \
             for the border and title block).\n  re-routed segments landing inside it: {}\n  \
             the same count for the wires the person drew, as the control: {}",
            self.reroute_in_border, self.original_in_border
        );
        let _ = writeln!(
            out,
            "\nruling report 2 — bus entries (T7: left out of the obstacle map, being diagonals \
             the lattice cannot describe).\n  bus entries on the sheet: {}\n  re-routed segments \
             passing within 1 G of one: {}",
            self.bus_entries,
            self.near_bus_entry.len()
        );
        for line in &self.near_bus_entry {
            let _ = writeln!(out, "    {line}");
        }
        out
    }

    /// The gate's own assertion: the total is within 15 %, in either direction.
    pub fn assert_within_tolerance(&self) {
        let (original, rerouted) = (self.original(), self.rerouted());
        assert!(original > 0, "the sheet drew nothing to compare against");
        let difference = (rerouted - original).abs();
        assert!(
            difference * 100 <= TOLERANCE_PERCENT * original,
            "{}: re-routing costs {rerouted} against the person's {original}, a deviation of {}. \
             The weights are not retuned in M4 — report this and park it.\n{}",
            self.label,
            percent(rerouted - original, original),
            self.report()
        );
    }

    /// The assertion that makes the rest of the gate honest.
    pub fn assert_skipped_pins_are_a_minority(&self) {
        assert!(
            self.skipped_pins * SKIP_BUDGET.1 < self.sheet_pins * SKIP_BUDGET.0,
            "{}: {} of {} pins are on strands this gate skipped, which is not under {}/{}. \
             A gate that skips the hard nets measures the easy ones.\n{}",
            self.label,
            self.skipped_pins,
            self.sheet_pins,
            SKIP_BUDGET.0,
            SKIP_BUDGET.1,
            self.report()
        );
    }
}

/// A signed deviation, in integer tenths of a per cent.
fn percent(difference: i64, base: i64) -> String {
    if base == 0 {
        return "n/a".to_owned();
    }
    let tenths = difference * 1000 / base;
    let sign = if tenths < 0 { "-" } else { "+" };
    format!("{sign}{}.{} %", tenths.abs() / 10, tenths.abs() % 10)
}

/// A name cut to fit a column.
fn elide(name: &str, width: usize) -> String {
    if name.chars().count() <= width {
        return name.to_owned();
    }
    name.chars().take(width - 1).chain(['…']).collect()
}

// ---------------------------------------------------------------------------
// The five steps of the procedure, in order
// ---------------------------------------------------------------------------

/// Measure one strand, or say why it was left out.
///
/// The five steps the task entry fixes, in its order:
///
/// 1. take the strands with two or more pins and no bus involvement;
/// 2. cost the **existing** wires against the sheet with the strand's own
///    drawing removed;
/// 3. re-route: a spanning tree over the pins by Manhattan distance, ties on
///    `(x, y)`, each edge routed in order and added to the map before the next;
/// 4. cost the result on **the same map** the original was costed on;
/// 5. everything declined is named with its reason, upstream.
fn measure(
    file: &LoadedFile,
    drawing: &Drawing,
    strand: &Strand,
    weights: &Routing,
) -> Result<Measured, String> {
    // 1. Eligibility. Bundle involvement is asked first, because a strand that
    // feeds a bus is out of the gate whatever else is true of it, and naming
    // its pin count instead would hide the reason that matters.
    if let Some(why) = bus_involvement(drawing, strand) {
        return Err(why);
    }
    if strand.pins.len() < 2 {
        return Err(format!(
            "{} pin(s) on this strand, so there is no connection to re-route",
            strand.pins.len()
        ));
    }
    if strand.wires.is_empty() {
        return Err("no wire joins its pins, so the person drew nothing to compare".to_owned());
    }
    if let Some(pin) = strand.pins.iter().find(|pin| !pin.at.is_on_grid()) {
        return Err(format!(
            "{} is off the placement grid at {}",
            pin.handle, pin.at
        ));
    }

    // 2. The map: the sheet, with this strand's own drawing taken out of it.
    let trimmed = without_the_strands_own_marks(file, strand, drawing);
    let own_wires: Vec<Uuid> = strand
        .wires
        .iter()
        .map(|&index| drawing.wires[index].uuid.clone())
        .collect();
    let own_terminals: Vec<String> = strand.pins.iter().map(|pin| pin.handle.clone()).collect();
    let sheet_path = strand.sheet_path.clone();
    let objects = SheetObjects::read(
        &trimmed,
        &sheet_path,
        &Routed {
            wires: &own_wires,
            terminals: &own_terminals,
        },
    );
    let geometry = objects.geometry();
    let page = objects.page();

    let mut original = Lattice::default();
    for &index in &strand.wires {
        let wire = &drawing.wires[index];
        if !original.add(&[wire.from, wire.to]) {
            return Err(format!(
                "the wire {} → {} is diagonal or off the lattice",
                wire.from, wire.to
            ));
        }
    }
    let terminals: BTreeSet<Point> = strand.pins.iter().map(|pin| pin.at).collect();
    let corners = window_corners(&original, &terminals);
    let window = Window::around(corners.0, corners.1, weights.margin, page, GRID);
    let obstacles = Obstacles::build(window, &geometry);
    let original_tally = cost_of_drawing(&original, &obstacles, &terminals).map_err(|refused| {
        format!(
            "the drawn wire is refused by {} at {}",
            refused.handle, refused.at
        )
    })?;

    // 3. The re-route.
    let mut laid: Vec<Segment> = Vec::new();
    let mut paths: Vec<Vec<Point>> = Vec::new();
    for (one, other) in spanning_tree(&strand.pins) {
        let (source, target) = (&strand.pins[one], &strand.pins[other]);
        let Some(path) = route_edge(
            &source.terminal,
            &target.terminal,
            &geometry,
            &laid,
            page,
            weights,
        ) else {
            return Err(format!(
                "no route joins {} to {}, which the person drew",
                source.handle, target.handle
            ));
        };
        // `routing.label_threshold` decides whether a connection is drawn at
        // all: over it the router proposes a pair of labels instead
        // (`research/wire-routing.md` §5.5). A proposal draws no wire, so there
        // is no wire cost to set against the person's — and costing it as zero
        // would be the "cheap is an anomaly" failure this gate exists to catch.
        // So the strand leaves the measurement, named, and its pins count
        // against the quarter like any other skip.
        if let Some(trigger) = Trigger::of(Some(walked(&path)), weights) {
            return Err(format!(
                "the router proposes labels rather than a wire from {} to {}: {}",
                source.handle,
                target.handle,
                trigger.reason(&target.handle)
            ));
        }
        for pair in path.windows(2) {
            laid.push(Segment {
                handle: format!("re-route {}", laid.len()),
                from: pair[0],
                to: pair[1],
                own_net: true,
            });
        }
        paths.push(path);
    }

    // 4. The same map, the same function.
    let mut rerouted = Lattice::default();
    for path in &paths {
        if !rerouted.add(path) {
            return Err(
                "the router produced a path this gate cannot lay on the lattice".to_owned(),
            );
        }
    }
    let rerouted_tally = cost_of_drawing(&rerouted, &obstacles, &terminals).map_err(|refused| {
        format!(
            "the re-route is refused by {} at {}, on the map it was routed against",
            refused.handle, refused.at
        )
    })?;

    Ok(Measured {
        net: strand.net.clone(),
        pins: strand.pins.len(),
        segments: strand.wires.len(),
        original_cost: kicli::route::Cost::of(original_tally, weights).total(),
        original: original_tally,
        rerouted_cost: kicli::route::Cost::of(rerouted_tally, weights).total(),
        rerouted: rerouted_tally,
        original_wires: strand
            .wires
            .iter()
            .map(|&index| points_of(&[drawing.wires[index].from, drawing.wires[index].to]))
            .collect(),
        paths,
    })
}

/// Does this strand touch a bundle?
///
/// A bus entry is a diagonal the lattice cannot describe and a bus carries a
/// bundle this milestone does not route, so a strand that meets either is
/// outside the gate rather than measured wrongly.
fn bus_involvement(drawing: &Drawing, strand: &Strand) -> Option<String> {
    for &index in &strand.wires {
        let wire = &drawing.wires[index];
        for end in [wire.from, wire.to] {
            if drawing
                .bus_entries
                .iter()
                .any(|entry| entry.0 == end || entry.1 == end)
            {
                return Some(format!("a wire end at {end} meets a bus entry"));
            }
            if drawing
                .buses
                .iter()
                .any(|bus| on_segment(bus.from, bus.to, end))
            {
                return Some(format!("a wire end at {end} lands on a bus"));
            }
        }
    }
    None
}

/// The two corners the strand's own window is built from.
///
/// Every point of the drawn strand and every pin it reaches, so that the wires
/// being costed are all inside the window that costs them. Each re-routed edge
/// then gets its own request-sized window, which this one contains.
fn window_corners(lattice: &Lattice, terminals: &BTreeSet<Point>) -> (Point, Point) {
    let mut points: Vec<Point> = lattice.adjacency().into_keys().collect();
    points.extend(terminals.iter().copied());
    let smallest = |pick: fn(&Point) -> i32| points.iter().map(pick).min().unwrap_or_default();
    let largest = |pick: fn(&Point) -> i32| points.iter().map(pick).max().unwrap_or_default();
    (
        Point::new(smallest(|at| at.x.0), smallest(|at| at.y.0)),
        Point::new(largest(|at| at.x.0), largest(|at| at.y.0)),
    )
}

// ---------------------------------------------------------------------------
// The two checks
// ---------------------------------------------------------------------------

/// The purpose-built in-repo fixture, which gates the default run.
fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sch/routing/calibration.kicad_sch")
}

#[test]
fn re_routing_a_known_good_sheet_costs_what_the_original_did() {
    let sheet = Sheet::root_of(&fixture());
    let measured = Calibration::of(&sheet);
    println!("{}", measured.report());
    measured.assert_skipped_pins_are_a_minority();
    measured.assert_within_tolerance();
    assert!(
        measured.strands() >= 20,
        "the calibration fixture carries {} measured strands, and the task asks for 20",
        measured.strands()
    );
    assert!(
        measured.original_segments() >= 40,
        "the calibration fixture carries {} measured segments, and the task asks for 40",
        measured.original_segments()
    );
}

/// The demo sheet the corpus run gates on.
///
/// `ampli_ht` is `research/wire-routing.md` §2's tidy hand-drawn sheet: 81
/// wires, 13 junctions, 1 crossing, 0.12 crossings per ten wires. It is a child
/// of `complex_hierarchy`, and it is loaded through its parent because a
/// reference designator belongs to a placement.
#[cfg(feature = "corpus")]
fn demo_sheet() -> Option<Sheet> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/corpus/demos/complex_hierarchy/complex_hierarchy.kicad_sch");
    if !root.is_file() {
        return None;
    }
    Some(Sheet::child_of(&root, "ampli_ht.kicad_sch").expect("the demo places ampli_ht"))
}

#[cfg(feature = "corpus")]
#[test]
fn re_routing_a_demo_sheet_costs_what_the_original_did() {
    let Some(sheet) = demo_sheet() else {
        eprintln!("skipped: the corpus is not there. Run `cargo xtask corpus` first.");
        return;
    };
    let measured = Calibration::of(&sheet);
    println!("{}", measured.report());
    measured.assert_skipped_pins_are_a_minority();
    measured.assert_within_tolerance();
}

// ---------------------------------------------------------------------------
// The fixture, and the recipe it is built from
// ---------------------------------------------------------------------------
//
// The fixture is drawn by the recipe below rather than by hand, because a
// hand-typed sheet of forty segments carries a typo nobody sees. **The drawing
// rule is stated before the router is consulted and is not adjusted
// afterwards**: every connection is drawn as the shortest orthogonal path with
// the fewest corners that honours both pins' escape directions, laid clear of
// the symbol bodies. A fixture tuned until the router agrees with it measures
// nothing, so the rule comes first and the number comes second.
//
// Provenance: written by this recipe, then canonicalised with
// `kicad-cli sch upgrade --force`, which is what the MANIFEST records as
// `kicad-cli`.

/// One grid step, in millimetres. Every coordinate below is a whole number of
/// these, so nothing in the drawing can land off the placement grid.
const STEP: f64 = 1.27;

/// A position in grid steps, as the two strings a probe writes.
fn at(x: i32, y: i32) -> (String, String) {
    (
        millimetres(f64::from(x) * STEP),
        millimetres(f64::from(y) * STEP),
    )
}

/// A four-pin chip with a body: two pins on the left, two on the right.
///
/// Pin 1 and pin 2 leave to the left, pin 3 and pin 4 to the right. Placed at
/// 90° the same symbol's pins leave up and down, which is what makes an L
/// between two chips possible without a resistor in the middle.
fn chip() -> String {
    symbol(
        "IC",
        "U",
        false,
        &[(
            "1_1",
            vec![
                rectangle(("-7.62", "-7.62"), ("7.62", "7.62")),
                pin("passive", ("-10.16", "5.08"), "0", "1", "A"),
                pin("passive", ("-10.16", "-5.08"), "0", "2", "B"),
                pin("passive", ("10.16", "5.08"), "180", "3", "C"),
                pin("passive", ("10.16", "-5.08"), "180", "4", "D"),
            ],
        )],
    )
}

/// A ground symbol: one power input at the anchor, the graphic below it, so a
/// wire leaves it upwards.
fn ground() -> String {
    symbol(
        "GND",
        "#PWR",
        true,
        &[("1_1", vec![pin("power_in", ("0", "0"), "270", "1", "GND")])],
    )
}

/// The drawing, item by item.
///
/// Four bands: straight runs and L-shapes to ground; staggered chips joined by
/// Z-shapes; three groups of a chip and a chip turned on its side, joined by
/// L-shapes; and one chip feeding a bus through two bus entries, which is the
/// part the gate is expected to decline.
pub fn draw(directory: PathBuf) -> Probe {
    let mut probe = Probe::new("route-calibration", directory);
    probe.define(chip());
    probe.define(ground());

    let mut grounds = 0;
    let mut chips = 0;

    // A chip at a position, at an angle, with a reference of its own.
    let mut place_chip = |probe: &mut Probe, x: i32, y: i32, angle: &str| {
        chips += 1;
        let reference = format!("U{chips}");
        let position = at(x, y);
        let mut placed = Placed::new(
            "IC",
            &reference,
            (&position.0, &position.1),
            &["1", "2", "3", "4"],
        );
        placed.angle = angle;
        probe.place_symbol(&placed);
    };

    // Band 1: four chips in a row, joined pin to pin.
    for column in 0..4 {
        place_chip(&mut probe, 30 + 36 * column, 20, "0");
    }
    // Band 2: four chips, staggered, so every join is a Z.
    for column in 0..4 {
        let y = if column % 2 == 0 { 48 } else { 56 };
        place_chip(&mut probe, 30 + 36 * column, y, "0");
    }
    // Band 3: three groups of a chip and a chip turned on its side.
    for group in 0..3 {
        place_chip(&mut probe, 30 + 70 * group, 90, "0");
        place_chip(&mut probe, 60 + 70 * group, 106, "90");
    }
    // Band 4: the chip that feeds the bus.
    place_chip(&mut probe, 30, 140, "0");

    let mut place_ground = |probe: &mut Probe, x: i32, y: i32| {
        grounds += 1;
        let reference = format!("#PWR{grounds:02}");
        let position = at(x, y);
        probe.place_unit(
            "GND",
            &reference,
            (&position.0, &position.1),
            1,
            "GND",
            &["1"],
        );
    };

    // The grounds, in the order the wires below reach them.
    let ground_sites = [
        // Band 1 ends.
        (14, 26),
        (18, 34),
        (154, 26),
        (150, 34),
        // Band 2 ends.
        (14, 54),
        (18, 62),
        (154, 62),
        (150, 70),
        // Band 3, four per group.
        (14, 96),
        (18, 104),
        (56, 120),
        (64, 120),
        (84, 96),
        (88, 104),
        (126, 120),
        (134, 120),
        (154, 96),
        (158, 104),
        (196, 120),
        (204, 120),
        // Band 4.
        (14, 150),
        (18, 152),
    ];
    for &(x, y) in &ground_sites {
        place_ground(&mut probe, x, y);
    }

    for polyline in wires() {
        for pair in polyline.windows(2) {
            let (from, to) = (at(pair[0].0, pair[0].1), at(pair[1].0, pair[1].1));
            probe.wire((&from.0, &from.1), (&to.0, &to.1));
        }
    }

    // Three names, each on the wire it names, as a person writes them.
    for (index, (x, y)) in [(52, 16), (52, 24), (124, 16)].iter().enumerate() {
        let anchor = at(*x, *y);
        probe.label_of_kind(
            LabelKind::Local,
            &format!("BAND1_{index}"),
            (&anchor.0, &anchor.1),
        );
    }
    // One note, in clear space, so the sheet carries a text box the map holds.
    let note = at(70, 76);
    probe.free_text("calibration sheet", (&note.0, &note.1));

    // The bus and the two entries that feed it.
    let (bus_top, bus_bottom) = (at(66, 130), at(66, 150));
    probe.bus((&bus_top.0, &bus_top.1), (&bus_bottom.0, &bus_bottom.1));
    for (y, dy) in [(136, "2.54"), (144, "-2.54")] {
        let entry = at(64, y);
        probe.bus_entry((&entry.0, &entry.1), ("2.54", dy));
    }

    probe
}

/// Every wire of the fixture, as polylines in grid steps.
///
/// Drawn to the rule stated at the top of this file: shortest, fewest corners,
/// both escapes honoured, clear of the bodies.
fn wires() -> Vec<Vec<(i32, i32)>> {
    let mut all: Vec<Vec<(i32, i32)>> = Vec::new();

    // Band 1: three straight runs on each of the two pin rows.
    for column in 0..3 {
        let left = 30 + 36 * column;
        all.push(vec![(left + 8, 16), (left + 28, 16)]);
        all.push(vec![(left + 8, 24), (left + 28, 24)]);
    }
    // Band 1: the four outer pins, each down to its own ground.
    all.push(vec![(22, 16), (14, 16), (14, 26)]);
    all.push(vec![(22, 24), (18, 24), (18, 34)]);
    all.push(vec![(146, 16), (154, 16), (154, 26)]);
    all.push(vec![(146, 24), (150, 24), (150, 34)]);

    // Band 2: six Z-shapes. The middle line of the upper Z and of the lower one
    // are put on opposite sides of the span, so the two never share a run.
    for column in 0..3 {
        let left = 30 + 36 * column;
        let (top, bottom) = if column % 2 == 0 { (48, 56) } else { (56, 48) };
        let (upper_middle, lower_middle) = if bottom > top { (14, 6) } else { (6, 14) };
        let start = left + 8;
        let end = left + 28;
        all.push(vec![
            (start, top - 4),
            (start + upper_middle, top - 4),
            (start + upper_middle, bottom - 4),
            (end, bottom - 4),
        ]);
        all.push(vec![
            (start, top + 4),
            (start + lower_middle, top + 4),
            (start + lower_middle, bottom + 4),
            (end, bottom + 4),
        ]);
    }
    // Band 2: the four outer pins to ground.
    all.push(vec![(22, 44), (14, 44), (14, 54)]);
    all.push(vec![(22, 52), (18, 52), (18, 62)]);
    all.push(vec![(146, 52), (154, 52), (154, 62)]);
    all.push(vec![(146, 60), (150, 60), (150, 70)]);

    // Band 3: three groups, each a chip and a chip on its side.
    for group in 0..3 {
        let base = 30 + 70 * group;
        // The two L-shapes between the two chips, crossing nothing.
        all.push(vec![(base + 8, 86), (base + 34, 86), (base + 34, 98)]);
        all.push(vec![(base + 8, 94), (base + 26, 94), (base + 26, 98)]);
        // The upright chip's left pins, out to ground.
        all.push(vec![(base - 8, 86), (base - 16, 86), (base - 16, 96)]);
        all.push(vec![(base - 8, 94), (base - 12, 94), (base - 12, 104)]);
        // The side-on chip's lower pins, straight down to ground.
        all.push(vec![(base + 26, 114), (base + 26, 120)]);
        all.push(vec![(base + 34, 114), (base + 34, 120)]);
    }

    // Band 4: two pins into the bus, two out to ground.
    all.push(vec![(38, 136), (64, 136)]);
    all.push(vec![(38, 144), (64, 144)]);
    all.push(vec![(22, 136), (14, 136), (14, 150)]);
    all.push(vec![(22, 144), (18, 144), (18, 152)]);

    all
}

/// Where the committed fixture lives.
pub fn committed() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sch/routing/calibration.kicad_sch")
}

/// Build the fixture into a scratch directory and canonicalise it.
///
/// Returns nothing when `kicad-cli` is not asked for: the canonical form is
/// KiCad's, and a fixture this recipe wrote but KiCad never rewrote is not the
/// committed one.
pub fn rebuild() -> Option<PathBuf> {
    std::env::var("KICLI_TEST_KICAD_CLI").ok()?;
    let binary = std::env::var("KICLI_KICAD_CLI").unwrap_or_else(|_| "kicad-cli".to_owned());
    let probe = draw(PathBuf::from(env!("CARGO_TARGET_TMPDIR")));
    let written = probe.write();
    let status = Command::new(binary)
        .args(["sch", "upgrade", "--force"])
        .arg(&written)
        .status()
        .ok()?;
    status.success().then_some(written)
}

/// The committed fixture is what the recipe builds.
///
/// A fixture and a recipe that have drifted apart are two claims about one
/// drawing, and the file is the one that gets measured. This runs only where
/// `kicad-cli` is asked for, because the canonical bytes are KiCad's.
#[test]
fn the_calibration_fixture_is_what_the_recipe_builds() {
    let Some(fresh) = rebuild() else {
        eprintln!("skipped: set KICLI_TEST_KICAD_CLI to rebuild the calibration fixture");
        return;
    };
    let built = std::fs::read_to_string(&fresh).expect("the rebuilt fixture reads");
    let held = std::fs::read_to_string(committed()).expect("the committed fixture reads");
    assert_eq!(
        built,
        held,
        "the committed calibration fixture is not what the recipe builds. \
         The recipe is at {}, the rebuild at {}",
        file!(),
        fresh.display()
    );
}

// ---------------------------------------------------------------------------
// The fixture's own connectivity, as KiCad states it
// ---------------------------------------------------------------------------

/// KiCad's own netlist of the fixture, committed beside it.
fn oracle() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sch/routing/calibration.netlist")
}

/// The strands this gate measures are the nets KiCad reads.
///
/// The gate stands on kicli's extractor to say which pins share a conductor,
/// and the extractor is only worth standing on where KiCad agrees. This is the
/// oracle check the new fixture owes: the partition kicli reads out of it is
/// the partition `kicad-cli sch export netlist` writes.
#[test]
fn the_calibration_fixture_partitions_as_kicad_says() {
    let committed = std::fs::read_to_string(oracle()).expect("the oracle is readable");
    let kicad = Netlist::parse(&committed).partition();
    let hierarchy = Hierarchy::load(&fixture()).expect("the fixture loads");
    let kicli = kicli_partition(&extract(&hierarchy));
    assert!(
        differences(&kicli, &kicad).is_none(),
        "{}",
        differences(&kicli, &kicad).unwrap_or_default()
    );
}

/// The committed oracle is still what KiCad answers.
#[test]
fn the_calibration_oracle_is_current() {
    let Some(tool) = Kicad::found_or_skip("regenerate the calibration oracle") else {
        return;
    };
    let fixtures = Fixtures::new(env!("CARGO_TARGET_TMPDIR"), env!("CARGO_MANIFEST_DIR"));
    let copy = fixtures.copy_project("calibration-oracle", &fixture());
    let fresh = tool.netlist(
        &copy,
        &fixtures
            .scratch("calibration-oracle-fresh")
            .join("calibration.netlist"),
    );
    let committed = std::fs::read_to_string(oracle()).expect("the oracle is readable");
    let now = fresh.partition();
    assert!(
        differences(&Netlist::parse(&committed).partition(), &now).is_none(),
        "the committed netlist is stale: {}",
        differences(&Netlist::parse(&committed).partition(), &now).unwrap_or_default()
    );
}

// ---------------------------------------------------------------------------
// The two ruling reports' instruments, shown able to answer anything but zero
// ---------------------------------------------------------------------------

/// The border region is the band KiCad's own drawing sheet paints.
///
/// Both sheets answer zero for the ruling report, so the counter has to be
/// shown able to answer something else — otherwise "no re-routed segment lands
/// in the border" and "the counter is broken" read the same.
#[test]
fn the_border_region_is_the_band_kicad_draws() {
    // A4 landscape, as both calibration sheets use.
    let page = Rect::new(Point::default(), Point::new(2_970_022, 2_100_072));
    // One millimetre in from the paper's edge is inside the border.
    assert!(in_border_region(page, Point::new(10_000, 10_000)));
    assert!(in_border_region(page, Point::new(2_960_022, 2_090_072)));
    // Just inside the 10 mm margin is not.
    assert!(!in_border_region(page, Point::new(110_000, 110_000)));
    assert!(!in_border_region(page, Point::new(1_000_000, 1_000_000)));
    // The title block sits inside the margin and is border all the same: it is
    // 110 mm wide and 34 mm tall in the bottom-right corner of the margin box.
    let inner_right = 2_970_022 - SHEET_MARGIN;
    let inner_bottom = 2_100_072 - SHEET_MARGIN;
    assert!(in_border_region(
        page,
        Point::new(inner_right - 500_000, inner_bottom - 100_000)
    ));
    // A hand's breadth above the title block is clear again.
    assert!(!in_border_region(
        page,
        Point::new(inner_right - 500_000, inner_bottom - 400_000)
    ));
}

/// The bus-entry halo is one grid step, measured as the obstacle map measures
/// the ring around a body.
#[test]
fn a_bus_entry_halo_is_one_grid_step() {
    let entry = (Point::new(0, 0), Point::new(GRID.0, GRID.0));
    assert!(within_one_step(Point::new(GRID.0, 0), entry));
    assert!(within_one_step(Point::new(2 * GRID.0, GRID.0), entry));
    assert!(!within_one_step(Point::new(3 * GRID.0, 0), entry));
    assert!(!within_one_step(Point::new(0, -2 * GRID.0), entry));
}

/// The whole report is a function of the drawing, not of the file's item order.
///
/// The hazard the environment-variation break names: this gate builds a
/// spanning tree with ties broken on `(x, y)` and routes its edges in order, so
/// anything that reaches the answer through the order a file happens to list
/// its objects in is a calibration number that is not one. The comparison is of
/// the **whole report**, not of the totals: the run-order break found totals
/// that agreed and rows that did not, which is a report no reader can diff.
#[test]
fn the_report_does_not_depend_on_the_order_the_file_lists_its_objects() {
    let forwards = Calibration::of(&Sheet::root_of(&fixture())).report();
    let backwards = Calibration::of(&Sheet::root_of(&fixture()).with_objects_reversed()).report();
    assert_eq!(forwards, backwards);
}

/// The same, on the demo sheet, which has junctions and branching strands the
/// fixture does not.
#[cfg(feature = "corpus")]
#[test]
fn the_demo_report_does_not_depend_on_the_order_the_file_lists_its_objects() {
    let Some(sheet) = demo_sheet() else {
        eprintln!("skipped: the corpus is not there. Run `cargo xtask corpus` first.");
        return;
    };
    let forwards = Calibration::of(&sheet).report();
    let sheet = demo_sheet().expect("the corpus is still there");
    let backwards = Calibration::of(&sheet.with_objects_reversed()).report();
    assert_eq!(forwards, backwards);
}
