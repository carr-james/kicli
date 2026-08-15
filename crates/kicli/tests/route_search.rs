//! A\* over turn-aware states, measured on drawings rather than on lists.
//!
//! Every check here builds a probe drawing, sorts it onto the lattice the way a
//! route request does, and asks the search for a route. A hand-built list of
//! rectangles would encode the same assumption as the code that reads one; a
//! drawing cannot.
//!
//! The exception is [`the_queue_order_is_total`], which is about a comparator.
//! No drawing can produce a pair of queue entries, so the pairs are generated —
//! the condition the routing window's own tests already record for hand-stated
//! input.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use kicli::geometry::{GRID, Iu, Point, resolve_pins};
use kicli::model::{Config, Hierarchy, definition_of, read_library};
use kicli::route::{
    Cell, Cost, Heading, Obstacles, Queued, Routed, Search, Shapes, SheetObjects, State, Tally,
    Terminal, Uncostable, Window,
};
use kicli_probe::{Kicad, Probe, pin, rectangle, symbol};

/// Where this binary writes the drawings it builds.
fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("route-search")
}

/// The pin numbers the probe symbol draws.
const PINS: [&str; 4] = ["1", "2", "3", "4"];

/// A length from the millimetres the drawing is written in.
fn mm(text: &str) -> Iu {
    Iu::from_millimetres_text(text).expect("a millimetre reading")
}

/// A point from the millimetres the drawing is written in.
///
/// The drawing and the expectation then read the same number, so a check cannot
/// ask about a point the drawing never meant.
fn at(x: &str, y: &str) -> Point {
    Point::new(mm(x).0, mm(y).0)
}

/// A symbol with a square body and one pin on each of its four edges.
///
/// Each pin's angle points from its connection point **towards** the body,
/// which is what `Device:R` draws, so pin 1 faces west, 2 south, 3 east and 4
/// north. The body box a router sees is the union of the rectangle and the pin
/// segments, so it reaches the connection points: 3.81 mm, three grid steps,
/// from the anchor on every side — which is why a pin is a terminal whose own
/// cell is blocked by its own symbol.
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

/// Write a probe drawing that defines the symbol above, and the file it wrote.
fn written(name: &str, draw: impl FnOnce(&mut Probe)) -> PathBuf {
    let mut probe = Probe::new(name, scratch());
    probe.define(quad());
    draw(&mut probe);
    probe.write()
}

/// The same, loaded.
fn drawing(name: &str, draw: impl FnOnce(&mut Probe)) -> Hierarchy {
    Hierarchy::load(&written(name, draw)).expect("the probe loads")
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

/// A drawing, its two terminals, and the map a route request builds from it.
struct Bench {
    source: Terminal,
    target: Terminal,
    obstacles: Obstacles,
}

impl Bench {
    /// The lists and the lattice, for a drawing whose two terminals are known.
    ///
    /// Both terminals are named as the route's own, so neither pin nor its halo
    /// is an obstacle. The **bodies** they sit in are still obstacles, which is
    /// the case the target-cell exception exists for.
    fn new(hierarchy: &Hierarchy, source: Terminal, target: Terminal) -> Self {
        let (file, path) = root(hierarchy);
        let named = [source.name.clone(), target.name.clone()];
        let objects = SheetObjects::read(
            file,
            path,
            &Routed {
                wires: &[],
                terminals: &named,
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

    /// What the search answers.
    fn search(&self) -> Search {
        Search::of(
            &self.source,
            &self.target,
            &self.obstacles,
            &Config::default().routing,
        )
    }

    /// What the shapes answer, for the drawings that defeat them.
    fn shapes(&self) -> Shapes {
        Shapes::of(
            &self.source,
            &self.target,
            &self.obstacles,
            &Config::default().routing,
        )
    }
}

/// Every grid step of a path, with the heading it is taken in.
///
/// Walking the vertices is also what proves the path is orthogonal and on the
/// lattice: a leg along neither axis names no heading, and one that is not a
/// whole number of grid steps is caught here rather than asserted separately.
fn steps_of(path: &[Point]) -> Vec<(Point, Heading)> {
    let mut steps = Vec::new();
    for pair in path.windows(2) {
        let heading = Heading::between(pair[0], pair[1])
            .unwrap_or_else(|| panic!("{pair:?} is along neither axis"));
        let span = (pair[1].x.0 - pair[0].x.0).abs() + (pair[1].y.0 - pair[0].y.0).abs();
        assert_eq!(span % GRID.0, 0, "{pair:?} is not a whole number of steps");
        for step in 1..=span / GRID.0 {
            steps.push((heading.step(pair[0], Iu(step * GRID.0)), heading));
        }
    }
    steps
}

/// Assert that nothing but the arrival is a step the map refuses.
///
/// The route's own terminus is excepted, and every point before it is not.
fn free_of_hard_blocks(path: &[Point], obstacles: &Obstacles) {
    let steps = steps_of(path);
    let last = steps.len() - 1;
    for (index, &(at, heading)) in steps.iter().enumerate() {
        let blocked = obstacles.entering(at, heading).blocked_by;
        if index < last {
            assert_eq!(blocked, None, "the route is blocked at {at}");
        }
    }
}

#[test]
fn the_search_and_the_walk_except_the_same_ends() {
    // Three symbols in a row. Every pin of the probe symbol sits on its own
    // body box — the box is the union of the rectangle and the pin segments —
    // so a terminal's own cell is a hard block, and a route can only arrive
    // there if the block is excepted. `Tally::of_path` excepts the two ends of
    // the path it walks; A* expands against `Obstacles::entering` and cannot
    // inherit that, so it excepts its own goal cell to the same rule. Two homes
    // for one rule drift silently, and this check is what fails when they do.
    let hierarchy = drawing("twin-ends", |probe| {
        probe.place("QUAD", "U1", ("76.2", "101.6"), &PINS);
        probe.place("QUAD", "U2", ("95.25", "101.6"), &PINS);
        probe.place("QUAD", "U3", ("114.3", "101.6"), &PINS);
    });

    let source = pin_terminal(&hierarchy, "U1", "3");
    let target = pin_terminal(&hierarchy, "U2", "1");
    assert_eq!(source.at, at("80.01", "101.6"));
    assert_eq!(source.escape, Some(Heading::PlusX));
    assert_eq!(target.at, at("91.44", "101.6"));
    assert_eq!(target.escape, Some(Heading::MinusX));
    let bench = Bench::new(&hierarchy, source, target);

    // The controls, before anything is concluded: both ends really are covered
    // by their own symbol's body, so the route below is not arriving at an
    // empty cell and is not leaving one either.
    assert_eq!(
        bench
            .obstacles
            .entering(at("80.01", "101.6"), Heading::PlusX)
            .blocked_by
            .as_deref(),
        Some("U1"),
        "the source's own body covers its own pin"
    );
    assert_eq!(
        bench
            .obstacles
            .entering(at("91.44", "101.6"), Heading::PlusX)
            .blocked_by
            .as_deref(),
        Some("U2"),
        "and so does the target's"
    );

    // A* reaches it. Nine grid steps along one row, no corner, and two steps
    // through a crowded ring: the escape cell of each terminal is one step
    // outside its own body. Every number here is read off the drawing.
    let search = bench.search();
    let route = search
        .route()
        .expect("A* reaches a pin its own symbol's body covers");
    assert_eq!(
        route.path,
        vec![at("80.01", "101.6"), at("91.44", "101.6")],
        "{:?}",
        search.blocked_by()
    );
    assert_eq!(
        route.tally,
        Tally {
            steps: 9,
            corners: 0,
            crossings: 0,
            text_steps: 0,
            near_steps: 2,
        }
    );
    assert_eq!(route.cost.total(), 9 + 2 * 2);

    // And the walk costs the very path the search returned, to the same tally
    // and the same cost. This is the twin: the search accepted an arrival the
    // walk must accept too.
    let walked = Tally::of_path(&route.path, &bench.obstacles)
        .expect("the walk costs the path the search returned");
    assert_eq!(walked, route.tally, "the search and the walk disagree");
    assert_eq!(Cost::of(walked, &Config::default().routing), route.cost);

    // The other half of the same rule: passing **through** a body is not
    // excepted, in either home. The route from U1 to U3 has U2 in the way.
    let through = Bench::new(
        &hierarchy,
        pin_terminal(&hierarchy, "U1", "3"),
        pin_terminal(&hierarchy, "U3", "1"),
    );
    let straight = [at("80.01", "101.6"), at("110.49", "101.6")];
    let refused =
        Tally::of_path(&straight, &through.obstacles).expect_err("U2 stands on the straight line");
    let Uncostable::Blocked { handle, at: point } = refused else {
        panic!("{refused}");
    };
    // The halo of U2's own west pin stands one step before the body does, and
    // U2.1 is not this route's terminal, so it is an obstacle like any other.
    assert_eq!(handle, "U2.1");
    assert_eq!(point, at("90.17", "101.6"));

    // The search goes round instead. Column 95.25 — the column U2's north and
    // south pins stand on — is closed from row 96.52 to row 106.68 by those
    // pins' halos, so the route must cross it five rows off centre: 24 steps
    // across, five up and five back, four corners, and the same two crowded
    // escape cells as before.
    let around = through.search();
    let route = around.route().expect("a route round the symbol in the way");
    assert_eq!(route.tally.steps, 34);
    assert_eq!(route.tally.corners, 4);
    assert_eq!(route.tally.near_steps, 2);
    assert_eq!(route.cost.total(), 34 + 4 * 6 + 2 * 2);
    free_of_hard_blocks(&route.path, &through.obstacles);

    // And it is cheaper than the one shape that fits. The U reaches outward
    // from the escape points themselves, so both its legs run down the crowded
    // ring of a symbol the search steps clear of: same length, same corners,
    // eight more crowded steps. A search that returned more than the cheapest
    // route it could see would show up here.
    let shape = through
        .shapes()
        .best()
        .expect("the U at five steps out clears the pins' halos")
        .clone();
    assert_eq!(shape.tally.steps, route.tally.steps);
    assert_eq!(shape.tally.corners, route.tally.corners);
    assert_eq!(shape.tally.near_steps, 10);
    assert!(
        route.cost.total() < shape.cost.total(),
        "the search did no better than the shape: {} against {}",
        route.cost.total(),
        shape.cost.total()
    );

    assert_eq!(
        Tally::of_path(&route.path, &through.obstacles)
            .expect("the walk costs the path the search returned"),
        route.tally,
        "the search and the walk disagree on the route round"
    );
}

/// The row every drawing below routes along.
const ROW: &str = "101.6";

/// The two walls of the maze, and the row each one leaves open.
///
/// A wall closes one column over the whole height of the routing window, so a
/// route cannot go round it: the window is the two terminals' bounding box
/// inflated by the configured margin, and the wall spans exactly that.
fn wall(probe: &mut Probe, column: &str, gap: Iu) {
    let margin = Config::default().routing.margin.0 / GRID.0;
    for step in -margin..=margin {
        let row = Iu(mm(ROW).0 + step * GRID.0);
        if row != gap {
            probe.no_connect((column, &row.to_string()));
        }
    }
}

/// The drawing whose obstacles defeat every shape.
///
/// Two walls, staggered: the first is open four rows below the terminals, the
/// second only on the terminals' own row. A route must therefore drop to the
/// lower row, cross the first wall, come back up, and cross the second — five
/// runs, four corners. Every silhouette of the enumeration draws at most one
/// jog between the escape points, so every one of them meets a wall.
fn maze(name: &str) -> PathBuf {
    written(name, |probe| {
        probe.place("QUAD", "U1", ("76.2", ROW), &PINS);
        probe.place("QUAD", "U2", ("114.3", ROW), &PINS);
        wall(probe, "88.9", Iu(mm(ROW).0 + 4 * GRID.0));
        wall(probe, "99.06", mm(ROW));
    })
}

#[test]
fn a_star_routes_what_no_shape_can() {
    let hierarchy = Hierarchy::load(&maze("maze")).expect("the probe loads");
    let bench = Bench::new(
        &hierarchy,
        pin_terminal(&hierarchy, "U1", "3"),
        pin_terminal(&hierarchy, "U2", "1"),
    );

    // The control on the drawing itself: the walls span the whole window, so
    // nothing routes round their ends.
    let window = bench.obstacles.window().area();
    assert_eq!(window.start(), at("69.85", "91.44"));
    assert_eq!(window.end(), at("120.65", "111.76"));

    // No shape reaches the target. The enumeration was tried in full — one
    // straight run, two Ls, one Z per grid line between the escape points on
    // each axis, and four U sides per offset out to the configured limit — and
    // every candidate met a wall or drew over itself.
    let shapes = bench.shapes();
    assert!(
        shapes.best().is_none(),
        "a shape reached it after all: {:?}",
        shapes.best().map(|candidate| &candidate.path)
    );
    assert_eq!(shapes.considered(), 1 + 2 + 23 + 1 + 4 * 6);
    assert!(
        !shapes.blocked_by().is_empty(),
        "no candidate was refused, so nothing was tried"
    );

    // A* does reach it: down four rows, across the first wall's gap, back up,
    // and out through the second's. Twenty-four steps across, four down and
    // four back, four corners, and the two crowded escape cells.
    let search = bench.search();
    let route = search
        .route()
        .expect("A* threads the maze the shapes cannot");
    assert_eq!(route.tally.steps, 24 + 4 + 4);
    assert_eq!(route.tally.corners, 4);
    assert_eq!(route.tally.near_steps, 2);
    assert_eq!(route.cost.total(), 32 + 4 * 6 + 2 * 2);

    // Orthogonal, on the lattice, and free of hard blocks — the walk expands
    // the vertices and refuses a leg that is neither.
    free_of_hard_blocks(&route.path, &bench.obstacles);
    assert_eq!(route.path.first(), Some(&bench.source.at));
    assert_eq!(route.path.last(), Some(&bench.target.at));

    // And it goes through the two gaps, which is the only way through.
    let visited: BTreeSet<Point> = steps_of(&route.path)
        .into_iter()
        .map(|(point, _)| point)
        .collect();
    assert!(visited.contains(&at("88.9", "106.68")), "{visited:?}");
    assert!(visited.contains(&at("99.06", "101.6")), "{visited:?}");

    // The walk costs the very path the search returned.
    assert_eq!(
        Tally::of_path(&route.path, &bench.obstacles).expect("the route can be walked"),
        route.tally
    );
}

#[test]
fn a_blocked_route_names_what_blocked_it() {
    // A target whose only way in is closed: a route arrives at a pin along the
    // pin's own direction, so the one cell it can arrive from is the escape
    // point, and a no-connect stands on it.
    let escape = at("90.17", "101.6");
    let file = written("walled-in", |probe| {
        probe.place("QUAD", "U1", ("76.2", ROW), &PINS);
        probe.place("QUAD", "U2", ("95.25", ROW), &PINS);
        probe.no_connect(("90.17", ROW));
    });
    let before = std::fs::read(&file).expect("the drawing reads");
    let hierarchy = Hierarchy::load(&file).expect("the probe loads");
    let bench = Bench::new(
        &hierarchy,
        pin_terminal(&hierarchy, "U1", "3"),
        pin_terminal(&hierarchy, "U2", "1"),
    );

    let search = bench.search();
    assert!(
        search.route().is_none(),
        "{:?}",
        search.route().map(|route| &route.path)
    );
    assert!(!search.blocked_by().is_empty(), "a bare failure");

    // The handles are identifiers of objects the drawing holds: a reference
    // designator, a pin of one, or the identifier of an item in the file. A
    // handle that named nothing would be a list an agent cannot act on.
    let (loaded, path) = root(&hierarchy);
    let mut real: BTreeSet<String> = BTreeSet::new();
    for symbol in loaded.schematic.symbols() {
        let Some(refdes) = symbol.reference_on(path) else {
            continue;
        };
        real.insert(refdes.0.clone());
        for number in PINS {
            real.insert(format!("{}.{number}", refdes.0));
        }
    }
    let mut markers: BTreeSet<String> = BTreeSet::new();
    for item in &loaded.schematic.items {
        if let kicli::model::items::Item::NoConnect(marker) = item {
            markers.insert(marker.uuid.short().to_owned());
        }
    }
    real.extend(markers.iter().cloned());
    for handle in search.blocked_by() {
        assert!(
            real.contains(handle),
            "{handle} names nothing in the drawing"
        );
    }
    assert_eq!(markers.len(), 1, "the drawing draws one no-connect");
    assert!(
        search
            .blocked_by()
            .iter()
            .any(|handle| markers.contains(handle)),
        "the refusal does not name the marker that closed the way: {:?}",
        search.blocked_by()
    );

    // Nothing was written: the search is pure over the lists it was handed.
    assert_eq!(
        std::fs::read(&file).expect("the drawing still reads"),
        before,
        "the search wrote to the drawing"
    );

    // The control that must fire. The same drawing without the marker routes
    // straight in, so what refused the route above is the marker and not a
    // search that refuses everything.
    let open = drawing("walled-in-open", |probe| {
        probe.place("QUAD", "U1", ("76.2", ROW), &PINS);
        probe.place("QUAD", "U2", ("95.25", ROW), &PINS);
    });
    let clear = Bench::new(
        &open,
        pin_terminal(&open, "U1", "3"),
        pin_terminal(&open, "U2", "1"),
    );
    let route = clear.search();
    let route = route.route().expect("the way is open now");
    assert_eq!(route.path, vec![at("80.01", "101.6"), at("91.44", "101.6")]);
    assert!(
        steps_of(&route.path)
            .iter()
            .any(|&(point, _)| point == escape),
        "the open route goes through the cell the marker stood on"
    );
}

#[test]
fn the_queue_order_is_total() {
    // The queue orders on `(f, g, x, y, dir)`, so no tie is ever resolved by
    // heap internals: two entries that compare equal are the same entry. A cell
    // index grows with the coordinate it counts, so ordering on the cell is
    // ordering on x then y.
    let mut entries = Vec::new();
    for f in [0_i64, 7] {
        for g in [0_i64, 3] {
            for column in 0..3 {
                for row in 0..3 {
                    for dir in Heading::EVERY {
                        entries.push(Queued {
                            f,
                            g,
                            state: State {
                                at: Cell { column, row },
                                dir,
                            },
                        });
                    }
                }
            }
        }
    }
    assert_eq!(entries.len(), 2 * 2 * 3 * 3 * 4);

    // The order the module documents, written out independently of it: the
    // heading's place is the expansion order §4.3 fixes, `+x, −x, +y, −y`.
    let place = |heading: Heading| {
        Heading::EVERY
            .iter()
            .position(|&candidate| candidate == heading)
            .expect("every heading is in the expansion order")
    };
    let key = |entry: &Queued| {
        (
            entry.f,
            entry.g,
            entry.state.at.column,
            entry.state.at.row,
            place(entry.state.dir),
        )
    };

    for one in &entries {
        for other in &entries {
            let order = one.cmp(other);
            assert_eq!(
                order,
                other.cmp(one).reverse(),
                "the comparator is not antisymmetric: {one:?} against {other:?}"
            );
            assert_eq!(
                order,
                key(one).cmp(&key(other)),
                "the order is not (f, g, x, y, dir): {one:?} against {other:?}"
            );
            if one.state != other.state {
                assert_ne!(
                    order,
                    std::cmp::Ordering::Equal,
                    "two distinct states compare equal: {one:?} against {other:?}"
                );
            }
        }
    }

    // Transitive, over a slice small enough to take every triple.
    let few = &entries[..24];
    for one in few {
        for other in few {
            for third in few {
                if one.cmp(other).is_le() && other.cmp(third).is_le() {
                    assert!(
                        one.cmp(third).is_le(),
                        "the order is not transitive: {one:?}, {other:?}, {third:?}"
                    );
                }
            }
        }
    }

    // And the control: the generated pairs really do differ in every field, so
    // the assertions above are not passing on one entry compared with itself.
    let states: BTreeSet<State> = entries.iter().map(|entry| entry.state).collect();
    assert_eq!(states.len(), 3 * 3 * 4);
    assert!(entries.iter().any(|entry| entry.f != entries[0].f));
    assert!(entries.iter().any(|entry| entry.g != entries[0].g));
}

#[test]
fn kicad_reads_the_a_star_route_as_one_net() {
    // The route the search found is a polyline of five wire records. Whether
    // KiCad reads five records that meet end to end as one net is a question
    // about KiCad, so it is asked of KiCad rather than of the arithmetic.
    let Some(kicad) = Kicad::found_or_skip("read an A* route back as a netlist") else {
        return;
    };

    let planned = Hierarchy::load(&maze("maze-oracle-plan")).expect("the probe loads");
    let bench = Bench::new(
        &planned,
        pin_terminal(&planned, "U1", "3"),
        pin_terminal(&planned, "U2", "1"),
    );
    let search = bench.search();
    let route = search.route().expect("A* threads the maze").path.clone();
    assert_eq!(route.len(), 6, "five wire records: {route:?}");

    let drawn = |name: &str, legs: &[(Point, Point)]| -> PathBuf {
        let mut probe = Probe::new(name, scratch());
        probe.define(quad());
        probe.place("QUAD", "U1", ("76.2", ROW), &PINS);
        probe.place("QUAD", "U2", ("114.3", ROW), &PINS);
        wall(&mut probe, "88.9", Iu(mm(ROW).0 + 4 * GRID.0));
        wall(&mut probe, "99.06", mm(ROW));
        for &(from, to) in legs {
            probe.wire(
                (&from.x.to_string(), &from.y.to_string()),
                (&to.x.to_string(), &to.y.to_string()),
            );
        }
        probe.write()
    };
    let legs: Vec<(Point, Point)> = route.windows(2).map(|leg| (leg[0], leg[1])).collect();

    let together = |netlist: &kicli_probe::Netlist| {
        netlist
            .partition()
            .iter()
            .any(|net| net.iter().any(|pin| pin == "U1.3") && net.iter().any(|pin| pin == "U2.1"))
    };
    let joined = kicad.netlist_beside(&drawn("maze-oracle", &legs));
    assert!(
        together(&joined),
        "KiCad did not read the route as a connection: {:?}",
        joined.partition()
    );

    // The control that must fire: the same route one grid step short of the
    // target leaves the two pins on separate nets. Without it the assertion
    // above would pass on a netlist that joined everything.
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
    let apart = kicad.netlist_beside(&drawn("maze-oracle-short", &short));
    assert!(
        !together(&apart),
        "a wire that stops short still joined them, so the probe measures nothing: {:?}",
        apart.partition()
    );
}
