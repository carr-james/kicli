//! What a route meets on the way, measured on drawings rather than on lists.
//!
//! Every check here builds a probe drawing, loads it, and sorts it onto the
//! lattice the way a route request does. A hand-built list of rectangles would
//! encode the same assumption as the code that reads one, so it would agree
//! with a body box in the wrong place; a drawing cannot.
//!
//! The drawings are the smallest that carry one object each, because the table
//! under test is one row per object.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use kicli::geometry::{GRID, Iu, Point, Rect};
use kicli::model::{Config, Hierarchy, Uuid};
use kicli::route::{
    Cell, Feature, Heading, Obstacles, Routed, SheetObjects, Treatment, Window, page_area,
};
use kicli_probe::drawing::LabelKind;
use kicli_probe::{Probe, pin, rectangle, symbol};

/// Where this binary writes the drawings it builds.
fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("route-obstacles")
}

/// The window margin the configuration gives, which is what a request uses.
fn margin() -> Iu {
    Config::default().routing.margin
}

/// A point from the millimetres the drawing is written in.
///
/// The drawing and the query then read the same number, so a test cannot ask
/// about a point the drawing never meant.
fn at(x: &str, y: &str) -> Point {
    let read = |text: &str| {
        Iu::from_millimetres_text(text)
            .expect("a millimetre reading")
            .0
    };
    Point::new(read(x), read(y))
}

/// Write a probe drawing and load it.
fn drawing(name: &str, draw: impl FnOnce(&mut Probe)) -> Hierarchy {
    let mut probe = Probe::new(name, scratch());
    draw(&mut probe);
    Hierarchy::load(&probe.write()).expect("the probe loads")
}

/// The same, for a drawing that places a child sheet.
fn drawing_with_child(name: &str, draw: impl FnOnce(&mut Probe)) -> Hierarchy {
    let mut probe = Probe::new(name, scratch());
    let child = Probe::child_of(&probe);
    draw(&mut probe);
    Hierarchy::load(&probe.write_all(&[&child])).expect("the probe loads")
}

/// The router's lists, for the root sheet of a loaded drawing.
fn objects(hierarchy: &Hierarchy, routed: &Routed) -> SheetObjects {
    let file = hierarchy.files.first().expect("the probe has a root sheet");
    let path = &hierarchy
        .placements
        .first()
        .expect("the root sheet is placed")
        .path;
    SheetObjects::read(file, path, routed)
}

/// The obstacle map of a window around two points.
fn map(objects: &SheetObjects, from: Point, to: Point) -> Obstacles {
    let window = Window::around(from, to, margin(), objects.page(), GRID);
    Obstacles::build(window, &objects.geometry())
}

/// A symbol with a square body and one pin on each of its four edges.
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

/// A symbol that is a body and nothing else, whose box misses the grid.
///
/// The half-side is 1.5 grid steps, so no edge of the box lands on a grid
/// point. A symbol with pins would put its box on the pins, which are on grid.
fn off_grid_body() -> String {
    symbol(
        "BOX",
        "U",
        false,
        &[(
            "1_1",
            vec![rectangle(("-1.905", "-1.905"), ("1.905", "1.905"))],
        )],
    )
}

/// The row of `research/wire-routing.md` §3.2 a feature belongs to.
///
/// The match is exhaustive over a closed set, so a new schematic object type is
/// a compile error here as well as at the site that classifies it. That is what
/// makes the coverage assertion below worth making.
fn row(feature: &Feature) -> &'static str {
    match feature {
        Feature::SymbolBody(_) => "symbol body",
        Feature::SheetBody(_) => "sheet body",
        Feature::ForeignPin(_) => "another symbol's pin",
        Feature::PinHalo(_) => "the step that pin escapes along",
        Feature::Junction(_) => "junction",
        Feature::NoConnect(_) => "no-connect",
        Feature::ForeignWire { .. } => "another net's wire",
        Feature::OwnWire(_) => "this net's wire",
        Feature::TextBox(_) => "a label or text box",
        Feature::NearBody(_) => "within one step of a body",
    }
}

/// Every row of the table, which the check below must reach.
const ROWS: [&str; 10] = [
    "symbol body",
    "sheet body",
    "another symbol's pin",
    "the step that pin escapes along",
    "junction",
    "no-connect",
    "another net's wire",
    "this net's wire",
    "a label or text box",
    "within one step of a body",
];

/// The one feature at a point that a report would call `handle`.
fn feature<'a>(map: &'a Obstacles, at: Point, handle: &str) -> &'a Feature {
    let cell = map
        .window()
        .cell(at)
        .unwrap_or_else(|| panic!("{at} is not a node of the window"));
    let mut found = map
        .features(cell)
        .iter()
        .filter(|feature| feature.handle() == handle);
    let feature = found.next().unwrap_or_else(|| {
        panic!(
            "nothing at {at} is called {handle}: {:?}",
            map.features(cell)
        )
    });
    assert!(
        found.next().is_none(),
        "two features at {at} are called {handle}"
    );
    feature
}

#[test]
fn every_obstacle_kind_is_classified() {
    // A probe's identifiers come from a counter, so every object of a drawing
    // is called the same eight characters. Each case therefore states the row
    // it expects as well as the handle, or a feature classified as the wrong
    // kind would pass for its neighbour.
    let mut seen: BTreeSet<&'static str> = BTreeSet::new();
    let mut classify =
        |map: &Obstacles, point: Point, handle: &str, wanted_row, heading, wanted| {
            let feature = feature(map, point, handle);
            assert_eq!(row(feature), wanted_row, "at {point}: {feature:?}");
            assert_eq!(
                feature.treatment(heading),
                wanted,
                "{wanted_row} at {point}: {feature:?}"
            );
            seen.insert(row(feature));
        };

    // A symbol body, and the ring of crowded points one step outside it. The
    // probe symbol reaches 3.81 mm from its anchor, which is three grid steps.
    let symbols = drawing("body", |probe| {
        probe.define(quad());
        probe.place("QUAD", "U1", ("101.6", "101.6"), &["1", "2", "3", "4"]);
    });
    let symbols = objects(&symbols, &Routed::default());
    let symbols = map(&symbols, at("88.9", "88.9"), at("114.3", "114.3"));
    classify(
        &symbols,
        at("101.6", "101.6"),
        "U1",
        "symbol body",
        Heading::PlusX,
        Treatment::Block,
    );
    classify(
        &symbols,
        at("96.52", "101.6"),
        "U1",
        "within one step of a body",
        Heading::PlusX,
        Treatment::Near,
    );

    // A child sheet's body. Its own pins are terminals, and they sit on the
    // border, which the body already covers.
    let sheets = drawing_with_child("sheet-body", |probe| {
        probe.sheet("IN", ("101.6", "101.6"));
    });
    let sheets = objects(&sheets, &Routed::default());
    let sheets = map(&sheets, at("88.9", "88.9"), at("139.7", "139.7"));
    classify(
        &sheets,
        at("114.3", "114.3"),
        "child",
        "sheet body",
        Heading::PlusY,
        Treatment::Block,
    );

    // A pin of a symbol the route does not end on, and the step that pin needs
    // for its own escape. The probe resistor puts pin 1 three grid steps above
    // its anchor, pointing down into the body.
    let pins = drawing("foreign-pin", |probe| {
        probe.place("R", "R1", ("101.6", "101.6"), &["1", "2"]);
    });
    let pins = objects(&pins, &Routed::default());
    let pins = map(&pins, at("88.9", "88.9"), at("114.3", "114.3"));
    classify(
        &pins,
        at("101.6", "97.79"),
        "R1.1",
        "another symbol's pin",
        Heading::PlusX,
        Treatment::Block,
    );
    classify(
        &pins,
        at("101.6", "96.52"),
        "R1.1",
        "the step that pin escapes along",
        Heading::PlusX,
        Treatment::Block,
    );

    // A junction and a no-connect, each on a drawing that holds only it.
    let marks = drawing("marks", |probe| {
        probe.junction(("101.6", "101.6"));
        probe.no_connect(("104.14", "101.6"));
    });
    let marks = objects(&marks, &Routed::default());
    let junction = marks.geometry().junctions[0].handle.clone();
    let no_connect = marks.geometry().no_connects[0].handle.clone();
    let marks = map(&marks, at("88.9", "88.9"), at("114.3", "114.3"));
    classify(
        &marks,
        at("101.6", "101.6"),
        &junction,
        "junction",
        Heading::PlusX,
        Treatment::Block,
    );
    classify(
        &marks,
        at("104.14", "101.6"),
        &no_connect,
        "no-connect",
        Heading::PlusX,
        Treatment::Block,
    );

    // A wire, met along its own axis and across it, and the same wire when it
    // belongs to the net being routed.
    let wires = drawing("wires", |probe| {
        probe.wire(("96.52", "101.6"), ("106.68", "101.6"));
    });
    let mine = wires
        .files
        .first()
        .and_then(|file| file.schematic.lines().next())
        .map(|line| line.uuid.clone())
        .expect("the probe draws one wire");
    let foreign = objects(&wires, &Routed::default());
    let handle = foreign.geometry().segments[0].handle.clone();
    let foreign = map(&foreign, at("88.9", "88.9"), at("114.3", "114.3"));
    classify(
        &foreign,
        at("101.6", "101.6"),
        &handle,
        "another net's wire",
        Heading::PlusX,
        Treatment::Block,
    );
    classify(
        &foreign,
        at("101.6", "101.6"),
        &handle,
        "another net's wire",
        Heading::PlusY,
        Treatment::Cross,
    );
    let own = objects(
        &wires,
        &Routed {
            wires: std::slice::from_ref(&mine),
            terminals: &[],
        },
    );
    let own = map(&own, at("88.9", "88.9"), at("114.3", "114.3"));
    classify(
        &own,
        at("101.6", "101.6"),
        &handle,
        "this net's wire",
        Heading::PlusX,
        Treatment::Terminate,
    );

    // A label, which costs a route that runs through what it says.
    let labels = drawing("label", |probe| {
        probe.label_of_kind(LabelKind::Local, "NET_A", ("101.6", "101.6"));
    });
    let labels = objects(&labels, &Routed::default());
    let text = labels.geometry().texts[0].handle.clone();
    let labels = map(&labels, at("88.9", "88.9"), at("114.3", "114.3"));
    classify(
        &labels,
        at("101.6", "101.6"),
        &text,
        "a label or text box",
        Heading::PlusX,
        Treatment::Text,
    );

    let measured: BTreeSet<&'static str> = ROWS.into_iter().collect();
    assert_eq!(seen, measured, "every row of the table is measured");
}

#[test]
fn a_collinear_wire_blocks_and_a_crossing_does_not() {
    // One drawing, one map, one cell, two headings. Two drawings would measure
    // two maps and leave the heading dependence itself unmeasured, which is the
    // one thing this check exists for.
    let hierarchy = drawing("heading-dependence", |probe| {
        probe.wire(("96.52", "101.6"), ("106.68", "101.6"));
    });
    let objects = objects(&hierarchy, &Routed::default());
    let handle = objects.geometry().segments[0].handle.clone();
    let map = map(&objects, at("88.9", "88.9"), at("114.3", "114.3"));
    let point = at("101.6", "101.6");
    let cell = map.window().cell(point).expect("the wire is in the window");
    assert_eq!(map.features(cell).len(), 1, "the cell holds only the wire");

    for along in [Heading::PlusX, Heading::MinusX] {
        let verdict = map.entering(point, along);
        assert!(!verdict.is_allowed(), "{along:?} runs along the wire");
        assert_eq!(verdict.blocked_by.as_deref(), Some(handle.as_str()));
        assert_eq!(verdict.crossings, 0, "a block is not a crossing");
    }
    for across in [Heading::PlusY, Heading::MinusY] {
        let verdict = map.entering(point, across);
        assert!(verdict.is_allowed(), "{across:?} crosses the wire");
        assert_eq!(verdict.blocked_by, None);
        assert_eq!(verdict.crossings, 1, "a crossing is costed once");
    }
}

#[test]
fn the_window_is_clipped_to_the_page() {
    let hierarchy = drawing("page-edge", |probe| {
        probe.place("R", "R1", ("5.08", "12.7"), &["1", "2"]);
    });
    let file = hierarchy.files.first().expect("the probe has a root sheet");
    let objects = objects(&hierarchy, &Routed::default());
    // A4, as the probe writes it: 11693 × 8268 mils.
    let page = objects.page();
    assert_eq!(page, page_area(&file.doc));
    assert_eq!(
        page,
        Rect::new(Point::default(), Point::new(2_970_022, 2_100_072))
    );

    // Pin 1 of the resistor sits four grid steps from the left edge, which is
    // less than the margin, so the window would reach off the page.
    let terminal = at("5.08", "8.89");
    let across = at("290.83", "203.2");
    let window = Window::around(terminal, across, margin(), page, GRID);
    assert_eq!(window.area().start(), page.start(), "the page wins");
    assert_eq!(window.area().end(), page.end(), "and wins on the far side");

    // The control: the same terminals on a page big enough for the margin keep
    // the whole margin, so it is the clip that moved the corner.
    let roomy = Rect::new(
        Point::new(-margin().0, -margin().0),
        Point::new(page.end().x.0 + margin().0, page.end().y.0 + margin().0),
    );
    let unclipped = Window::around(terminal, across, margin(), roomy, GRID);
    assert_eq!(
        unclipped.area().start(),
        Point::new(terminal.x.0 - margin().0, terminal.y.0 - margin().0)
    );

    // No lattice node lands outside the border.
    let map = map(&objects, terminal, across);
    let columns = window.area().width().0 / GRID.0;
    let rows = window.area().height().0 / GRID.0;
    for column in 0..=columns {
        for row in 0..=rows {
            let point = window.point(Cell { column, row });
            assert!(page.contains(point), "the node at {point} is off the page");
            assert!(window.cell(point).is_some(), "{point} is a node");
        }
    }

    // And the first point past the last node is refused, naming the border.
    let last = window.point(Cell {
        column: columns,
        row: rows,
    });
    assert!(map.entering(last, Heading::PlusX).is_allowed(), "{last}");
    for beyond in [
        Point::new(last.x.0 + GRID.0, last.y.0),
        Point::new(last.x.0, last.y.0 + GRID.0),
        Point::new(-GRID.0, last.y.0),
    ] {
        assert_eq!(window.cell(beyond), None, "{beyond} is not a node");
        assert_eq!(
            map.entering(beyond, Heading::PlusX).blocked_by.as_deref(),
            Some("page border"),
            "{beyond}"
        );
    }
}

#[test]
fn a_body_off_the_grid_still_marks_the_points_it_covers() {
    // The lattice belongs to the window, and a body box does not have to land
    // on it. A fixture whose boxes start on grid agrees with an implementation
    // that aligns the box to absolute zero and with one that does not, so it
    // measures neither.
    let hierarchy = drawing("off-grid-body", |probe| {
        probe.define(off_grid_body());
        probe.place("BOX", "U2", ("101.6", "101.6"), &[]);
    });
    let objects = objects(&hierarchy, &Routed::default());
    let body = objects.geometry().symbol_bodies[0].area;
    assert!(
        !body.start().is_on_grid() && !body.end().is_on_grid(),
        "the body box {body} is on the grid, so this check measures nothing"
    );

    let map = map(&objects, at("88.9", "88.9"), at("114.3", "114.3"));
    let window = map.window();
    let mut covered = Vec::new();
    for column in 0..=window.area().width().0 / GRID.0 {
        for row in 0..=window.area().height().0 / GRID.0 {
            let cell = Cell { column, row };
            let point = window.point(cell);
            let blocked = map
                .features(cell)
                .iter()
                .any(|feature| matches!(feature, Feature::SymbolBody(_)));
            assert_eq!(
                blocked,
                body.contains(point),
                "{point} is {} the body {body}",
                if blocked {
                    "marked outside"
                } else {
                    "inside and unmarked"
                }
            );
            if blocked {
                covered.push(point);
            }
        }
    }

    // Three grid points on each axis fall inside a box 3.81 mm on a side.
    assert_eq!(covered.len(), 9, "{covered:?}");
    assert!(covered.contains(&at("100.33", "100.33")));
    assert!(covered.contains(&at("102.87", "102.87")));
}

#[test]
fn a_sheet_becomes_the_lists_the_search_reads() {
    let hierarchy = drawing_with_child("every-list", |probe| {
        probe.place("R", "R1", ("101.6", "101.6"), &["1", "2"]);
        probe.sheet("IN", ("139.7", "101.6"));
        probe.wire(("101.6", "105.41"), ("114.3", "105.41"));
        probe.junction(("114.3", "105.41"));
        probe.no_connect(("101.6", "97.79"));
        probe.label_of_kind(LabelKind::Local, "NET_A", ("107.95", "105.41"));
        probe.free_text("a note", ("101.6", "116.84"));
    });
    let wire = hierarchy
        .files
        .first()
        .and_then(|file| file.schematic.lines().next())
        .map(|line| line.uuid.clone())
        .expect("the probe draws one wire");

    let read = objects(&hierarchy, &Routed::default());
    let geometry = read.geometry();
    assert_eq!(geometry.symbol_bodies.len(), 1);
    assert_eq!(geometry.sheet_bodies.len(), 1);
    assert_eq!(geometry.junctions.len(), 1);
    assert_eq!(geometry.no_connects.len(), 1);
    assert_eq!(geometry.segments.len(), 1);
    assert_eq!(geometry.texts.len(), 2, "a label and a piece of free text");

    // A pin is named the way a report names it, and knows which way it escapes.
    let pins: Vec<&str> = geometry
        .pins
        .iter()
        .map(|pin| pin.handle.as_str())
        .collect();
    assert_eq!(pins, ["R1.1", "R1.2"]);
    assert_eq!(geometry.pins[0].at, at("101.6", "97.79"));
    assert_eq!(geometry.pins[0].escape, Some(Heading::MinusY));
    assert_eq!(geometry.pins[1].escape, Some(Heading::PlusY));

    // Whose the wire is, is the caller's answer and not the reader's.
    assert!(!geometry.segments[0].own_net);
    let routed = objects(
        &hierarchy,
        &Routed {
            wires: std::slice::from_ref(&wire),
            terminals: &["R1.1".to_owned(), "R1.2".to_owned()],
        },
    );
    let routed = routed.geometry();
    assert!(routed.segments[0].own_net, "the caller named this wire");
    assert!(
        routed.pins.is_empty(),
        "a pin the route ends on is a terminal, not an obstacle"
    );

    // A wire is addressed by the eight characters a caller may type back.
    let handle = &geometry.segments[0].handle;
    assert_eq!(handle.len(), 8);
    assert!(
        wire.0.starts_with(handle.as_str()),
        "{handle} of {}",
        wire.0
    );
    assert_eq!(Uuid(wire.0.clone()).short(), handle.as_str());
}
