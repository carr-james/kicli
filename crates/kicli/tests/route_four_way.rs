//! The router never draws the fourth wire end into one point.
//!
//! `spec/SPEC.md` §9 Q2: a route that would terminate where three wire ends
//! already meet is refused and offset by one grid step, reporting the
//! adjustment. The refusal it is measured against is the one `edit::mark`
//! already carries — a junction on four wire ends draws one dot that four wires
//! run into, and a reader cannot tell which pair the designer meant to join.
//!
//! There is no routing verb yet, so "routing" here is the composition a route
//! request drives, as `tests/route_determinism.rs` drives it: settle the
//! terminals against the drawing, read the sheet into the router's lists, build
//! the window and the obstacle map, and ask the shapes. The one new step is the
//! first, and it is the step under test.
//!
//! **The drawing.** Three wires meet at `P`, which is therefore a three-end
//! point and a legal place for a junction today. `R1.1` is routed to `P`. The
//! route's own end would be the fourth, so the terminus moves one grid step
//! east along the wire that already runs east from `P`, and lands on that
//! wire's interior — still on the net it was asked for.
//!
//! The netlist oracle runs only with `KICLI_TEST_KICAD_CLI` set. Without it the
//! connectivity claim is kicli's own extractor; with it, KiCad is asked about
//! the file kicli wrote.

use std::path::{Path, PathBuf};

use kicli::connectivity::extract;
use kicli::edit::mark::{MarkError, PinAddress, add_junction};
use kicli::edit::wire::{End, Polyline, draw};
use kicli::geometry::{GRID, Iu, Point, resolve_pins};
use kicli::model::items::{SheetPath, Uuid};
use kicli::model::{
    Config, Hierarchy, LoadedFile, Refdes, Target, WriteOptions, definition_of, read_library,
};
use kicli::route::report::{Adjusted, Adjustment, Report, Status};
use kicli::route::terminal::Approach;
use kicli::route::{Obstacles, Routed, Shapes, SheetObjects, Terminal, Window};
use kicli_probe::Probe;
use kicli_probe::oracle::{Kicad, Partition, differences, kicli_partition, net};

/// Where this binary writes the drawings it builds.
fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("route-four-way")
}

/// A point from two millimetre readings, as a KiCad file writes them.
fn at(x: &str, y: &str) -> Point {
    let read = |text: &str| {
        Iu::from_millimetres_text(text)
            .unwrap_or_else(|| panic!("{text} is a millimetre reading"))
            .0
    };
    Point::new(read(x), read(y))
}

/// The point three wires meet at, which a fourth end would make a four-way
/// junction.
fn meeting_point() -> Point {
    at("114.3", "88.9")
}

/// Which arms of the meeting point a drawing carries.
///
/// The three-armed drawing is the case under test. The two-armed one is the
/// boundary control: one wire end fewer, and the route's own end is the third
/// rather than the fourth, so nothing may move.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arms {
    /// North, south and east: three ends, and the route's own would be the
    /// fourth.
    Three,
    /// South and east: two ends, and the route's own would be the third.
    Two,
    /// The three arms written in a different order in the file.
    ///
    /// KiCad reorders items when it saves, so the order a file lists its wires
    /// in is not a stable input. The offset must not depend on it.
    ThreeReordered,
}

/// The drawing this check routes over.
///
/// `R1.1` is the source, one grid step east of the south arm and facing north.
/// `R2.1` is what the meeting point's east arm leads to, so the netlist has a
/// two-pin net to claim rather than a dangling one.
fn drawing(name: &str, arms: Arms) -> Probe {
    let mut probe = Probe::new(name, scratch());
    probe.place("R", "R1", ("115.57", "105.41"), &["1", "2"]);
    probe.place("R", "R2", ("127", "92.71"), &["1", "2"]);
    // The arms, each an end at the meeting point.
    let north = |probe: &mut Probe| probe.wire(("114.3", "88.9"), ("114.3", "76.2"));
    let south = |probe: &mut Probe| probe.wire(("114.3", "88.9"), ("114.3", "101.6"));
    let east = |probe: &mut Probe| probe.wire(("114.3", "88.9"), ("127", "88.9"));
    match arms {
        Arms::Three => {
            north(&mut probe);
            south(&mut probe);
            east(&mut probe);
        }
        Arms::Two => {
            south(&mut probe);
            east(&mut probe);
        }
        Arms::ThreeReordered => {
            east(&mut probe);
            north(&mut probe);
            south(&mut probe);
        }
    }
    probe
}

/// Load a written drawing as a project rooted at it.
fn loaded(path: &Path) -> Hierarchy {
    Hierarchy::load(path).expect("the drawing loads")
}

/// The root placement of a loaded drawing, and the file it draws.
fn root(hierarchy: &Hierarchy) -> (&LoadedFile, &SheetPath) {
    let placement = hierarchy
        .placements
        .first()
        .expect("the root sheet is placed");
    (&hierarchy.files[placement.file], &placement.path)
}

/// The target a request writes through: the file, its directory, its sheet.
fn target<'a>(path: &'a Path, project: &'a Path, sheet: &'a SheetPath) -> Target<'a> {
    Target {
        path,
        project,
        sheet_path: sheet,
        grid: GRID,
        options: WriteOptions::default(),
    }
}

/// One pin of one placed symbol, as a terminal.
fn pin_terminal(hierarchy: &Hierarchy, reference: &str, number: &str) -> Terminal {
    let (file, path) = root(hierarchy);
    let schematic = &file.schematic;
    let library = read_library(&file.doc, &schematic.library_symbols, schematic.version);
    for symbol in schematic.symbols() {
        if symbol.reference_on(path).map(|found| found.0.as_str()) != Some(reference) {
            continue;
        }
        let definition = definition_of(&library, symbol).expect("the definition is embedded");
        for resolved in resolve_pins(&symbol.drawn_on(path), definition) {
            if resolved.number == number {
                return Terminal::of_pin(reference, &resolved);
            }
        }
    }
    panic!("{reference}.{number} is on this drawing");
}

/// Every wire of the drawing, which is the net the route is asked to join.
fn every_wire(hierarchy: &Hierarchy) -> Vec<Uuid> {
    root(hierarchy)
        .0
        .schematic
        .lines()
        .map(|line| line.uuid.clone())
        .collect()
}

/// What one route request answers: the report, and the terminals it used.
///
/// The composition in full. `Approach::of` settles the two terminals against
/// the drawing first, and everything after it is asked about the terminals it
/// answered with — which is what makes the offset a routing decision rather
/// than a note appended to one.
fn route(hierarchy: &Hierarchy, source: &Terminal, target: &Terminal) -> (Report, Approach) {
    let (file, path) = root(hierarchy);
    let routing = Config::default().routing;
    let approach = Approach::of(source, target, &file.schematic, GRID);
    let named = [approach.source.name.clone(), approach.target.name.clone()];
    let wires = every_wire(hierarchy);
    let objects = SheetObjects::read(
        file,
        path,
        &Routed {
            wires: &wires,
            terminals: &named,
        },
    );
    let window = Window::around(
        approach.source.at,
        approach.target.at,
        routing.margin,
        objects.page(),
        GRID,
    );
    let obstacles = Obstacles::build(window, &objects.geometry());
    let shapes = Shapes::of(&approach.source, &approach.target, &obstacles, &routing);
    let best = shapes
        .best()
        .unwrap_or_else(|| panic!("a route was found: blocked by {:?}", shapes.blocked_by()));

    let mut report = Report::of(Status::Routed, &approach.source.name, &approach.target.name);
    report.path.clone_from(&best.path);
    report.tally = best.tally;
    report.cost = best.cost;
    report.adjusted.clone_from(&approach.adjusted);
    report.alternatives_considered = shapes.considered();
    (report, approach)
}

/// Ask for a junction at a point, and answer with the refusal or nothing.
///
/// Nothing is written either way in the refusing case: `add_junction` checks
/// before it edits.
fn junction_at(path: &Path, point: Point) -> Result<(), MarkError> {
    let mut hierarchy = loaded(path);
    let sheet = hierarchy.placements[0].path.clone();
    let project = path.parent().expect("the drawing sits in a directory");
    add_junction(
        &mut hierarchy,
        point,
        &Uuid("00000000-0000-4000-8009-000000000001".to_owned()),
        &target(path, project, &sheet),
        "after",
    )
    .map(|_| ())
}

#[test]
fn the_router_never_makes_a_four_way_junction() {
    let point = meeting_point();
    let path = drawing("four-way", Arms::Three).write();
    let hierarchy = loaded(&path);
    let source = pin_terminal(&hierarchy, "R1", "1");
    let target = Terminal::of_point(point, "NET");
    let (report, approach) = route(&hierarchy, &source, &target);

    // The terminus moved one grid step east, along the wire that already runs
    // east out of the meeting point. East is the first of `Heading::EVERY` that
    // a wire leaves the point by.
    let landed = at("115.57", "88.9");
    assert_eq!(
        approach.target.at, landed,
        "the terminus is one grid step off the point three wires meet at"
    );
    assert_ne!(approach.target.at, point, "it did not stay on the point");
    assert_eq!(
        approach.source.at, source.at,
        "the source carries no wire end and does not move"
    );

    // The route lands there, which is the half of the rule the report cannot
    // stand in for: a report that names an adjustment the path did not make
    // describes a drawing nobody drew.
    assert_eq!(
        report.path.last().copied(),
        Some(landed),
        "the route ends where the terminal was moved to"
    );
    assert_ne!(
        report.path.last().copied(),
        Some(point),
        "the route does not end on the four-way point"
    );

    // And the report names it, which is the other half: an offset nobody is
    // told about is a wire that silently does not end where it was asked to.
    assert_eq!(
        report.adjusted,
        vec![Adjusted {
            terminal: "NET".to_owned(),
            by: Point::new(GRID.0, 0),
            why: Adjustment::FourWayJunction,
        }],
        "the report names the terminal, the displacement and the reason"
    );
    assert_eq!(
        report.adjusted[0].terminal, report.to,
        "it names the terminal as `to` does"
    );
    // `by` is a displacement, so the point that was asked for is recoverable
    // from the path without the contract storing it twice.
    assert_eq!(
        report.path.last().copied().expect("the route has an end") - report.adjusted[0].by,
        point,
        "the requested point is the terminus less the offset"
    );

    // The boundary. One wire end fewer at the meeting point and the route's own
    // end is the third rather than the fourth, so nothing moves and nothing is
    // reported. Without this arm a rule that fired on any wire end at all would
    // pass every assertion above.
    let two = drawing("four-way-two-arms", Arms::Two).write();
    let hierarchy = loaded(&two);
    let source = pin_terminal(&hierarchy, "R1", "1");
    let (report, approach) = route(&hierarchy, &source, &Terminal::of_point(point, "NET"));
    assert_eq!(
        approach.target.at, point,
        "two wire ends leave room for the route's own"
    );
    assert!(
        report.adjusted.is_empty(),
        "nothing moved, so nothing is reported"
    );
    assert_eq!(
        report.path.last().copied(),
        Some(point),
        "the route ends where it was asked to"
    );

    // The order the file lists its wires in is not a stable input — KiCad
    // reorders items when it saves — so the same drawing written in another
    // order must give the same offset.
    let reordered = drawing("four-way-reordered", Arms::ThreeReordered).write();
    let hierarchy = loaded(&reordered);
    let source = pin_terminal(&hierarchy, "R1", "1");
    let (shuffled, _) = route(&hierarchy, &source, &Terminal::of_point(point, "NET"));
    assert_eq!(
        shuffled.adjusted[0].by,
        Point::new(GRID.0, 0),
        "the offset does not depend on which wire the file lists first"
    );
    assert_eq!(
        shuffled.path.last().copied(),
        Some(landed),
        "nor does where the route lands"
    );

    // The shape the offset avoided is real, and the refusal that names it is
    // still live. With the route's own end drawn into the meeting point — the
    // wire the router declined to draw — `add_junction` there refuses as a
    // four-way junction.
    let mut avoided = drawing("four-way-avoided", Arms::Three);
    avoided.wire(("115.57", "101.6"), ("115.57", "88.9"));
    avoided.wire(("115.57", "88.9"), ("114.3", "88.9"));
    let avoided = avoided.write();
    let refusal = junction_at(&avoided, point).expect_err("four wire ends refuse a junction");
    assert!(
        matches!(refusal, MarkError::FourWayJunction { at, .. } if at == point),
        "the refusal is the four-way one, at the meeting point: {refusal}"
    );

    // The control on that refusal. On the drawing the router really produced,
    // the meeting point still carries three ends and a junction there is
    // allowed — so the refusal above is about the fourth end and not about the
    // point. Without this, a `mark` that refused every junction would pass.
    junction_at(&path, point).expect("three wire ends still take a junction");
}

#[test]
fn the_offset_terminus_still_joins_the_net_kicad_reads() {
    // The oracle. The offset moves the wire's end off the point that was asked
    // for, so the question that matters is not whether the arithmetic is right
    // but whether KiCad still reads the connection kicli claimed.
    let Some(kicad) = Kicad::found_or_skip("ask KiCad about the offset terminus") else {
        return;
    };
    let joined = written_and_measured("four-way-oracle", true, &kicad);
    let mut expected: Partition = Partition::new();
    expected.insert(net(&["R1.1", "R2.1"]));
    assert!(
        joined.1.contains(&net(&["R1.1", "R2.1"])),
        "KiCad joins the routed pin to the net the terminus was moved along: {:?}",
        joined.1
    );
    assert_eq!(
        differences(&joined.0, &joined.1),
        None,
        "kicli and KiCad partition the drawing the same way"
    );

    // The control. Without the junction the route needs at its landing point, a
    // wire end on another wire's interior is not a connection — so the reading
    // above is caused by what kicli wrote rather than by the two pins having
    // been joined all along.
    let apart = written_and_measured("four-way-oracle-control", false, &kicad);
    assert!(
        !apart.1.contains(&net(&["R1.1", "R2.1"])),
        "without the junction KiCad reads no connection: {:?}",
        apart.1
    );
    assert_eq!(
        differences(&apart.0, &apart.1),
        None,
        "kicli and KiCad agree about the unjoined drawing too"
    );
    assert_ne!(
        expected.pop_first(),
        None,
        "the expectation this arm compares against is not empty"
    );
}

/// Route the drawing, write the wire kicli chose, and read both partitions.
///
/// `join` says whether the junction the landing point needs is written. It is
/// the one difference between the two arms of the oracle, so the control
/// differs from the measurement in exactly one object.
fn written_and_measured(name: &str, join: bool, kicad: &Kicad) -> (Partition, Partition) {
    let point = meeting_point();
    let path = drawing(name, Arms::Three).write();
    let mut hierarchy = loaded(&path);
    let source = pin_terminal(&hierarchy, "R1", "1");
    let (report, approach) = route(&hierarchy, &source, &Terminal::of_point(point, "NET"));
    let landed = approach.target.at;
    assert!(
        !report.adjusted.is_empty(),
        "the drawing this arm measures is the adjusted one"
    );

    let sheet = hierarchy.placements[0].path.clone();
    let project = path.parent().expect("the drawing sits in a directory");
    let via: Vec<Point> = report.path[1..report.path.len() - 1].to_vec();
    draw(
        &mut hierarchy,
        &Polyline {
            from: End::Pin(PinAddress::new(Refdes("R1".to_owned()), "1")),
            to: End::At(landed),
            via,
        },
        &Config::default().routing,
        &target(&path, project, &sheet),
        "after",
    )
    .expect("the route kicli chose is drawable");
    if join {
        add_junction(
            &mut hierarchy,
            landed,
            &Uuid("00000000-0000-4000-8009-000000000002".to_owned()),
            &target(&path, project, &sheet),
            "after",
        )
        .expect("the landing point takes the junction the route needs");
    }

    let read = loaded(&path);
    (
        kicli_partition(&extract(&read)),
        kicad.netlist_beside(&path).partition(),
    )
}
