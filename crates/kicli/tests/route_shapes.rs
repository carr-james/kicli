//! The routes a person draws, measured on drawings rather than on lists.
//!
//! Every check here builds a probe drawing, sorts it onto the lattice the way a
//! route request does, and asks for the candidate shapes. A hand-built list of
//! rectangles would encode the same assumption as the code that reads one, so
//! it would agree with a body box in the wrong place; a drawing cannot.
//!
//! Each drawing leaves one shape cheapest, and it does so by **geometry** — a
//! shape that turns twice where another turns once loses, and a shape whose
//! corner is covered by a marker is refused. The six drawings together reach
//! every shape of the enumeration, which is what stops the check passing on one
//! silhouette six times.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use kicli::geometry::{GRID, Iu, Point, resolve_pins};
use kicli::model::{Config, Hierarchy, definition_of, read_library};
use kicli::route::{
    Candidate, Cost, Heading, Obstacles, Report, Routed, Shape, Shapes, SheetObjects, Status,
    Tally, Terminal, Uncostable, Window,
};
use kicli_probe::{Kicad, Probe, pin, rectangle, symbol};

/// Where this binary writes the drawings it builds.
fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("route-shapes")
}

/// The identifier `Probe::child_of` gives the child sheet it draws.
const CHILD: &str = "00000000-0000-4000-8000-cccccccccccc";

/// The pin numbers the probe symbol draws.
const PINS: [&str; 4] = ["1", "2", "3", "4"];

/// A point from the millimetres the drawing is written in.
///
/// The drawing and the expectation then read the same number, so a check cannot
/// ask about a point the drawing never meant.
fn at(x: &str, y: &str) -> Point {
    let read = |text: &str| {
        Iu::from_millimetres_text(text)
            .expect("a millimetre reading")
            .0
    };
    Point::new(read(x), read(y))
}

/// A symbol with a square body and one pin on each of its four edges.
///
/// Each pin's angle points from its connection point **towards** the body,
/// which is what `Device:R` draws, so pin 1 faces west, 2 south, 3 east and 4
/// north. The body box a router sees is the union of the rectangle and the pin
/// segments, so it reaches the connection points: 3.81 mm, three grid steps,
/// from the anchor on every side.
fn quad() -> String {
    symbol(
        "QUAD",
        "U",
        false,
        &[(
            "1_1",
            vec![
                rectangle(("-2.54", "-2.54"), ("2.54", "2.54")),
                pin("passive", ("-3.81", "0"), "0", "1", "W"),
                pin("passive", ("0", "-3.81"), "90", "2", "S"),
                pin("passive", ("3.81", "0"), "180", "3", "E"),
                pin("passive", ("0", "3.81"), "270", "4", "N"),
            ],
        )],
    )
}

/// Write a probe drawing that defines the symbol above, and load it.
fn drawing(name: &str, draw: impl FnOnce(&mut Probe)) -> Hierarchy {
    let mut probe = Probe::new(name, scratch());
    probe.define(quad());
    draw(&mut probe);
    Hierarchy::load(&probe.write()).expect("the probe loads")
}

/// The root placement of a loaded probe, and the file it draws.
fn root(hierarchy: &Hierarchy) -> (&kicli::model::LoadedFile, &kicli::model::items::SheetPath) {
    let placement = hierarchy
        .placements
        .first()
        .expect("the root sheet is placed");
    (&hierarchy.files[placement.file], &placement.path)
}

/// The terminal one pin of one placed symbol makes.
///
/// The position and the direction come from the drawing, so a check states the
/// route it wants in the drawing's own millimetres.
fn pin_terminal(hierarchy: &Hierarchy, reference: &str, number: &str) -> Terminal {
    let (file, path) = root(hierarchy);
    let schematic = &file.schematic;
    let library = read_library(&file.doc, &schematic.library_symbols, schematic.version);
    for symbol in schematic.symbols() {
        if symbol.reference_on(path).map(|refdes| refdes.0.as_str()) != Some(reference) {
            continue;
        }
        let definition = definition_of(&library, symbol).expect("the definition is embedded");
        for resolved in resolve_pins(&symbol.drawn_on(path), definition) {
            if resolved.number == number {
                return Terminal::of_pin(reference, &resolved);
            }
        }
    }
    panic!("the drawing has no pin {number} of {reference}");
}

/// The terminal the first port of the first child sheet makes.
fn port_terminal(hierarchy: &Hierarchy) -> Terminal {
    let (file, _) = root(hierarchy);
    let sheet = file
        .schematic
        .sheets()
        .next()
        .expect("the probe draws a sheet");
    Terminal::of_sheet_pin(sheet.pins.first().expect("the sheet carries a port"))
}

/// A drawing, its two terminals, and the map a route request builds from it.
struct Bench {
    source: Terminal,
    target: Terminal,
    obstacles: Obstacles,
}

impl Bench {
    /// The lists and the lattice, for a drawing whose two terminals are known.
    ///
    /// Both terminals are named as the route's own. A pin the route ends on is
    /// a terminal rather than an obstacle, and so is the step it escapes along;
    /// a caller that does not say so finds the target's own halo across the
    /// last step but one, which is what the last check here measures.
    fn new(hierarchy: &Hierarchy, source: Terminal, target: Terminal) -> Self {
        Self::with_terminals(
            hierarchy,
            &[source.name.clone(), target.name.clone()],
            source,
            target,
        )
    }

    /// The same, for a caller that names some other set of terminals.
    fn with_terminals(
        hierarchy: &Hierarchy,
        named: &[String],
        source: Terminal,
        target: Terminal,
    ) -> Self {
        let (file, path) = root(hierarchy);
        let objects = SheetObjects::read(
            file,
            path,
            &Routed {
                wires: &[],
                terminals: named,
            },
        );
        let window = Window::around(
            source.at,
            target.at,
            Config::default().routing.margin,
            objects.page(),
            GRID,
        );
        let obstacles = Obstacles::build(window, &objects.geometry());
        Self {
            source,
            target,
            obstacles,
        }
    }

    /// Every candidate, in the order the shapes are tried.
    fn shapes(&self) -> Shapes {
        Shapes::of(
            &self.source,
            &self.target,
            &self.obstacles,
            &Config::default().routing,
        )
    }

    /// The route the request produces.
    fn route(&self) -> Candidate {
        let shapes = self.shapes();
        shapes
            .best()
            .unwrap_or_else(|| {
                panic!(
                    "no shape reaches {}: blocked by {:?}",
                    self.target.name,
                    shapes.blocked_by()
                )
            })
            .clone()
    }
}

/// One drawing whose obstacles leave one shape cheapest, and the route it owes.
struct Case {
    bench: Bench,
    shape: Shape,
    path: Vec<Point>,
}

/// The six drawings, one per shape.
///
/// Two checks of one binary run at once, so each takes its own copy of the
/// drawings under its own prefix; two tests writing one file would read what
/// the other was still writing.
fn cases(prefix: &str) -> Vec<Case> {
    let mut cases = Vec::new();

    // I — two pins facing each other across an empty row. Every later shape
    // draws the same straight line, so the one that is reported is decided by
    // the order the shapes are tried in.
    let straight = drawing(&format!("{prefix}-straight"), |probe| {
        probe.place("QUAD", "U1", ("76.2", "101.6"), &PINS);
        probe.place("QUAD", "U2", ("127", "101.6"), &PINS);
    });
    cases.push(Case {
        bench: Bench::new(
            &straight,
            pin_terminal(&straight, "U1", "3"),
            pin_terminal(&straight, "U2", "1"),
        ),
        shape: Shape::Straight,
        path: vec![at("80.01", "101.6"), at("123.19", "101.6")],
    });

    // L, horizontal first — a pin facing east and one facing north. The
    // horizontal leg continues the source's own escape and the vertical leg
    // runs into the target's, so the route turns once; taking the legs the
    // other way round turns three times over the same distance.
    let across = drawing(&format!("{prefix}-l-across"), |probe| {
        probe.place("QUAD", "U1", ("76.2", "101.6"), &PINS);
        probe.place("QUAD", "U2", ("127", "127"), &PINS);
    });
    cases.push(Case {
        bench: Bench::new(
            &across,
            pin_terminal(&across, "U1", "3"),
            pin_terminal(&across, "U2", "4"),
        ),
        shape: Shape::LHorizontalFirst,
        path: vec![
            at("80.01", "101.6"),
            at("127", "101.6"),
            at("127", "123.19"),
        ],
    });

    // L, vertical first — the same pair of escapes turned a quarter turn: a pin
    // facing south and one facing west.
    let down = drawing(&format!("{prefix}-l-down"), |probe| {
        probe.place("QUAD", "U1", ("76.2", "76.2"), &PINS);
        probe.place("QUAD", "U2", ("127", "127"), &PINS);
    });
    cases.push(Case {
        bench: Bench::new(
            &down,
            pin_terminal(&down, "U1", "2"),
            pin_terminal(&down, "U2", "1"),
        ),
        shape: Shape::LVerticalFirst,
        path: vec![at("76.2", "80.01"), at("76.2", "127"), at("123.19", "127")],
    });

    // Z, vertical middle — two pins facing each other, four rows apart, with a
    // no-connect on each of the two columns an L would turn on. Every remaining
    // three-segment route turns twice over the same distance, so the tie-break
    // decides, and the lexicographically smallest vertex list is the one whose
    // middle column is furthest left.
    let column = drawing(&format!("{prefix}-z-column"), |probe| {
        probe.place("QUAD", "U1", ("76.2", "101.6"), &PINS);
        probe.place("QUAD", "U2", ("91.44", "106.68"), &PINS);
        probe.no_connect(("81.28", "104.14"));
        probe.no_connect(("86.36", "104.14"));
    });
    cases.push(Case {
        bench: Bench::new(
            &column,
            pin_terminal(&column, "U1", "3"),
            pin_terminal(&column, "U2", "1"),
        ),
        shape: Shape::ZVerticalMiddle,
        path: vec![
            at("80.01", "101.6"),
            at("82.55", "101.6"),
            at("82.55", "106.68"),
            at("87.63", "106.68"),
        ],
    });

    // Z, horizontal middle — the same drawing turned a quarter turn.
    let row = drawing(&format!("{prefix}-z-row"), |probe| {
        probe.place("QUAD", "U1", ("101.6", "76.2"), &PINS);
        probe.place("QUAD", "U2", ("106.68", "91.44"), &PINS);
        probe.no_connect(("104.14", "81.28"));
        probe.no_connect(("104.14", "86.36"));
    });
    cases.push(Case {
        bench: Bench::new(
            &row,
            pin_terminal(&row, "U1", "2"),
            pin_terminal(&row, "U2", "4"),
        ),
        shape: Shape::ZHorizontalMiddle,
        path: vec![
            at("101.6", "80.01"),
            at("101.6", "82.55"),
            at("106.68", "82.55"),
            at("106.68", "87.63"),
        ],
    });

    // U — two pins facing the **same** way, so the route must reach past the
    // target and come back into it along its own direction. Every three-segment
    // route between the two escape points either turns back on itself at the
    // target or turns on the one column a no-connect stands in, and the shapes
    // that stay inside the span turn four times where the U turns twice.
    let around = drawing(&format!("{prefix}-u-around"), |probe| {
        probe.place("QUAD", "U1", ("76.2", "101.6"), &PINS);
        probe.place("QUAD", "U2", ("91.44", "111.76"), &PINS);
        probe.no_connect(("96.52", "106.68"));
    });
    cases.push(Case {
        bench: Bench::new(
            &around,
            pin_terminal(&around, "U1", "3"),
            pin_terminal(&around, "U2", "3"),
        ),
        shape: Shape::UOutside,
        path: vec![
            at("80.01", "101.6"),
            at("97.79", "101.6"),
            at("97.79", "111.76"),
            at("95.25", "111.76"),
        ],
    });

    cases
}

#[test]
fn each_shape_is_drawn_when_it_is_the_cheapest() {
    let weights = Config::default().routing;
    let mut drawn: BTreeSet<Shape> = BTreeSet::new();
    for case in cases("cheapest") {
        let route = case.bench.route();
        assert_eq!(
            route.shape, case.shape,
            "the route is {:?} and not {:?}: {:?}",
            route.shape, case.shape, route.path
        );
        assert_eq!(route.path, case.path, "{:?}", case.shape);

        // What the route says it met is what walking it meets, so a candidate
        // carrying another candidate's tally would disagree here.
        let walked = Tally::of_path(&route.path, &case.bench.obstacles)
            .expect("the route that was chosen can be walked");
        assert_eq!(walked, route.tally, "{:?}", case.shape);
        assert_eq!(route.cost, Cost::of(walked, &weights), "{:?}", case.shape);
        drawn.insert(route.shape);
    }
    assert_eq!(
        drawn,
        Shape::EVERY.into_iter().collect::<BTreeSet<Shape>>(),
        "every shape of the enumeration is drawn by some drawing"
    );
}

#[test]
fn every_candidate_is_a_polyline_a_wire_can_be_drawn_from() {
    // A candidate that stepped back the way it came would draw a wire over its
    // own last segment, and one that kept a vertex mid-run would write two wire
    // records where a reader sees one. Neither is a route, so neither may be
    // offered — and the enumeration produces both, because a shape whose first
    // leg runs against the source's escape is one of the shapes.
    let mut offered = 0;
    let mut refused = 0;
    for case in cases("polylines") {
        let shapes = case.bench.shapes();
        let considered = usize::try_from(shapes.considered()).expect("a candidate count");
        refused += considered - shapes.feasible().len();
        for candidate in shapes.feasible() {
            offered += 1;
            let path = &candidate.path;
            assert!(path.len() >= 2, "{path:?}");
            let mut last: Option<Heading> = None;
            for step in path.windows(2) {
                let heading = Heading::between(step[0], step[1])
                    .unwrap_or_else(|| panic!("{step:?} is along neither axis"));
                if let Some(before) = last {
                    assert_ne!(before, heading, "{path:?} keeps a vertex mid-run");
                    assert_ne!(before, heading.reversed(), "{path:?} turns back on itself");
                }
                last = Some(heading);
            }

            // The escape rule, at both ends: the route leaves along the
            // source's own direction and arrives along the target's.
            let (source, target) = (&case.bench.source, &case.bench.target);
            assert_eq!(path.first(), Some(&source.at));
            assert_eq!(path.last(), Some(&target.at));
            assert_eq!(Heading::between(path[0], path[1]), source.escape);
            let arrival = Heading::between(path[path.len() - 2], path[path.len() - 1]);
            assert_eq!(arrival, target.escape.map(Heading::reversed));
        }
    }
    // The controls: candidates were offered, and candidates were refused. A
    // check that walked an empty list would pass every assertion above.
    assert!(offered > 10, "{offered} candidates were offered");
    assert!(refused > 10, "{refused} candidates were refused");
}

#[test]
fn equal_cost_shapes_break_the_documented_way() {
    // A pin facing west and one facing east, whose escape points are one grid
    // step apart on each axis. The span is then one column and one row wide, so
    // the Z families can only redraw the two Ls, and the two Ls turn twice over
    // the same distance through the same crowding. The check measures nothing
    // if they do not tie, so the tie is asserted before anything is concluded
    // from it.
    //
    // The first drawing this check was written on had the terminals four steps
    // apart, and a Z whose middle column cleared a body's crowded ring beat
    // both Ls at 26 against 34 — measured, 2026-08-15. That is the router
    // working; it is not a tie, so the drawing was narrowed rather than the
    // expectation adjusted.
    let hierarchy = drawing("equal-cost", |probe| {
        probe.place("QUAD", "U1", ("101.6", "101.6"), &PINS);
        probe.place("QUAD", "U2", ("90.17", "102.87"), &PINS);
    });
    let bench = Bench::new(
        &hierarchy,
        pin_terminal(&hierarchy, "U1", "1"),
        pin_terminal(&hierarchy, "U2", "3"),
    );
    let shapes = bench.shapes();
    let first_of = |wanted: Shape| -> &Candidate {
        shapes
            .feasible()
            .iter()
            .find(|candidate| candidate.shape == wanted)
            .unwrap_or_else(|| panic!("{wanted:?} is not feasible"))
    };

    let across = first_of(Shape::LHorizontalFirst);
    let down = first_of(Shape::LVerticalFirst);
    assert_eq!(across.cost, down.cost, "the two Ls cost the same");
    assert_eq!(across.tally.corners, down.tally.corners);
    assert_eq!(across.tally.steps, down.tally.steps);
    assert_ne!(across.path, down.path, "and they are two different routes");

    let best = shapes.best().expect("a route exists");
    assert_eq!(best.shape, Shape::LHorizontalFirst, "{:?}", best.path);
    assert_eq!(
        best.path,
        vec![
            at("97.79", "101.6"),
            at("95.25", "101.6"),
            at("95.25", "102.87"),
            at("93.98", "102.87"),
        ]
    );

    // A later family offers the same route by another name: a Z whose free
    // coordinate lands on the target's own column draws the L exactly. Nothing
    // in the cost and nothing in the vertices can tell those two apart, so the
    // order the shapes are tried in is the only thing that can — which is what
    // this pair of assertions measures.
    let twins: Vec<Shape> = shapes
        .feasible()
        .iter()
        .filter(|candidate| candidate.path == best.path)
        .map(|candidate| candidate.shape)
        .collect();
    assert!(
        twins.len() > 1,
        "no later shape draws the same route: {twins:?}"
    );
    assert!(
        twins.iter().all(|shape| *shape >= best.shape),
        "an earlier shape drew this route and lost it: {twins:?}"
    );

    // And the order itself, which is the whole of §7's first guarantee. The
    // span is one column by one row, so each Z family holds exactly two
    // candidates and its free coordinate ascending puts the one nearest the
    // smaller coordinate first. Nothing here is a tie-break: it is the sequence
    // the shapes were built in, which is what a tie falls back on.
    let enumerated: Vec<(Shape, &[Point])> = shapes
        .feasible()
        .iter()
        .map(|candidate| (candidate.shape, candidate.path.as_slice()))
        .collect();
    let (families, detours) = enumerated.split_at(6);
    assert_eq!(
        families,
        [
            (Shape::LHorizontalFirst, across.path.as_slice()),
            (Shape::LVerticalFirst, down.path.as_slice()),
            (Shape::ZVerticalMiddle, across.path.as_slice()),
            (Shape::ZVerticalMiddle, down.path.as_slice()),
            (Shape::ZHorizontalMiddle, across.path.as_slice()),
            (Shape::ZHorizontalMiddle, down.path.as_slice()),
        ],
        "the shapes are tried in the order §7 fixes, free coordinate ascending"
    );

    // The U family is last, and its offset ascends: each step further out is a
    // longer route, so the lengths never fall as the enumeration runs on.
    assert!(
        detours.iter().all(|&(shape, _)| shape == Shape::UOutside),
        "{detours:?}"
    );
    let reaches: Vec<u32> = shapes.feasible()[6..]
        .iter()
        .map(|candidate| candidate.tally.steps)
        .collect();
    assert!(
        reaches.windows(2).all(|pair| pair[0] <= pair[1]),
        "the U offsets do not ascend: {reaches:?}"
    );
    assert!(
        reaches.last() > reaches.first(),
        "every U reached the same distance: {reaches:?}"
    );

    // Within one offset the four sides run in the order §4.3 fixes for the
    // search's own expansion — `+x, −x, +y, −y`, one order in the router rather
    // than two. Both x sides turn back against the source's escape here and are
    // refused, so what this drawing can see of that order is the +y side being
    // offered before the −y side.
    let outward = |path: &[Point]| path.iter().map(|point| point.y).max();
    assert!(
        outward(detours[0].1) > outward(detours[1].1),
        "the sides of one offset are not in order: {:?}",
        &detours[..2]
    );

    // Every candidate the enumeration produced is counted, feasible or not: no
    // straight run, because the escape points share neither coordinate; two Ls;
    // one Z per grid line between the escape points on each axis; and four U
    // offsets per grid step out to the configured limit.
    let weights = Config::default().routing;
    let from = bench.source.escape_point(GRID);
    let to = bench.target.escape_point(GRID);
    assert_ne!(from.x, to.x, "no straight run is possible");
    assert_ne!(from.y, to.y);
    let lines = |one: Iu, other: Iu| {
        u32::try_from((one.0 - other.0).abs() / GRID.0).expect("a span in grid steps") + 1
    };
    let offsets = u32::try_from(weights.u_max.0 / GRID.0).expect("a limit in grid steps");
    let expected = 2 + lines(from.x, to.x) + lines(from.y, to.y) + 4 * offsets;
    assert_eq!(shapes.considered(), expected);
    assert_eq!(expected, 30, "two columns by two rows, and six offsets");

    // Which is the number a report carries, beside the route it chose.
    let mut report = Report::of(Status::Routed, &bench.source.name, &bench.target.name);
    report.path.clone_from(&best.path);
    report.tally = best.tally;
    report.cost = best.cost;
    report.alternatives_considered = shapes.considered();
    assert_eq!(report.segments(), 3);
    assert_eq!(report.corners(), 2);
    assert_eq!(report.alternatives_considered, expected);
}

#[test]
fn a_route_arrives_at_a_sheet_pin_the_sheet_body_covers() {
    // The case the target-cell exception exists for. A sheet pin sits on the
    // sheet's border, and the border is inside the body box, which is a hard
    // block — so the route is refused at the very point it was asked to reach
    // unless arriving there is excepted. The exception has one home, in the
    // walk that costs a path, and this check reaches it through the shapes.
    let mut probe = Probe::new("sheet-pin", scratch());
    let child = Probe::child_of(&probe);
    probe.define(quad());
    probe.place("QUAD", "U1", ("101.6", "101.6"), &PINS);
    // The port sits on the sheet's left edge, which is the edge its angle
    // names, so a wire leaves it westwards into the empty page.
    probe.sheet_named(CHILD, "child", "IN", ("127", "101.6"), "180");
    let hierarchy = Hierarchy::load(&probe.write_all(&[&child])).expect("the probe loads");

    let target = port_terminal(&hierarchy);
    assert_eq!(target.at, at("127", "101.6"));
    assert_eq!(target.escape, Some(Heading::MinusX));
    let bench = Bench::new(&hierarchy, pin_terminal(&hierarchy, "U1", "3"), target);

    // The control: the terminus really is covered, so the route below is not
    // arriving at an empty cell.
    let port = at("127", "101.6");
    assert_eq!(
        bench
            .obstacles
            .entering(port, Heading::PlusX)
            .blocked_by
            .as_deref(),
        Some("child"),
        "the sheet body covers its own border"
    );

    let route = bench.route();
    assert_eq!(route.shape, Shape::Straight);
    assert_eq!(route.path, vec![at("105.41", "101.6"), port]);

    // And the second control: passing through that cell rather than stopping on
    // it is still refused, naming the sheet. The exception is for arriving.
    let through = [at("105.41", "101.6"), at("128.27", "101.6")];
    let refused = Tally::of_path(&through, &bench.obstacles).expect_err("the body is in the way");
    let Uncostable::Blocked { handle, at: point } = refused else {
        panic!("{refused}");
    };
    assert_eq!(handle, "child");
    assert_eq!(point, port, "the point it was allowed to arrive at");
}

#[test]
fn kicad_reads_the_route_to_a_sheet_pin_as_the_connection_it_claims() {
    // The debt the terminal rule (T6) recorded: a sheet pin's angle names the
    // **edge it sits on**, and that was established from KiCad's own parser
    // rather than measured against the running tool, so the first task that
    // routes to a sheet pin owes it a probe. This is that probe, and the route
    // it draws is the one the shapes chose rather than one written by hand.
    let Some(kicad) = Kicad::found_or_skip("measure a sheet pin's edge against KiCad") else {
        return;
    };

    // The drawing, with the child's own end of the port made visible to a
    // netlist: a resistor pin on a wire the hierarchical label `IN` names. A
    // sheet pin is not a component pin, so nothing else would show the join.
    let build = |name: &str, wires: &[(Point, Point)]| -> PathBuf {
        let mut probe = Probe::new(name, scratch());
        let mut child = Probe::child_of(&probe);
        // The shape goes in as written, and KiCad reads `(shape input)` rather
        // than a bare `input` — `tests/fixtures/sch/nets/nets_channel.kicad_sch`
        // line 384, which KiCad wrote. A bare token leaves the label in the
        // file and out of KiCad's netlist, which is how this probe first
        // measured its own defect rather than KiCad's answer.
        child.strand_of_kind(
            "hierarchical_label",
            "(shape input)",
            "R1",
            "25.4",
            "29.21",
            "IN",
        );
        probe.define(quad());
        probe.place("QUAD", "U1", ("101.6", "101.6"), &PINS);
        probe.sheet_named(CHILD, "child", "IN", ("127", "101.6"), "180");
        for &(from, to) in wires {
            probe.wire(
                (&from.x.to_string(), &from.y.to_string()),
                (&to.x.to_string(), &to.y.to_string()),
            );
        }
        probe.write_all(&[&child])
    };

    // The route the shapes choose, over the drawing with nothing drawn on it.
    let planned = Hierarchy::load(&build("sheet-pin-oracle-plan", &[])).expect("the probe loads");
    let bench = Bench::new(
        &planned,
        pin_terminal(&planned, "U1", "3"),
        port_terminal(&planned),
    );
    let route = bench.route().path;
    let legs: Vec<(Point, Point)> = route.windows(2).map(|step| (step[0], step[1])).collect();

    let joined = kicad.netlist_beside(&build("sheet-pin-oracle", &legs));
    let together = |netlist: &kicli_probe::Netlist| {
        netlist
            .partition()
            .iter()
            .any(|net| net.iter().any(|pin| pin == "U1.3") && net.iter().any(|pin| pin == "R1.1"))
    };
    assert!(
        together(&joined),
        "KiCad did not join the pin to the port: {:?}",
        joined.partition()
    );

    // The control that must fire: the same route one grid step short of the
    // port leaves the two on separate nets. Without it the assertion above
    // would pass on a netlist that joined everything.
    let short: Vec<(Point, Point)> = legs
        .iter()
        .map(|&(from, to)| {
            if to == bench.target.at {
                (from, Point::new(to.x.0 - GRID.0, to.y.0))
            } else {
                (from, to)
            }
        })
        .collect();
    let apart = kicad.netlist_beside(&build("sheet-pin-oracle-short", &short));
    assert!(
        !together(&apart),
        "a wire that stops short still joined them, so the probe measures nothing: {:?}",
        apart.partition()
    );
}

#[test]
fn a_terminal_the_caller_did_not_name_blocks_its_own_route() {
    // The obligation the cost model's exception carries with it: a pin the
    // route ends on goes in the caller's list of terminals, or the pin's own
    // halo — the step it escapes along — stands across the last step but one.
    // The exception covers the terminus and nothing before it.
    let hierarchy = drawing("unnamed-terminal", |probe| {
        probe.place("QUAD", "U1", ("76.2", "101.6"), &PINS);
        probe.place("QUAD", "U2", ("127", "127"), &PINS);
    });
    let source = pin_terminal(&hierarchy, "U1", "3");
    let target = pin_terminal(&hierarchy, "U2", "4");

    let named = Bench::new(&hierarchy, source.clone(), target.clone());
    let route = named.route();
    assert_eq!(route.shape, Shape::LHorizontalFirst);
    assert_eq!(route.path.last(), Some(&at("127", "123.19")));

    // The same drawing, with the target left out of the caller's list: the halo
    // one step short of the pin refuses every candidate, by name.
    let named_source = std::slice::from_ref(&source.name).to_vec();
    let silent = Bench::with_terminals(&hierarchy, &named_source, source, target);
    let shapes = silent.shapes();
    assert!(shapes.best().is_none(), "{:?}", shapes.feasible());
    assert!(
        shapes.blocked_by().iter().any(|handle| handle == "U2.4"),
        "the refusal names the pin whose halo blocked it: {:?}",
        shapes.blocked_by()
    );
}
