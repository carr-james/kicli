//! `KI-CONN-001` reports a pin that touches a wire it is not connected to.
//!
//! # Where each side of every comparison comes from
//!
//! The rule's own two sides are two different questions put to one partition:
//! the net the extractor lists the pin on, and the net of a pin sitting at an
//! **end** of the wire. They are not one answer compared with itself, and a
//! break that collapsed or shattered the partition moves them apart rather than
//! together.
//!
//! The **checks'** two sides are the harder half, and `opening-1` predicted the
//! trap before it fired: *"a check asserting that the reported net equals the
//! extractor's net is a degenerate-equality candidate."* So no expectation here
//! is taken from kicli's extractor.
//!
//! - On the committed fixture, the expectation is derived from
//!   `sch/nets/nets.netlist` — the bytes `kicad-cli sch export netlist` wrote,
//!   carried in the fixture manifest with `kicad-cli` provenance. KiCad's
//!   partition says which pins share a net; the rule never contributes to the
//!   expectation it is measured against. Only the *positions* come from kicli,
//!   through its reader, which is a different subsystem from the union-find and
//!   the one the trap does not concern.
//! - On a probe, the expectation is the drawing's own construction, and
//!   [`Probe::partition`] asserts kicli's partition against `kicad-cli`'s on the
//!   same file whenever the tool is installed. A probe whose connectivity KiCad
//!   disagrees with panics rather than reporting.

use kicli::connectivity::extract;
use kicli::geometry::{Point, on_segment, resolve_pins};
use kicli::lint::{Drawing, Engine, Finding, RuleId};
use kicli::model::Hierarchy;
use kicli::model::items::{Item, LineKind};
use kicli_probe::drawing::{LabelKind, Probe, millimetres, power};
use kicli_probe::oracle::Netlist;
use kicli_probe::scratch::Fixtures;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The rule under test.
const RULE: RuleId = RuleId("KI-CONN-001");

/// One internal unit, in the millimetres a file is written in.
///
/// The exclusion the rule states is one internal unit, and this is what one
/// internal unit is: `geometry::UNITS_PER_MM` is ten thousand, and KiCad writes
/// four decimals, so a step of exactly this size is the smallest move a
/// schematic can record.
const ONE_IU_MM: f64 = 0.0001;

/// The committed fixtures this binary reads, and the scratch it writes in.
fn fixtures() -> Fixtures {
    Fixtures::new(env!("CARGO_TARGET_TMPDIR"), env!("CARGO_MANIFEST_DIR"))
}

/// The connectivity fixture, the one that carries a cluster of each kind.
fn nets_fixture(name: &str) -> PathBuf {
    fixtures().fixture("sch/nets").join(name)
}

/// Every `KI-CONN-001` finding of a project on disk, in report order.
///
/// The drawings are built the way a scoring command will build them: one per
/// placement, each with the project's partition attached.
fn findings_of(root: &Path) -> Vec<Finding> {
    let hierarchy = Hierarchy::load(root).expect("the hierarchy loads");
    let nets = extract(&hierarchy);
    let drawings: Vec<Drawing<'_>> = hierarchy
        .placements
        .iter()
        .map(|placement| {
            let file = &hierarchy.files[placement.file];
            Drawing::read(&file.doc, &file.schematic, &placement.path).with_nets(&nets)
        })
        .collect();
    Engine::of_every_rule()
        .examine_all(&drawings)
        .into_iter()
        .filter(|finding| finding.rule == RULE)
        .collect()
}

/// The pins the rule reported, as a netlist writes them, such as `R11.1`.
fn reported_pins(findings: &[Finding]) -> BTreeSet<String> {
    findings
        .iter()
        .map(|finding| {
            finding
                .message
                .split_whitespace()
                .nth(1)
                .unwrap_or_default()
                .to_owned()
        })
        .collect()
}

/// Where every listed pin of a project sits, and every wire segment it holds.
///
/// This is the reader's answer and never the extractor's: it resolves pin
/// positions and reads wire ends, and asks nothing about nets.
struct Geometry {
    pins: Vec<(String, Point)>,
    wires: Vec<(Point, Point)>,
}

fn geometry_of(root: &Path) -> Geometry {
    let hierarchy = Hierarchy::load(root).expect("the hierarchy loads");
    let mut pins = Vec::new();
    let mut wires = Vec::new();
    for placement in &hierarchy.placements {
        let file = &hierarchy.files[placement.file];
        let drawing = Drawing::read(&file.doc, &file.schematic, &placement.path);
        for item in &file.schematic.items {
            match item {
                Item::Line(line) if matches!(line.kind, LineKind::Wire) => {
                    wires.push((line.from, line.to));
                }
                Item::Symbol(symbol) => {
                    let (Some(definition), Some(reference)) = (
                        drawing.definition_of(symbol),
                        symbol.reference_on(&placement.path),
                    ) else {
                        continue;
                    };
                    for pin in resolve_pins(&symbol.drawn_on(&placement.path), definition) {
                        pins.push((format!("{}.{}", reference.0, pin.number), pin.position));
                    }
                }
                _ => {}
            }
        }
    }
    Geometry { pins, wires }
}

/// The net KiCad put a pin on, by the pin's netlist label.
fn kicad_net_of<'a>(netlist: &'a Netlist, pin: &str) -> Option<&'a str> {
    netlist
        .nets()
        .iter()
        .find(|net| net.pins.iter().any(|listed| listed == pin))
        .map(|net| net.name.as_str())
}

/// The pins `KI-CONN-001` must report, worked out from KiCad's own netlist.
///
/// For every pin on a wire's interior, KiCad's partition is asked twice: once
/// for that pin, once for a pin sitting at an end of the same wire — which is
/// on the wire's net by KiCad's own first merge rule, a wire's connection
/// points being its two ends. A finding is expected exactly when the two
/// answers differ. Nothing kicli computed enters this.
fn expected_by_kicad(root: &Path, netlist: &Netlist) -> BTreeSet<String> {
    let geometry = geometry_of(root);
    let mut expected = BTreeSet::new();
    for (label, at) in &geometry.pins {
        for (from, to) in &geometry.wires {
            if *at == *from || *at == *to || !on_segment(*from, *to, *at) {
                continue;
            }
            let Some(end) = geometry
                .pins
                .iter()
                .find(|(_, seat)| seat == from || seat == to)
            else {
                continue;
            };
            let (Some(mine), Some(theirs)) =
                (kicad_net_of(netlist, label), kicad_net_of(netlist, &end.0))
            else {
                continue;
            };
            if mine != theirs {
                expected.insert(label.clone());
            }
        }
    }
    expected
}

#[test]
fn the_committed_fixture_answers_both_directions_as_kicad_does() {
    let root = nets_fixture("nets.kicad_sch");
    let netlist = Netlist::parse(
        &std::fs::read_to_string(nets_fixture("nets.netlist")).expect("the oracle is readable"),
    );
    let expected = expected_by_kicad(&root, &netlist);

    // Two controls on the expectation itself, because a set that was empty, or
    // that held every pin, would make the comparison below say nothing.
    assert_eq!(
        expected,
        BTreeSet::from(["R11.1".to_owned()]),
        "KiCad's netlist says exactly one pin on this fixture sits on a wire \
         it is not on the net of"
    );
    assert!(
        geometry_of(&root).pins.len() > 10,
        "the fixture holds many pins, so naming one is a discrimination"
    );

    let findings = findings_of(&root);
    assert_eq!(reported_pins(&findings), expected);

    // The negative direction, named rather than implied: the junctioned
    // cluster is the same drawing but for the junction, and it must be silent.
    // R15 pin 1 sits mid-wire exactly as R11 pin 1 does.
    assert!(
        !reported_pins(&findings).contains("R15.1"),
        "the junctioned cluster is a connection and is not reported"
    );
}

#[test]
fn the_finding_names_the_pin_the_wire_and_the_junction_that_repairs_it() {
    let findings = findings_of(&nets_fixture("nets.kicad_sch"));
    let [finding] = findings.as_slice() else {
        panic!("one finding: {findings:?}");
    };
    assert_eq!(finding.pos, Point::new(381_000, 889_000));
    assert_eq!(finding.objects.len(), 2, "the pin and the wire");
    assert_eq!(
        finding.fix.as_deref(),
        Some("kicli junction add --at 38.1,88.9")
    );
    assert!(
        finding.message.starts_with("pin R11.1 touches this wire"),
        "{}",
        finding.message
    );
}

#[test]
fn a_drawing_with_no_partition_attached_cannot_answer_and_says_nothing() {
    // The recorded hazard, executable. A `Drawing` built without a partition
    // makes this rule silent, so a command that gates a build on connectivity
    // must attach one. It is here so that the silence is a measured property
    // rather than a surprise found by whoever builds `sch score`.
    let root = nets_fixture("nets.kicad_sch");
    let hierarchy = Hierarchy::load(&root).expect("the hierarchy loads");
    let drawings: Vec<Drawing<'_>> = hierarchy
        .placements
        .iter()
        .map(|placement| {
            let file = &hierarchy.files[placement.file];
            Drawing::read(&file.doc, &file.schematic, &placement.path)
        })
        .collect();
    let bare: Vec<Finding> = Engine::of_every_rule()
        .examine_all(&drawings)
        .into_iter()
        .filter(|finding| finding.rule == RULE)
        .collect();
    assert!(bare.is_empty());
    assert_eq!(findings_of(&root).len(), 1, "and the same drawing with one");
}

/// A wire from `25.4,y` to `50.8,y`, with a resistor pin on each end.
///
/// The resistor anchor sits one pin length below the wire, so pin 1 lands on
/// it. This is the shape of every cluster the research notes measured.
fn strand(probe: &mut Probe, y: f64, left: &str, right: &str) {
    let wire = millimetres(y);
    let anchor = millimetres(y + 3.81);
    probe.place("R", left, ("25.4", &anchor), &["1", "2"]);
    probe.place("R", right, ("50.8", &anchor), &["1", "2"]);
    probe.wire(("25.4", &wire), ("50.8", &wire));
}

/// A resistor whose pin 1 lands at `x,y`.
fn pin_at(probe: &mut Probe, reference: &str, x: f64, y: f64) {
    probe.place(
        "R",
        reference,
        (&millimetres(x), &millimetres(y + 3.81)),
        &["1", "2"],
    );
}

#[test]
fn the_endpoint_boundary_is_exact_from_both_sides() {
    // What the committed fixture cannot test: its mid-span pin sits at the
    // exact centre of the wire, so it says nothing about where the interior
    // begins. This probe puts a pin at the end and one internal unit inside it,
    // which is the whole of the false-positive direction.
    let directory = fixtures().scratch("conn-boundary");
    let y = 25.4;

    let mut at_the_end = Probe::new("conn-at-the-end", directory.clone());
    strand(&mut at_the_end, y, "R1", "R2");
    pin_at(&mut at_the_end, "R3", 25.4, y);
    assert!(
        !at_the_end.partition().is_empty(),
        "the probe drew a net, so KiCad has been asked about it"
    );
    assert!(
        findings_of(&at_the_end.write()).is_empty(),
        "a pin at a wire's end is an ordinary connection"
    );

    let mut one_inside = Probe::new("conn-one-iu-inside", directory);
    strand(&mut one_inside, y, "R1", "R2");
    pin_at(&mut one_inside, "R3", 25.4 + ONE_IU_MM, y);
    assert!(!one_inside.partition().is_empty());
    assert_eq!(
        reported_pins(&findings_of(&one_inside.write())),
        BTreeSet::from(["R3.1".to_owned()]),
        "one internal unit inside the end is the interior"
    );
}

#[test]
fn a_pin_sharing_a_mid_wire_anchor_with_a_label_is_reported() {
    // The goal state's claim: this case needs no code of its own, because the
    // pin's net is not the wire's net, which is already the test. Measured
    // rather than assumed — and the label is what makes it interesting, since a
    // label alone on a wire's interior *does* join the wire.
    let directory = fixtures().scratch("conn-label");
    let y = 25.4;
    let mut probe = Probe::new("conn-label-and-pin", directory);
    strand(&mut probe, y, "R1", "R2");
    pin_at(&mut probe, "R3", 38.1, y);
    probe.label_of_kind(LabelKind::Local, "WITHPIN", ("38.1", &millimetres(y)));

    assert!(!probe.partition().is_empty());
    assert_eq!(
        reported_pins(&findings_of(&probe.write())),
        BTreeSet::from(["R3.1".to_owned()]),
        "the label and the pin make a net of their own and leave the wire out"
    );
}

/// A ground symbol whose connection point is exactly at `x,y`.
///
/// The `Value` field is what a power symbol names its net with, and
/// [`Placed::new`] values a placement after its reference designator, so a
/// ground symbol placed the ordinary way would drive a net called `#PWR01`.
/// Measured: `kicad-cli sch export netlist` on the first draft of the probe
/// below wrote `(name "#PWR01")`, which is how the mistake was found.
fn ground(probe: &mut Probe, reference: &str, x: &str, y: &str) {
    probe.place_unit("GND", reference, (x, y), 1, "GND", &["1"]);
}

#[test]
fn a_mid_wire_pin_already_on_the_wires_net_by_name_is_not_reported() {
    // The case that proves this rule is the partition's answer and not a
    // junction test. Both clusters below have a pin on a wire's interior with
    // no junction under it. In the first the pin is a ground symbol and the
    // wire already carries the global `GND` net, so the drawing means exactly
    // what it looks like; in the second the pin is an ordinary resistor and it
    // does not.
    //
    // A rule written as "there is no junction at the point" reports both, and
    // this is the check that refuses it.
    let directory = fixtures().scratch("conn-named");
    let mut probe = Probe::new("conn-named-by-power", directory);
    probe.define(power("GND"));

    // The wire's own net is `GND`, because a ground symbol sits on its right
    // end. A second ground symbol mid-span is on that net by name.
    let joined = 25.4;
    let joined_wire = millimetres(joined);
    pin_at(&mut probe, "R1", 25.4, joined);
    probe.wire(("25.4", &joined_wire), ("50.8", &joined_wire));
    ground(&mut probe, "#PWR01", "50.8", &joined_wire);
    ground(&mut probe, "#PWR02", "38.1", &joined_wire);

    let split = 50.8;
    strand(&mut probe, split, "R4", "R5");
    pin_at(&mut probe, "R6", 38.1, split);

    assert!(!probe.partition().is_empty());
    assert_eq!(
        reported_pins(&findings_of(&probe.write())),
        BTreeSet::from(["R6.1".to_owned()]),
        "the ground pin is on the wire's net already; the resistor pin is not"
    );
}

#[test]
fn a_ground_symbol_on_a_locally_labelled_wire_joins_it_and_is_not_reported() {
    // The same negative direction as the check above, reached by a different
    // merge. There the wire's name came from a second ground symbol; here it
    // comes from a plain local label, and the two names meet as names rather
    // than as geometry.
    //
    // Measured rather than reasoned, and it took two readings to get right.
    // `kicad-cli sch export netlist` on this probe writes `(name "GND")`. The
    // first draft placed the ground symbol the ordinary way, which values it
    // after its reference designator, and KiCad wrote `(name "/GND")` with the
    // power pin on a net of its own — a different drawing that happened to look
    // like this one. The name in the netlist is what tells the two apart.
    let directory = fixtures().scratch("conn-local-gnd");
    let mut probe = Probe::new("conn-local-gnd", directory);
    probe.define(power("GND"));
    let y = 25.4;
    strand(&mut probe, y, "R1", "R2");
    probe.label_of_kind(LabelKind::Local, "GND", ("44.45", &millimetres(y)));
    ground(&mut probe, "#PWR01", "38.1", &millimetres(y));

    assert!(!probe.partition().is_empty());
    assert!(
        findings_of(&probe.write()).is_empty(),
        "the ground symbol is on the labelled wire's net, so nothing is wrong"
    );
}
