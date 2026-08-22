//! `KI-GRID-001` sees every connectable class, and none of the exempt ones.
//!
//! # Why there are six drawings and not one
//!
//! A single drawing carrying one off-grid object of every class is a
//! **degenerate fixture**: a rule that examined pins and nothing else would
//! still find *something* on it, and the check would pass. So each class gets
//! a drawing of its own, in which every other class is on the grid. A rule
//! that stopped looking at junctions fails the junction drawing and no other,
//! which is what makes the six of them a coverage measurement rather than one
//! assertion repeated.
//!
//! # Why nothing here asserts a number of findings
//!
//! "N findings" fails identically when the rule **stops examining a class**
//! and when it **double-reports one**, and the two are opposite defects. Every
//! check below asserts the whole list of `(rule, position)` pairs, in report
//! order, so a missed class and a doubled one are different failures with
//! different messages.
//!
//! # The standing exemption control
//!
//! Every drawing this file builds carries one piece of **off-grid free text**,
//! whatever class is being displaced. So a widening of the rule to text
//! positions does not fail one check here; it fails all of them.

use std::path::{Path, PathBuf};

use kicli::geometry::Point;
use kicli::lint::gate::{Blocker, Report};
use kicli::lint::score::Density;
use kicli::lint::{Drawing, Engine, Finding, Rule, RuleId, Severity, Tier};
use kicli::model::items::{Item, SheetPath};
use kicli::model::{Hierarchy, LoadedFile, Schematic};
use kicli_probe::drawing::LabelKind;
use kicli_probe::{Port, Probe};
use kicli_sexpr::Doc;

/// The rule under test.
const GRID: RuleId = RuleId("KI-GRID-001");

/// The kind KiCad's own electrical rule check reports for an off-grid point.
const ERC_OFF_GRID: &str = "endpoint_off_grid";

/// The uuid of the sheet symbol every drawing here draws.
///
/// The probe crate keeps one of its own, and it is private. Naming one here
/// lets the port be written somewhere other than the sheet's corner, which is
/// what the sheet-pin drawing needs.
const CHILD: &str = "00000000-0000-4000-8000-00000000cccc";

// Every coordinate below is in internal units, written out rather than
// computed from the millimetre text the drawing is built from. The conversion
// is 10 000 internal units to the millimetre and the grid is 12 700 of them,
// both from `spec/SPEC.md` §5.2. A coordinate derived from the same string the
// drawing was written with would share an ancestor with the thing it checks.

/// The pin of the skewed symbol, half a grid step to the right of its anchor.
const SKEWED_PIN: Point = Point::new(514_350, 469_900);
/// The far end of the wire, displaced.
const WIRE_END: Point = Point::new(387_350, 254_000);
/// The junction, displaced.
const JUNCTION: Point = Point::new(641_350, 254_000);
/// The no-connect marker, displaced along Y rather than X.
const NO_CONNECT: Point = Point::new(762_000, 260_350);
/// The label anchor, displaced.
const LABEL: Point = Point::new(895_350, 254_000);
/// The sheet pin, displaced along Y so it stays on the sheet's right edge.
const SHEET_PIN: Point = Point::new(1_270_000, 895_350);

/// Which class of connectable point a drawing is built to displace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Class {
    /// Nothing is displaced. The control the other six stand on.
    Nothing,
    /// One pin of a symbol whose own anchor is on the grid.
    Pin,
    /// The anchor of a symbol whose pins are on the grid, which must be seen
    /// by nothing: a placement anchor is not a connection point.
    AnchorOnly,
    /// One end of one wire.
    WireEnd,
    /// The junction.
    Junction,
    /// The no-connect marker.
    NoConnect,
    /// The label's anchor.
    Label,
    /// The sheet's port.
    SheetPin,
}

/// Where the drawings this binary writes go.
fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("lint-grid")
}

/// The committed fixture tree.
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// A symbol whose pin 1 sits half a grid step to the right of its anchor.
///
/// Placing it on the grid puts one pin off the grid and leaves the other on
/// it. A rule reading the placement anchor instead of the resolved pin
/// position finds nothing on this drawing.
fn skewed() -> String {
    kicli_probe::symbol(
        "RSkew",
        "R",
        false,
        &[(
            "1_1",
            vec![
                kicli_probe::pin("passive", ("0.635", "3.81"), "270", "1", ""),
                kicli_probe::pin("passive", ("0", "-3.81"), "90", "2", ""),
            ],
        )],
    )
}

/// A symbol whose pins sit half a grid step to the LEFT of its anchor.
///
/// Placing it half a step to the right puts the anchor off the grid and both
/// pins back on it. It is the other half of the same measurement: the rule
/// must find nothing here.
fn counter_skewed() -> String {
    kicli_probe::symbol(
        "RBack",
        "R",
        false,
        &[(
            "1_1",
            vec![
                kicli_probe::pin("passive", ("-0.635", "3.81"), "270", "1", ""),
                kicli_probe::pin("passive", ("-0.635", "-3.81"), "90", "2", ""),
            ],
        )],
    )
}

/// One drawing carrying every connectable class, with one class displaced.
///
/// **The probe name is the check's own.** The tests of one binary run in
/// parallel and a probe writes to a path keyed by its name, so two checks
/// sharing a name write one file from two threads.
fn drawing_with(check: &str, class: Class) -> PathBuf {
    let mut probe = Probe::new(check, scratch());
    probe.define(skewed());
    probe.define(counter_skewed());

    // A symbol, and the pin that may be displaced. Displacing a pin is done by
    // changing the symbol drawn, not the place it is drawn, so the anchor is
    // on the grid in every one of the eight drawings.
    match class {
        Class::Pin => probe.place("RSkew", "R1", ("50.8", "50.8"), &["1", "2"]),
        Class::AnchorOnly => probe.place("RBack", "R1", ("51.435", "50.8"), &["1", "2"]),
        _ => probe.place("R", "R1", ("50.8", "50.8"), &["1", "2"]),
    }

    // A wire, with its far end displaced or not.
    let wire_end = if class == Class::WireEnd {
        "38.735"
    } else {
        "38.1"
    };
    probe.wire(("25.4", "25.4"), (wire_end, "25.4"));

    let junction_x = if class == Class::Junction {
        "64.135"
    } else {
        "63.5"
    };
    probe.junction((junction_x, "25.4"));

    let no_connect_y = if class == Class::NoConnect {
        "26.035"
    } else {
        "25.4"
    };
    probe.no_connect(("76.2", no_connect_y));

    let label_x = if class == Class::Label {
        "89.535"
    } else {
        "88.9"
    };
    probe.label_of_kind(LabelKind::Local, "NET", (label_x, "25.4"));

    let port_y = if class == Class::SheetPin {
        "89.535"
    } else {
        "88.9"
    };
    probe.sheet_of_size(
        CHILD,
        "child",
        ("101.6", "76.2"),
        ("25.4", "25.4"),
        &[Port {
            name: "PORT",
            at: ("127", port_y),
            angle: "0",
        }],
    );

    // The standing exemption control, in every drawing. Its x is half a grid
    // step off, and no check here ever expects a finding at it.
    probe.free_text("note", ("26.035", "101.6"));

    let child = Probe::named_child_of(&probe, "child", CHILD, 2);
    probe.write_all(&[&child])
}

/// Every placement of a loaded drawing, as the rules see it.
fn placements(hierarchy: &Hierarchy) -> Vec<Drawing<'_>> {
    hierarchy
        .placements
        .iter()
        .map(|placement| {
            let file: &LoadedFile = &hierarchy.files[placement.file];
            Drawing::read(&file.doc, &file.schematic, &placement.path)
        })
        .collect()
}

/// The rule, taken from the registry the build wrote.
///
/// Reading it out of the registry rather than naming the type is deliberate:
/// a rule that failed to register would make every check here run an empty
/// engine and pass, and [`the_rule_is_registered_from_its_own_file`] is what
/// refuses that.
fn rules() -> Vec<&'static dyn Rule> {
    kicli::lint::registry::all()
        .into_iter()
        .filter(|rule| rule.id() == GRID)
        .collect()
}

/// What the rule reported over a whole hierarchy, as `(rule, position)` pairs.
fn reported(path: &Path) -> Vec<(RuleId, Point)> {
    pairs(&findings(path))
}

/// Every finding of the rule over a whole hierarchy, in report order.
fn findings(path: &Path) -> Vec<Finding> {
    let hierarchy = Hierarchy::load(path).expect("the drawing loads");
    let drawings = placements(&hierarchy);
    Engine::of(rules()).examine_all(&drawings)
}

/// The `(rule, position)` pairs of a finding list, in the order it holds them.
fn pairs(found: &[Finding]) -> Vec<(RuleId, Point)> {
    found.iter().map(|one| (one.rule, one.pos)).collect()
}

#[test]
fn the_rule_is_registered_from_its_own_file() {
    // The presence control every other check in this file stands on. An engine
    // that ran no rules would report nothing about every drawing here, and
    // seven of the checks below assert exactly that about a clean one.
    let registered = rules();
    assert_eq!(
        registered.len(),
        1,
        "the rule registered once, from one file"
    );
    assert_eq!(registered[0].id(), GRID);
    assert_eq!(registered[0].tier(), Tier::One, "the rule blocks");
    assert_eq!(registered[0].severity(), Severity::Error);
    assert!(
        kicli::lint::registry::files().contains(&"grid"),
        "the registry names the file the rule came from: {:?}",
        kicli::lint::registry::files()
    );
}

#[test]
fn a_drawing_on_the_grid_reports_nothing() {
    // The control the six class checks stand on. Every one of them adds
    // exactly one displacement to this drawing, so a rule that fired on
    // anything here would make all six pass for the wrong reason.
    let path = drawing_with("clean", Class::Nothing);
    assert_eq!(reported(&path), Vec::new());
}

#[test]
fn an_off_grid_pin_is_found() {
    let path = drawing_with("pin", Class::Pin);
    assert_eq!(reported(&path), vec![(GRID, SKEWED_PIN)]);

    // The finding names the symbol and the pin instance, so a reader can reach
    // the object without going through a reference designator.
    let found = findings(&path);
    assert_eq!(found[0].objects.len(), 2, "{:?}", found[0].objects);
    assert_eq!(found[0].message, "the pin sits off the connection grid");
}

#[test]
fn a_symbol_anchor_off_the_grid_is_not_itself_a_finding() {
    // The other half of the pin measurement. Here the placement anchor is off
    // the grid and both resolved pins are on it, so a rule reading the anchor
    // reports one or two findings and this check fails. The pair is what
    // establishes that the rule reads resolved pin positions.
    let path = drawing_with("anchor", Class::AnchorOnly);
    assert_eq!(reported(&path), Vec::new());
}

#[test]
fn an_off_grid_wire_endpoint_is_found() {
    let path = drawing_with("wire", Class::WireEnd);
    assert_eq!(reported(&path), vec![(GRID, WIRE_END)]);
    // One end of the wire moved and the other did not, so a rule that reported
    // per wire rather than per end would name the wrong point.
    assert_eq!(
        findings(&path)[0].message,
        "the wire endpoint sits off the connection grid"
    );
}

#[test]
fn an_off_grid_junction_is_found() {
    let path = drawing_with("junction", Class::Junction);
    assert_eq!(reported(&path), vec![(GRID, JUNCTION)]);
}

#[test]
fn an_off_grid_no_connect_is_found() {
    let path = drawing_with("no-connect", Class::NoConnect);
    assert_eq!(reported(&path), vec![(GRID, NO_CONNECT)]);
}

#[test]
fn an_off_grid_label_anchor_is_found() {
    let path = drawing_with("label", Class::Label);
    assert_eq!(reported(&path), vec![(GRID, LABEL)]);
}

#[test]
fn an_off_grid_sheet_pin_is_found() {
    let path = drawing_with("sheet-pin", Class::SheetPin);
    assert_eq!(reported(&path), vec![(GRID, SHEET_PIN)]);
}

#[test]
fn off_grid_text_is_exempt_at_a_coordinate_the_rule_does_report() {
    // The exemption, with the control that makes it mean something. The free
    // text every drawing here carries sits at a point the rule reports
    // happily when a junction is put there instead. So the text is passed over
    // because of what it is, not because the rule cannot see that coordinate.
    let exempt = drawing_with("text-exempt", Class::Nothing);
    assert_eq!(reported(&exempt), Vec::new());

    let mut probe = Probe::new("text-control", scratch());
    probe.free_text("note", ("26.035", "101.6"));
    probe.junction(("26.035", "101.6"));
    let same_point = probe.write();
    assert_eq!(
        reported(&same_point),
        vec![(GRID, Point::new(260_350, 1_016_000))],
        "the rule does report that very coordinate, for a class that connects"
    );
}

#[test]
fn kicads_own_field_autoplacement_is_exempt() {
    // `research/geometry.md` Contradiction 2, as a file KiCad wrote. The
    // fixture's provenance in tests/fixtures/MANIFEST is kicad-cli, so the
    // off-grid field positions in it are KiCad's own autoplacement rather than
    // ours.
    let path = fixture("sch/item_zoo.kicad_sch");
    let source = std::fs::read_to_string(&path).expect("the fixture reads");
    let doc = Doc::parse(&source).expect("the fixture parses");
    let schematic = Schematic::read(&doc).expect("the fixture is a schematic");
    let sheet = SheetPath("/".to_owned());
    let drawing = Drawing::read(&doc, &schematic, &sheet);

    // Presence control 1: the fixture really does carry text positions off the
    // grid. Without it this check passes on a drawing that has none, which is
    // the "reads nothing" blindness.
    let off_grid_text = exempt_positions(&schematic)
        .into_iter()
        .filter(|at| !at.is_on_grid())
        .count();
    assert!(
        off_grid_text > 0,
        "the fixture carries text KiCad placed off the grid"
    );

    // Presence control 2: and it carries connectable geometry of several
    // classes, so reporting nothing about it is a decision rather than an
    // empty file.
    assert!(
        connectable_classes(&schematic) >= 5,
        "the fixture carries connectable geometry of at least five classes"
    );

    let found = Engine::of(rules()).examine(&drawing);
    assert_eq!(
        pairs(&found),
        Vec::new(),
        "field and graphic text are exempt"
    );
}

/// Every position the rule must never read: field and graphic text anchors.
///
/// The list is built from the typed model rather than from the rule, so a rule
/// that quietly widened its own idea of what a field is cannot widen this too.
fn exempt_positions(schematic: &Schematic) -> Vec<Point> {
    let mut at = Vec::new();
    for item in &schematic.items {
        match item {
            Item::Symbol(symbol) => at.extend(symbol.fields.iter().map(|field| field.at)),
            Item::Sheet(sheet) => at.extend(sheet.fields.iter().map(|field| field.at)),
            Item::Label(label) => at.extend(label.fields.iter().map(|field| field.at)),
            Item::Text(text) => at.push(text.at),
            _ => {}
        }
    }
    at
}

/// How many of the six connectable classes a drawing holds at all.
fn connectable_classes(schematic: &Schematic) -> usize {
    let mut classes = [false; 6];
    for item in &schematic.items {
        match item {
            Item::Symbol(symbol) => classes[0] = !symbol.pins.is_empty(),
            Item::Line(_) => classes[1] = true,
            Item::Junction(_) => classes[2] = true,
            Item::NoConnect(_) => classes[3] = true,
            Item::Label(_) => classes[4] = true,
            Item::Sheet(sheet) => classes[5] = !sheet.pins.is_empty(),
            Item::Other { .. } | Item::BusEntry(_) | Item::Text(_) => {}
        }
    }
    classes.iter().filter(|held| **held).count()
}

#[test]
fn one_off_grid_pin_fails_the_gate_and_leaves_the_score_alone() {
    // The tier, measured rather than declared. One finding is enough, and it
    // costs nothing: a blocking rule reports a verdict, not a number.
    let path = drawing_with("gate", Class::Pin);
    let hierarchy = Hierarchy::load(&path).expect("the drawing loads");
    let drawings = placements(&hierarchy);
    let root = drawings.first().expect("the drawing has a sheet");
    let density = Density::of(root);
    let found = Engine::of(rules()).examine_all(&drawings);

    let report = Report::of(&found, density);
    assert_eq!(report.score().score(), 100, "{}", report.text());
    assert!(!report.gate().passes(), "{}", report.text());
    assert_eq!(
        report.gate().blockers(),
        [Blocker::Blocking {
            rule: GRID,
            count: 1
        }]
    );

    // And the clean drawing of the same shape passes, so the verdict is the
    // finding's and not the drawing's.
    let clean = drawing_with("gate-clean", Class::Nothing);
    let clean_hierarchy = Hierarchy::load(&clean).expect("the drawing loads");
    let clean_drawings = placements(&clean_hierarchy);
    let clean_found = Engine::of(rules()).examine_all(&clean_drawings);
    assert!(
        Report::of(&clean_found, density).gate().passes(),
        "the same drawing without the displacement passes"
    );
}

/// Every class, with the drawing that displaces it and the point it lands on.
///
/// One table, read by the oracle check below, so the overlap with KiCad's own
/// check is stated once for all six classes rather than argued class by class.
const CLASSES: [(Class, &str, Point); 6] = [
    (Class::Pin, "pin", SKEWED_PIN),
    (Class::WireEnd, "wire-end", WIRE_END),
    (Class::Junction, "junction", JUNCTION),
    (Class::NoConnect, "no-connect", NO_CONNECT),
    (Class::Label, "label", LABEL),
    (Class::SheetPin, "sheet-pin", SHEET_PIN),
];

#[test]
fn kicad_agrees_where_its_own_check_reaches() {
    // The oracle, and a correction to the rule's own catalogue entry.
    //
    // `research/style-rules.md` §4 and this task's entry both say KiCad's
    // `endpoint_off_grid` covers "wire endpoints only". **Measured against
    // KiCad 10.0.5 it covers symbol pins as well**, and KiCad's own message
    // says so: "Symbol pin or wire end off connection grid". The overlap this
    // rule has to avoid double-counting is therefore TWO classes, not one.
    //
    // The four classes KiCad does not reach are this rule's own ground, and
    // the table below is what says which is which. KiCad is an independent
    // instrument: nothing in this comparison is computed by kicli.
    //
    // Environment-gated, so it does not count toward this lane being done.
    let Some(kicad) = kicli_probe::Kicad::found_or_skip("ask KiCad about the grid") else {
        return;
    };

    // The control the whole comparison stands on: on the clean drawing KiCad
    // reports no off-grid violation at all, so a "present" below is the
    // displacement's doing.
    let clean = drawing_with("oracle-clean", Class::Nothing);
    assert_eq!(reported(&clean), Vec::new());
    assert!(
        !kicad
            .rule_check(&clean)
            .violation_kinds()
            .contains(ERC_OFF_GRID),
        "KiCad sees nothing off the grid on the drawing this rule passes"
    );

    for (class, name, at) in CLASSES {
        let path = drawing_with(&format!("oracle-{name}"), class);
        assert_eq!(reported(&path), vec![(GRID, at)], "{name}");

        let seen = kicad
            .rule_check(&path)
            .violation_kinds()
            .contains(ERC_OFF_GRID);
        let overlaps = matches!(class, Class::Pin | Class::WireEnd);
        assert_eq!(
            seen, overlaps,
            "{name}: KiCad's own off-grid check reaches this class or it does not"
        );
    }

    // And where the two do overlap, they name the same point. The coordinate
    // is read out of KiCad's report text, not out of anything kicli computed.
    let pin = drawing_with("oracle-pin-position", Class::Pin);
    let report = kicad.rule_check(&pin);
    let found = findings(&pin);
    let position = format!("({} mm, {} mm)", found[0].pos.x, found[0].pos.y);
    assert!(
        report.text().contains(&position),
        "KiCad puts the off-grid pin at {position}:\n{}",
        report.text()
    );

    // The other half of the pin measurement, confirmed by the other
    // instrument: a placement anchor off the grid whose pins are on it is not
    // an off-grid condition for KiCad either.
    let anchor = drawing_with("oracle-anchor", Class::AnchorOnly);
    assert_eq!(reported(&anchor), Vec::new());
    assert!(
        !kicad
            .rule_check(&anchor)
            .violation_kinds()
            .contains(ERC_OFF_GRID),
        "KiCad reads pin positions rather than placement anchors too"
    );
}
