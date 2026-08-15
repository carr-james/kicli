//! Drawing a wire through vertices the caller states.
//!
//! `wire draw` does no searching. The caller gives the corners and kicli says
//! whether they are drawable: on the grid, along one axis, clear of everything
//! a wire may not pass through, and leaving each end the way that end must be
//! left. A refusal names the vertex it is about and writes nothing.
//!
//! Every drawing here is built by the probe harness, except the one that
//! measures how many lines a write touches: that one is a file KiCad itself
//! wrote, because the question is about bytes.
//!
//! The netlist oracle runs only with `KICLI_TEST_KICAD_CLI` set. Without it the
//! connectivity claim is kicli's own extractor; with it, KiCad is asked about
//! the file kicli wrote.

use kicli::connectivity::extract;
use kicli::edit::mark::PinAddress;
use kicli::edit::wire::{End, Polyline, WireError, draw};
use kicli::geometry::{GRID, Iu, Point};
use kicli::model::{Config, Hierarchy, Refdes, SheetPath, Target, WriteOptions};
use kicli_probe::oracle::{Kicad, Partition, kicli_partition, net};
use kicli_probe::scratch::Fixtures;
use kicli_probe::{Port, Probe, rectangle, symbol};
use kicli_sexpr::changed_line_count;
use std::path::{Path, PathBuf};

/// Where this binary writes the drawings it builds.
fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("wire-draw")
}

/// The committed fixtures this binary reads, and the scratch it writes in.
fn fixtures() -> Fixtures {
    Fixtures::new(env!("CARGO_TARGET_TMPDIR"), env!("CARGO_MANIFEST_DIR"))
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

/// A symbol that is a body and nothing else, which a route may not pass
/// through.
///
/// It carries no pin on purpose: a pin would block the route one step before
/// its body does, and the case this drawing exists for is the body.
fn box_symbol() -> String {
    symbol(
        "BOX",
        "U",
        false,
        &[("1_1", vec![rectangle(("-5.08", "-5.08"), ("5.08", "5.08"))])],
    )
}

/// Two resistors, pin 1 of each at `y = 50.8`, 25.4 mm apart.
///
/// A resistor's pin 1 sits above its anchor and its body below it, so a wire
/// leaves pin 1 upwards. That is the escape both ends of the drawings below
/// are drawn to honour.
fn two_resistors(name: &str) -> Probe {
    let mut probe = Probe::new(name, scratch());
    probe.place("R", "R1", ("50.8", "54.61"), &["1", "2"]);
    probe.place("R", "R2", ("76.2", "54.61"), &["1", "2"]);
    probe
}

/// The same, with a body between them that a route may not enter.
fn two_resistors_and_a_box(name: &str) -> Probe {
    let mut probe = two_resistors(name);
    probe.define(box_symbol());
    probe.place("BOX", "U1", ("63.5", "45.72"), &[]);
    probe
}

/// A sheet with one port on its right edge, and the child that port leads to.
///
/// The port is written on the edge its angle names, which is what the tool was
/// measured to require: a port whose angle disagrees with its position is
/// moved, and a wire drawn to where the file put it meets nothing.
fn a_sheet_with_a_port(name: &'static str) -> (Probe, Probe) {
    let mut probe = Probe::new(name, scratch());
    let mut child = Probe::child_of(&probe);
    probe.sheet_of_size(
        "00000000-0000-4000-8000-cccccccccccc",
        "child",
        ("101.6", "63.5"),
        ("25.4", "25.4"),
        &[Port {
            name: "OUT",
            at: ("127", "71.12"),
            angle: "0",
        }],
    );
    child.strand_of_kind(
        "hierarchical_label",
        "(shape bidirectional)",
        "RC1",
        "25.4",
        "29.21",
        "OUT",
    );
    (probe, child)
}

/// Pin 1 of a placed symbol, as a request names it.
fn pin_of(reference: &str) -> End {
    End::Pin(PinAddress::new(Refdes(reference.to_owned()), "1"))
}

/// Load a written drawing as a project rooted at it.
fn loaded(path: &Path) -> Hierarchy {
    Hierarchy::load(path).expect("the drawing loads")
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

/// What one request did, with the file's bytes before and after it.
///
/// The bytes are what a refusal is judged on. A check that read only the error
/// would pass just as happily against a command that wrote a broken file and
/// then complained about it.
struct Attempt {
    outcome: Result<usize, WireError>,
    before: String,
    after: String,
}

impl Attempt {
    /// The refusal, or a panic naming what was drawn instead.
    fn refusal(&self) -> String {
        match &self.outcome {
            Ok(segments) => panic!("the request drew {segments} segments instead of refusing"),
            Err(refusal) => refusal.to_string(),
        }
    }

    /// Assert that the file is exactly what it was.
    fn wrote_nothing(&self) {
        assert_eq!(self.before, self.after, "the file was written anyway");
    }
}

/// Draw a wire on a written drawing, keeping the bytes on both sides.
fn attempt(path: &Path, request: &Polyline) -> Attempt {
    let mut hierarchy = loaded(path);
    let sheet = hierarchy.placements[0].path.clone();
    let project = path.parent().expect("the drawing sits in a directory");
    let before = std::fs::read_to_string(path).expect("the drawing reads");
    let outcome = draw(
        &mut hierarchy,
        request,
        &Config::default().routing,
        &target(path, project, &sheet),
        "after",
    );
    let after = std::fs::read_to_string(path).expect("the drawing reads");
    Attempt {
        outcome: outcome.map(|wire| wire.report.segments()),
        before,
        after,
    }
}

/// What KiCad says about a file, when the environment asked for the tool.
fn oracle(path: &Path) -> Option<Partition> {
    Kicad::found().map(|kicad| kicad.netlist_beside(path).partition())
}

#[test]
fn explicit_vertices_are_validated_before_anything_is_written() {
    // Each case is one fault on one drawing. The refusal must name the vertex
    // the fault is at, and the file must be byte for byte what it was.
    let cases: [(&str, Vec<Point>, &str); 3] = [
        (
            // A diagonal, which no KiCad wire can hold.
            "diagonal",
            vec![at("50.8", "45.72")],
            "(76.2,50.8)",
        ),
        (
            // A corner that misses the 1.27 mm lattice by half a step.
            "off-grid",
            vec![
                at("50.8", "45.72"),
                at("76.835", "45.72"),
                at("76.835", "44.45"),
                at("76.2", "44.45"),
            ],
            "(76.835,45.72)",
        ),
        (
            // Straight through the body of U1, which is a hard block.
            "through-a-body",
            vec![at("50.8", "45.72"), at("76.2", "45.72")],
            "U1",
        ),
    ];

    for (name, via, named) in cases {
        let probe = two_resistors_and_a_box(&format!("refused-{name}"));
        let path = probe.write();
        let request = Polyline {
            from: pin_of("R1"),
            to: pin_of("R2"),
            via,
        };
        let attempt = attempt(&path, &request);

        let refusal = attempt.refusal();
        assert!(
            refusal.contains(named),
            "{name}: the refusal does not name {named}: {refusal}"
        );
        attempt.wrote_nothing();
        assert!(
            !attempt.after.contains("(wire"),
            "{name}: a wire record reached the file"
        );
    }
}

#[test]
fn an_off_grid_vertex_is_refused_rather_than_snapped() {
    // The half of the off-grid case the assertion above cannot see: kicli does
    // not quietly move the corner to the nearest lattice point and draw a wire
    // the caller did not ask for.
    let probe = two_resistors("off-grid-is-not-snapped");
    let path = probe.write();
    let request = Polyline {
        from: pin_of("R1"),
        to: pin_of("R2"),
        via: vec![
            at("50.8", "45.72"),
            at("76.835", "45.72"),
            at("76.835", "44.45"),
            at("76.2", "44.45"),
        ],
    };
    let attempt = attempt(&path, &request);
    assert!(
        attempt.refusal().contains("off the grid"),
        "the refusal says what is wrong with the vertex"
    );
    attempt.wrote_nothing();
    assert!(
        !attempt.after.contains("76.835"),
        "the corner reached the file"
    );
}

#[test]
fn a_wire_must_leave_each_end_the_way_that_end_is_left() {
    // A resistor's pin 1 is left upwards, because its body is below it. A
    // request that turns sideways at the pin is refused, and the refusal says
    // which point the wire has to pass through.
    let probe = two_resistors("escape-is-honoured");
    let path = probe.write();
    let sideways = Polyline {
        from: pin_of("R1"),
        to: pin_of("R2"),
        via: vec![at("63.5", "50.8"), at("63.5", "45.72"), at("76.2", "45.72")],
    };
    let attempt = attempt(&path, &sideways);
    let refusal = attempt.refusal();
    assert!(
        refusal.contains("R1.1"),
        "the refusal names the end it is about: {refusal}"
    );
    assert!(
        refusal.contains("(50.8,49.53)"),
        "the refusal says where the wire must pass: {refusal}"
    );
    attempt.wrote_nothing();
}

#[test]
fn a_wire_leaves_a_port_the_way_the_measured_rule_says() {
    // The sheet-pin rule this task measured, as the drawing verb enforces it.
    // A port on the right edge is left rightwards, away from the sheet body.
    let (probe, child) = a_sheet_with_a_port("port-escape-outward");
    let path = probe.write_all(&[&child]);
    let outward = Polyline {
        from: End::Port("OUT".to_owned()),
        to: End::At(at("137.16", "71.12")),
        via: Vec::new(),
    };
    let drawn = attempt(&path, &outward);
    assert_eq!(
        drawn.outcome.as_ref().ok(),
        Some(&1),
        "a wire may leave a port outwards: {:?}",
        drawn.outcome.as_ref().err().map(ToString::to_string)
    );

    // The same port, the same length of wire, the other way: into the body of
    // the sheet the port belongs to.
    let (probe, child) = a_sheet_with_a_port("port-escape-inward");
    let path = probe.write_all(&[&child]);
    let inward = Polyline {
        from: End::Port("OUT".to_owned()),
        to: End::At(at("116.84", "71.12")),
        via: Vec::new(),
    };
    let attempt = attempt(&path, &inward);
    let refusal = attempt.refusal();
    assert!(
        refusal.contains("OUT"),
        "the refusal names the port: {refusal}"
    );
    assert!(
        refusal.contains("(128.27,71.12)"),
        "the refusal says where a wire leaves the port: {refusal}"
    );
    attempt.wrote_nothing();
}

#[test]
fn a_drawn_polyline_is_one_record_per_segment() {
    let probe = two_resistors("one-record-per-segment");
    let path = probe.write();
    let project = path.parent().expect("the drawing sits in a directory");
    let mut hierarchy = loaded(&path);
    let sheet = hierarchy.placements[0].path.clone();

    // Up from R1.1, across, and down into R2.1: three segments.
    let request = Polyline {
        from: pin_of("R1"),
        to: pin_of("R2"),
        via: vec![at("50.8", "45.72"), at("76.2", "45.72")],
    };
    let drawn = draw(
        &mut hierarchy,
        &request,
        &Config::default().routing,
        &target(&path, project, &sheet),
        "after",
    )
    .expect("the polyline is drawable");

    assert_eq!(drawn.report.segments(), 3, "three segments");
    assert_eq!(
        drawn.report.added.wires.len(),
        3,
        "one record per segment, because a KiCad wire is always two points"
    );
    let written = std::fs::read_to_string(&path).expect("the drawing reads");
    assert_eq!(
        written.matches("(wire").count(),
        3,
        "the file holds one wire record per segment:\n{written}"
    );
    for uuid in &drawn.report.added.wires {
        assert!(written.contains(&uuid.0), "the report names what it wrote");
    }
    // Every invariant ran on what was written, and every one held.
    assert!(
        drawn.mutation.invariants.passed(),
        "the invariants did not hold: {}",
        drawn.mutation.render()
    );

    // The extractor reports the connection the caller asked for.
    let reloaded = loaded(&path);
    let joined = net(&["R1.1", "R2.1"]);
    assert!(
        kicli_partition(&extract(&reloaded)).contains(&joined),
        "kicli does not report R1.1 joined to R2.1"
    );
    // And so does KiCad, when the tool was asked for.
    if let Some(kicad) = oracle(&path) {
        assert!(
            kicad.contains(&joined),
            "KiCad does not report R1.1 joined to R2.1: {kicad:?}"
        );
    }
}

#[test]
fn a_crossing_is_named_as_well_as_counted() {
    // A wire of another net, met across its own axis, is allowed and costed.
    // The report names it, because a count with no name tells an agent that a
    // route is dear and not what to move.
    let mut probe = two_resistors("crossing-is-named");
    probe.wire(("63.5", "40.64"), ("63.5", "50.8"));
    let path = probe.write();
    let project = path.parent().expect("the drawing sits in a directory");
    let mut hierarchy = loaded(&path);
    let sheet = hierarchy.placements[0].path.clone();

    let request = Polyline {
        from: pin_of("R1"),
        to: pin_of("R2"),
        via: vec![at("50.8", "45.72"), at("76.2", "45.72")],
    };
    let drawn = draw(
        &mut hierarchy,
        &request,
        &Config::default().routing,
        &target(&path, project, &sheet),
        "after",
    )
    .expect("a crossing is drawable");

    assert_eq!(drawn.report.tally.crossings, 1, "one crossing was counted");
    assert_eq!(
        drawn.report.crossings.len(),
        1,
        "and the same crossing was named"
    );
    assert_eq!(drawn.report.crossings[0].at, at("63.5", "45.72"));
    assert!(drawn.report.cost.crossings > 0, "and it was priced");
}

#[test]
fn a_wire_command_changes_only_its_own_lines() {
    // P3, on a file KiCad wrote. Every line the command did not add must be
    // byte-identical, and the count of changed lines must not exceed the lines
    // the new records occupy.
    let project = fixtures().scratch("edit_wire_p3");
    let file = fixtures().copy_file(&project, "sch/multi_instance/channel.kicad_sch");
    let before = std::fs::read_to_string(&file).expect("the fixture reads");

    let mut hierarchy = loaded(&file);
    let sheet = hierarchy.placements[0].path.clone();
    let request = Polyline {
        from: End::At(at("76.2", "76.2")),
        to: End::At(at("88.9", "88.9")),
        via: vec![at("88.9", "76.2")],
    };
    let drawn = draw(
        &mut hierarchy,
        &request,
        &Config::default().routing,
        &target(&file, &project, &sheet),
        "after",
    )
    .expect("the wire is drawable");
    let after = std::fs::read_to_string(&file).expect("the written file reads");

    // The fixture carried no wire, so every wire record in the written file is
    // one this command added. Without that, the comparison below would drop
    // lines the command never touched and pass by removing the evidence.
    assert!(
        !before.contains("(wire"),
        "the fixture already held a wire, so this comparison proves nothing"
    );
    assert_eq!(
        drawn.report.added.wires.len(),
        2,
        "one record per segment was reported"
    );
    for uuid in &drawn.report.added.wires {
        assert!(after.contains(&uuid.0), "the report names what it wrote");
    }
    assert_eq!(
        without_wires(&before),
        without_wires(&after),
        "every line the command did not add is byte-identical"
    );

    // The bound the command documents: its own records and nothing else.
    let bound = after.lines().count() - before.lines().count();
    let changed = changed_line_count(&before, &after);
    assert!(bound > 0, "the command wrote something to be bounded");
    assert!(
        changed <= bound,
        "{changed} lines changed, which is past the {bound} the new records occupy"
    );
}

/// Every line of a file except the ones its `wire` records occupy.
///
/// The records are found by bracket balance from the line that opens one, so a
/// record written over several lines is taken whole.
fn without_wires(text: &str) -> Vec<&str> {
    let balance = |line: &str| -> i32 {
        let count = |bracket: char| i32::try_from(line.matches(bracket).count()).unwrap_or(0);
        count('(') - count(')')
    };
    let mut kept = Vec::new();
    let mut depth = 0;
    for line in text.lines() {
        if depth == 0 && !line.trim_start().starts_with("(wire") {
            kept.push(line);
            continue;
        }
        depth += balance(line);
    }
    kept
}
