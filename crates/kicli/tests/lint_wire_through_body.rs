//! `KI-WIRE-001` reports a wire drawn across a symbol body, and nothing else.
//!
//! # Where the expectations come from
//!
//! Not from `symbol_boxes`, and not from the rule. Every drawing here is built
//! by the probe harness from library coordinates this file writes, and every
//! expectation is worked out by hand from those coordinates and written down as
//! internal units. [`the_body_box_this_rule_reads_is_where_the_library_puts_it`]
//! is the calibration: it asserts the hand-derived box against the one the
//! geometry module computes, so a "no finding" check below cannot pass because
//! the symbol turned out to be somewhere else. That is also what keeps the
//! control off the rule's own ancestry — the independent side of the comparison
//! is arithmetic in this file's comments, not a second call into the linter.
//!
//! # The symbol, and why it is shaped like this
//!
//! A pin's connection point is on the body box boundary whenever it is the
//! outermost contributor along its own axis, and a wire running to such a pin
//! clips to a single point — so it is excluded by the **length** half of the
//! rule and the exclusion is never exercised. `INSET` therefore carries two
//! pins on its left: an outer one that sets the box edge and an **inset** one
//! 5.08 mm inside it. A wire ending on the inset pin clips to a part of real
//! length, so only the **proximity** half can suppress it. That is the case the
//! task entry calls worth more than every other check here, and a symbol with
//! symmetric pins cannot express it.
//!
//! ```text
//!              x: 86.36      91.44              109.22      (mm)
//!                 |             |                   |
//!   y 96.52  . . . +-------------------------------+   <- body box, top
//!                  |                               |
//!   y  99.06 ---A--|---->o pin 1 (inset)           |
//!   y 100.33 ---D--|-------------------------------|-----> crosses
//!   y 101.6        |                        pin 3 o|----C-->
//!   y 104.14 ---B->o pin 2 (on the edge)           |
//!   y 106.68 . . . +-------------------------------+   <- body box, bottom
//! ```

use kicli::geometry::{Point, Rect, symbol_boxes};
use kicli::lint::gate::{Blocker, Counted, Gate};
use kicli::lint::score::Density;
use kicli::lint::{Drawing, Engine, Finding, RuleId, Tier};
use kicli::model::items::{Item, Line, LineKind, Uuid};
use kicli::model::{Hierarchy, LoadedFile};
use kicli_probe::oracle::Kicad;
use kicli_probe::{Probe, pin, rectangle, symbol};
use std::path::{Path, PathBuf};

/// The rule under test.
const RULE: RuleId = RuleId("KI-WIRE-001");

/// Where the drawings this binary builds are written.
fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("lint-wire-body")
}

/// Where `INSET` is placed, in millimetres.
const PLACED_AT: (&str, &str) = ("101.6", "101.6");

/// The pin numbers `INSET` draws.
const PINS: [&str; 3] = ["1", "2", "3"];

/// A symbol whose left-hand pins sit at two different insets.
///
/// Library coordinates, y upwards, as a `.kicad_sym` writes them. Every pin is
/// 2.54 mm long, which is what `kicli_probe::pin` draws.
///
/// - the body is a square 10.16 mm across, centred on the anchor;
/// - pin 2 at `-15.24` is the outermost thing on the left, so it sets the box
///   edge and its own connection point lies **on** that edge;
/// - pin 1 at `-10.16` is 5.08 mm inside the edge pin 2 set, so its connection
///   point is **inside** the box;
/// - pin 3 at `7.62` sets the right-hand edge the same way pin 2 sets the left.
fn inset() -> String {
    symbol(
        "INSET",
        "U",
        false,
        &[(
            "1_1",
            vec![
                rectangle(("-5.08", "5.08"), ("5.08", "-5.08")),
                pin("passive", ("-10.16", "2.54"), "0", "1", "IN"),
                pin("passive", ("-15.24", "-2.54"), "0", "2", "EDGE"),
                pin("passive", ("7.62", "0"), "180", "3", "OUT"),
            ],
        )],
    )
}

/// The body box of `INSET` placed at [`PLACED_AT`], worked out by hand.
///
/// The library's y runs upwards and the schematic's downwards, and the box is
/// symmetric about y, so the flip moves nothing on that axis. On x the box runs
/// from pin 2's connection point to pin 3's:
///
/// ```text
/// x: 101.6 - 15.24 = 86.36    to    101.6 + 7.62 = 109.22
/// y: 101.6 -  5.08 = 96.52    to    101.6 + 5.08 = 106.68
/// ```
fn hand_derived_body() -> Rect {
    Rect::new(
        Point::new(863_600, 965_200),
        Point::new(1_092_200, 1_066_800),
    )
}

/// Pin 1, the inset one, in schematic internal units: `91.44,99.06`.
const INSET_PIN: Point = Point::new(914_400, 990_600);

/// Pin 2, the one on the box edge: `86.36,104.14`.
const EDGE_PIN: Point = Point::new(863_600, 1_041_400);

/// Pin 3, the one on the far edge: `109.22,101.6`.
const FAR_PIN: Point = Point::new(1_092_200, 1_016_000);

/// Place `INSET` as `U1`, and draw the three wires that legitimately meet it.
///
/// - **A** ends on the inset pin, so it clips to 5.08 mm of real length and only
///   the proximity half of the rule can excuse it;
/// - **B** ends on the edge pin, so it clips to a single point;
/// - **C** leaves the far pin outwards, so it clips to a single point too.
fn correctly_connected(probe: &mut Probe) {
    probe.define(inset());
    probe.place("INSET", "U1", PLACED_AT, &PINS);
    probe.wire(("76.2", "99.06"), ("91.44", "99.06"));
    probe.wire(("76.2", "104.14"), ("86.36", "104.14"));
    probe.wire(("109.22", "101.6"), ("127", "101.6"));
}

/// Every `KI-WIRE-001` finding of a drawing on disk, in report order.
fn findings_of(path: &Path) -> Vec<Finding> {
    let hierarchy = loaded(path);
    let drawings = drawings(&hierarchy);
    Engine::of_every_rule()
        .examine_all(&drawings)
        .into_iter()
        .filter(|finding| finding.rule == RULE)
        .collect()
}

fn loaded(path: &Path) -> Hierarchy {
    Hierarchy::load(path).expect("the probe drawing loads")
}

fn drawings(hierarchy: &Hierarchy) -> Vec<Drawing<'_>> {
    hierarchy
        .placements
        .iter()
        .map(|placement| {
            let file: &LoadedFile = &hierarchy.files[placement.file];
            Drawing::read(&file.doc, &file.schematic, &placement.path)
        })
        .collect()
}

/// The bus segments of the root sheet, in file order.
fn buses(hierarchy: &Hierarchy) -> Vec<Line> {
    lines(hierarchy, LineKind::Bus)
}

/// The wire segments of the root sheet, in file order.
fn wires(hierarchy: &Hierarchy) -> Vec<Line> {
    lines(hierarchy, LineKind::Wire)
}

/// The line segments of one kind on the root sheet, in file order.
fn lines(hierarchy: &Hierarchy, kind: LineKind) -> Vec<Line> {
    hierarchy.files[hierarchy.placements[0].file]
        .schematic
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Line(line) if line.kind == kind => Some(line.clone()),
            _ => None,
        })
        .collect()
}

/// The identifier of the wire drawn between these two points.
///
/// Taken from the loaded drawing rather than from the finding, so the two sides
/// of the comparison below come from different places.
fn wire_between(hierarchy: &Hierarchy, from: Point, to: Point) -> Uuid {
    wires(hierarchy)
        .into_iter()
        .find(|wire| (wire.from, wire.to) == (from, to))
        .map(|wire| wire.uuid)
        .expect("the drawing holds a wire between those points")
}

/// The identifier of the one symbol of the root sheet.
fn only_symbol(hierarchy: &Hierarchy) -> Uuid {
    let mut found: Vec<Uuid> = hierarchy.files[hierarchy.placements[0].file]
        .schematic
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Symbol(symbol) => Some(symbol.uuid.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(found.len(), 1, "the drawing holds one symbol");
    found.pop().expect("the drawing holds one symbol")
}

/// The identifier of the symbol with this reference designator.
fn symbol_called(hierarchy: &Hierarchy, reference: &str) -> Uuid {
    let path = &hierarchy.placements[0].path;
    hierarchy.files[hierarchy.placements[0].file]
        .schematic
        .symbols()
        .find(|symbol| symbol.reference_on(path).is_some_and(|r| r.0 == reference))
        .map(|symbol| symbol.uuid.clone())
        .expect("the drawing holds that symbol")
}

#[test]
fn the_body_box_this_rule_reads_is_where_the_library_puts_it() {
    // The calibration, and the anti-vacuity control for every check below that
    // asserts an absence. The hand-derived box in this file's comments is one
    // side; the geometry module is the other. The three pin positions are
    // asserted the same way, because the exclusion is measured against them.
    let mut probe = Probe::new("calibration", scratch());
    correctly_connected(&mut probe);
    let hierarchy = loaded(&probe.write());
    let file = &hierarchy.files[hierarchy.placements[0].file];
    let path = &hierarchy.placements[0].path;
    let drawing = Drawing::read(&file.doc, &file.schematic, path);

    let symbol = file
        .schematic
        .symbols()
        .next()
        .expect("the drawing holds a symbol");
    let definition = drawing
        .definition_of(symbol)
        .expect("the file embeds the definition");
    let drawn = symbol.drawn_on(path);

    assert_eq!(
        symbol_boxes(&file.doc, &drawn, definition).body,
        hand_derived_body(),
        "the body box is where the library coordinates put it"
    );

    let pins: Vec<Point> = kicli::geometry::resolve_pins(&drawn, definition)
        .into_iter()
        .map(|pin| pin.position)
        .collect();
    assert_eq!(
        pins,
        vec![INSET_PIN, EDGE_PIN, FAR_PIN],
        "the pins, in order"
    );

    // The whole point of the symbol: the inset pin is strictly inside the box,
    // so a wire running to it has real length inside the box to clip.
    let body = hand_derived_body();
    assert!(body.contains(INSET_PIN));
    assert!(INSET_PIN.x > body.start().x && INSET_PIN.x < body.end().x);
    // And the other two are exactly on the edge, which is the contrast.
    assert_eq!(EDGE_PIN.x, body.start().x);
    assert_eq!(FAR_PIN.x, body.end().x);
}

#[test]
fn a_wire_that_terminates_on_a_pin_is_not_a_crossing() {
    // Goal state 1. Without the exclusion, wire A fires: it ends on the inset
    // pin, so its clipped part is 5.08 mm long and the length half of the rule
    // cannot excuse it. If this check does not exist, the rule is not
    // implemented.
    let mut probe = Probe::new("connected", scratch());
    correctly_connected(&mut probe);
    let path = probe.write();

    // The instrument read something: the drawing it was handed is the one
    // meant, and the rule it is asking about is registered.
    let hierarchy = loaded(&path);
    let drawings = drawings(&hierarchy);
    assert_eq!(drawings.len(), 1, "one placement");
    assert_eq!(
        Density::of(&drawings[0]),
        Density::of_counts(1, 3),
        "one symbol and three wires"
    );
    assert!(
        Engine::of_every_rule().codes().contains(&RULE),
        "the rule is registered, so an empty answer is the rule's answer"
    );

    assert_eq!(
        findings_of(&path),
        Vec::new(),
        "a wire that meets a pin of this symbol is a connection, not a crossing"
    );
}

#[test]
fn a_wire_drawn_through_a_body_names_that_wire_and_that_symbol() {
    // Four wires, three of them legitimate. One finding is not enough to
    // assert: the rule could have fired on the wrong one. So the wire and the
    // symbol are both named, from the drawing rather than from the finding.
    //
    // The bus is the fourth gap of the module header, made observable. It is
    // drawn clean through the body at 97.79 mm, away from every pin, so a rule
    // that read buses would report it and the count below would be two.
    let mut probe = Probe::new("crossing", scratch());
    correctly_connected(&mut probe);
    probe.wire(("76.2", "100.33"), ("127", "100.33"));
    probe.bus(("76.2", "97.79"), ("127", "97.79"));
    let path = probe.write();

    let hierarchy = loaded(&path);
    let crossing = wire_between(
        &hierarchy,
        Point::new(762_000, 1_003_300),
        Point::new(1_270_000, 1_003_300),
    );
    let symbol = only_symbol(&hierarchy);
    assert_eq!(wires(&hierarchy).len(), 4, "three legitimate wires and one");
    assert_eq!(
        buses(&hierarchy).len(),
        1,
        "and a bus across the body, which this rule does not read"
    );

    let found = findings_of(&path);
    assert_eq!(found.len(), 1, "exactly the crossing wire: {found:?}");
    assert_eq!(
        found[0].objects,
        vec![crossing, symbol],
        "the finding names the wire that crosses and the symbol it crosses"
    );
    assert_eq!(found[0].tier, Tier::One);
    assert!(
        found[0].message.contains("U1"),
        "the message names the symbol: {}",
        found[0].message
    );
    assert!(
        !found[0].message.contains("other"),
        "it crosses one body, so nothing is said about others: {}",
        found[0].message
    );
    // The centre of the hand-derived box: 86.36 + 228.6/10 / 2 = 97.79 mm.
    assert_eq!(found[0].pos, Point::new(977_900, 1_016_000));
    assert_eq!(found[0].fix, None, "the repair is a route, not a command");
}

#[test]
fn a_wire_that_touches_the_body_at_one_point_only_is_not_a_crossing() {
    // The recorded decision, as a check. The grazing wire passes exactly
    // through the box corner at 109.22,106.68 and nowhere near a pin — the
    // nearest is pin 3, 5.08 mm away — so the LENGTH half is the only thing
    // that can suppress it. Wire B is the same arithmetic at a pin.
    let mut probe = Probe::new("grazing", scratch());
    probe.define(inset());
    probe.place("INSET", "U1", PLACED_AT, &PINS);
    probe.wire(("76.2", "104.14"), ("86.36", "104.14"));
    probe.wire(("119.38", "96.52"), ("99.06", "116.84"));
    let path = probe.write();

    let hierarchy = loaded(&path);
    assert_eq!(wires(&hierarchy).len(), 2, "the two tangent wires");
    assert_eq!(
        findings_of(&path),
        Vec::new(),
        "a clipped part of no length is not a crossing"
    );
}

#[test]
fn the_gate_fails_on_the_first_crossing_whatever_share_was_declared() {
    // The saturation declaration is true and changes no outcome while the rule
    // is Tier 1, and both halves of that are asserted here rather than
    // believed. Two drawings: one where a quarter of the wires cross, well
    // under the declared half, and one where every wire does.
    let mut few = Probe::new("gate-one-of-four", scratch());
    correctly_connected(&mut few);
    few.wire(("76.2", "100.33"), ("127", "100.33"));

    let mut every = Probe::new("gate-all", scratch());
    every.define(inset());
    every.place("INSET", "U1", PLACED_AT, &PINS);
    every.wire(("76.2", "100.33"), ("127", "100.33"));
    every.wire(("76.2", "102.87"), ("127", "102.87"));

    for (probe, crossings, total) in [(&few, 1, 4), (&every, 2, 2)] {
        let path = probe.write();
        let hierarchy = loaded(&path);
        let sheets = drawings(&hierarchy);
        let density = Density::of(&sheets[0]);
        assert_eq!(density.wires(), total, "the wire count of {}", probe.name());

        let found = findings_of(&path);
        assert_eq!(found.len(), crossings, "the crossings of {}", probe.name());

        // What the rule declared, read off the finding the gate reads it off.
        let saturation = found[0].saturation;
        assert_eq!(saturation.counts(), Counted::Wires, "it counts wires");
        assert_eq!(saturation.share(), (1, 2), "at the standard share");
        assert_eq!(
            saturation.is_reached(u32::try_from(crossings).expect("a small count"), density),
            crossings * 2 >= usize::try_from(total).expect("a small count"),
            "the share is reached exactly when the arithmetic says so"
        );

        // And the verdict is the same either way, because Tier 1 blocks first.
        let gate = Gate::of(&found, density);
        assert_eq!(
            gate.blockers(),
            [Blocker::Blocking {
                rule: RULE,
                count: u32::try_from(crossings).expect("a small count"),
            }],
            "a blocking rule fails on its first finding, not on its share"
        );
    }
}

#[test]
fn kicad_puts_the_pins_where_the_exclusion_expects_them() {
    // The oracle, and it is the one that matters here. The exclusion is
    // measured against pin positions, and every other check in this binary
    // takes those positions from kicli — through the same geometry module the
    // rule itself calls. This check asks KiCad where the pins are and compares
    // with the HAND-DERIVED constants above, so the independent side of the
    // comparison is the library coordinates this file wrote and KiCad's own
    // reading of them, with nothing of kicli's on either side.
    //
    // The symbol is placed with nothing attached, so KiCad raises an
    // unconnected-pin violation for each pin and the report names it.
    let Some(tool) = Kicad::found_or_skip("ask KiCad where the pins are") else {
        return;
    };
    let mut probe = Probe::new("pin-oracle", scratch());
    probe.define(inset());
    probe.place("INSET", "U1", PLACED_AT, &PINS);
    let path = probe.write();

    let report = tool.rule_check(&path);
    let mut reported = report.pins_of("U1");
    reported.sort();
    assert_eq!(
        reported,
        vec![
            ("1".to_owned(), INSET_PIN),
            ("2".to_owned(), EDGE_PIN),
            ("3".to_owned(), FAR_PIN),
        ],
        "KiCad reports the pins where this file's arithmetic puts them:\n{}",
        report.text()
    );
}

#[test]
fn text_outside_the_body_does_not_make_a_crossing() {
    // "body, not full", as behaviour rather than as a position. Swapping
    // `.body` for `.full` in the rule is otherwise caught only by the
    // finding's reported position, which is a weak reason to prefer one box —
    // measured, and this check is the repair.
    //
    // The two boxes differ by a knowable amount in this drawing: the probe
    // writes every visible field at an ABSOLUTE `(at 0 0)`, and the full box
    // is the body unioned with every visible field's own box, so the full box
    // reaches the page origin and the body box does not. A wire at 50.8 mm is
    // therefore well inside the full box and nowhere near the body.
    let mut probe = Probe::new("text-not-body", scratch());
    probe.define(inset());
    probe.place("INSET", "U1", PLACED_AT, &PINS);
    probe.wire(("38.1", "50.8"), ("63.5", "50.8"));
    let path = probe.write();

    let hierarchy = loaded(&path);
    let file = &hierarchy.files[hierarchy.placements[0].file];
    let sheet = &hierarchy.placements[0].path;
    let drawing = Drawing::read(&file.doc, &file.schematic, sheet);
    let symbol = file
        .schematic
        .symbols()
        .next()
        .expect("the drawing holds a symbol");
    let definition = drawing
        .definition_of(symbol)
        .expect("the file embeds the definition");
    let boxes = symbol_boxes(&file.doc, &symbol.drawn_on(sheet), definition);

    // The control: the two boxes really do differ here, and the wire really is
    // inside one and outside the other. Without this the check could pass on a
    // drawing where no box distinction existed to get wrong.
    let wire = (Point::new(381_000, 508_000), Point::new(635_000, 508_000));
    assert_ne!(
        boxes.full, boxes.body,
        "the symbol draws text outside its body"
    );
    assert!(boxes.full.contains(wire.0) && boxes.full.contains(wire.1));
    assert!(!boxes.body.contains(wire.0) && !boxes.body.contains(wire.1));

    assert_eq!(
        findings_of(&path),
        Vec::new(),
        "a wire across a symbol's text is not a wire across its body"
    );
}

#[test]
fn a_wire_across_two_bodies_is_one_finding_that_counts_the_others() {
    // The per-wire decision, as a check. The repair is one re-route, so a wire
    // drawn across two symbols is one finding — naming the first in file order
    // and saying how many others it crossed.
    let mut probe = Probe::new("two-bodies", scratch());
    probe.define(inset());
    probe.place("INSET", "U1", PLACED_AT, &PINS);
    probe.place("INSET", "U2", ("152.4", "101.6"), &PINS);
    probe.wire(("76.2", "100.33"), ("177.8", "100.33"));
    let path = probe.write();

    let hierarchy = loaded(&path);
    let crossing = wire_between(
        &hierarchy,
        Point::new(762_000, 1_003_300),
        Point::new(1_778_000, 1_003_300),
    );
    let found = findings_of(&path);
    assert_eq!(found.len(), 1, "one wire, one finding: {found:?}");
    assert_eq!(
        found[0].objects,
        vec![crossing, symbol_called(&hierarchy, "U1")],
        "it names the first body in file order"
    );
    assert_eq!(
        found[0].message, "this wire crosses the body of U1 and 1 other",
        "and says how many others without listing them"
    );
    // U1's body centre, not U2's: 86.36 + (109.22 - 86.36) / 2 = 97.79 mm.
    assert_eq!(found[0].pos, Point::new(977_900, 1_016_000));
}
