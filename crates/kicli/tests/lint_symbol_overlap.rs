//! `KI-OVL-001` reports two symbol bodies that share an area, and nothing else.
//!
//! # Where the expectations come from
//!
//! Every expectation here is derived from the **numbers this file writes into
//! the drawing** — a library rectangle and a placement anchor — and never from
//! kicli's own box computation. The derivation is one sentence of the file
//! format: a library rectangle is Y-up and the reader negates it
//! (`research/sch-format.md`; `model::library::named_point`), so the `SLAB`
//! symbol's body reaches `WIDE` to the **right** of its anchor and `TALL`
//! **above** it on screen.
//!
//! **That derivation is itself asserted, rather than assumed.** A single
//! "these two touch" fixture would pass just as happily if the body were
//! somewhere else entirely and the two boxes happened to miss. So the boundary
//! checks **sweep** the second anchor across the edge in one-internal-unit
//! steps and assert that the answer changes at exactly the anchor the
//! derivation names. A body box of a different size or in a different place
//! moves the transition and fails the sweep.
//!
//! # Why the boundary symbol draws no pins
//!
//! `ENGINEERING.md` permits hand-shaped geometry only where no drawable request
//! can distinguish the behaviour. These are drawable requests — every drawing
//! here is written by the probe harness and read back through kicli's own
//! reader — and the symbol is a graphic-only part, which KiCad libraries hold
//! (frames, logos, mechanical outlines). A pin would add its own segment to the
//! body box and put a length this file does not control between the rectangle
//! and the edge under test, which is the opposite of what a boundary check
//! needs. The realistic cases carry pins: see `power_symbols_are_not_exempt`.
//!
//! # What is NOT checked here, and where it is instead
//!
//! The `--allow` list is exercised in `src/lint/rules/overlap.rs`'s own `tests`
//! module and cannot be exercised here. A configured rule needs the rule's own
//! type, and the registration seam puts a rule file behind a private `mod` the
//! build script writes: `kicli::lint::registry` hands out `&'static dyn Rule`
//! and nothing else. Recorded in
//! `tasks/M5/phase2-ki-ovl-001-symbol-overlap.md` as a seam finding.

use kicli::lint::score::Density;
use kicli::lint::{Drawing, Engine, Finding, Gate, RuleId};
use kicli::model::Hierarchy;
use kicli_probe::drawing::{Placed, millimetres, rectangle, symbol};
use kicli_probe::Probe;
use std::path::{Path, PathBuf};

/// The rule under test.
const RULE: RuleId = RuleId("KI-OVL-001");

/// One internal unit, in millimetres. Four decimals is all KiCad writes, so
/// this is the smallest move a schematic can record.
const ONE_IU: f64 = 0.0001;

/// How far the `SLAB` body reaches to the right of its anchor.
const WIDE: f64 = 10.16;

/// How far the `SLAB` body reaches above its anchor on screen.
const TALL: f64 = 2.54;

/// The anchor every drawing here puts its first symbol on.
const BASE: (f64, f64) = (100.0, 100.0);

/// Where this binary writes its drawings.
fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("lint-overlap")
}

/// A graphic-only part: one rectangle, no pins.
///
/// The rectangle is written Y-up, as a library file holds it. Its two sides
/// differ, so a quarter turn is visible in the body box.
fn slab() -> String {
    symbol(
        "SLAB",
        "U",
        false,
        &[(
            "1_1",
            vec![rectangle(("0", "0"), (&millimetres(WIDE), &millimetres(TALL)))],
        )],
    )
}

/// The same body, declared as a power symbol, with the power pin it carries.
///
/// A power symbol is a net name in the shape of a part, so it has a pin. The
/// pin adds its own segment to the body box, which is why the checks that use
/// this symbol assert which symbols are named rather than how far they overlap.
fn power_slab() -> String {
    symbol(
        "PWRSLAB",
        "#PWR",
        true,
        &[(
            "1_1",
            vec![
                rectangle(("0", "0"), (&millimetres(WIDE), &millimetres(TALL))),
                kicli_probe::drawing::pin("power_in", ("0", "0"), "270", "1", ""),
            ],
        )],
    )
}

/// A box in internal units: the two corners, smaller first on each axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Box2 {
    left: i64,
    top: i64,
    right: i64,
    bottom: i64,
}

/// Millimetres as the whole number of internal units KiCad writes.
fn iu(millimetres: f64) -> i64 {
    (millimetres * 10_000.0).round() as i64
}

/// The `SLAB` body at an anchor, **as this file's own reading of the format
/// says it is drawn**, with or without the quarter turn.
///
/// Unturned, the body reaches `WIDE` right of the anchor and `TALL` above it.
/// Turned, KiCad's matrix sends `(x, y)` to `(y, -x)`
/// (`libs/kimath/src/transform.cpp`), which takes the two corners
/// `(0, -TALL)` and `(WIDE, 0)` to `(-TALL, 0)` and `(0, -WIDE)` — so the body
/// reaches `TALL` to the **left** of the anchor and `WIDE` above it.
///
/// Nothing in this function calls kicli. It is the control the rule is
/// measured against.
fn slab_body(anchor: (f64, f64), turned: bool) -> Box2 {
    let (x, y) = (iu(anchor.0), iu(anchor.1));
    if turned {
        Box2 {
            left: x - iu(TALL),
            top: y - iu(WIDE),
            right: x,
            bottom: y,
        }
    } else {
        Box2 {
            left: x,
            top: y - iu(TALL),
            right: x + iu(WIDE),
            bottom: y,
        }
    }
}

/// Do two boxes share an area of at least one internal unit on each axis?
///
/// The published rule, written once here in the test's own arithmetic.
fn share_an_area(one: Box2, two: Box2) -> bool {
    one.left.max(two.left) < one.right.min(two.right)
        && one.top.max(two.top) < one.bottom.min(two.bottom)
}

/// Is this point inside the box?
fn holds(box2: Box2, x: i64, y: i64) -> bool {
    (box2.left..=box2.right).contains(&x) && (box2.top..=box2.bottom).contains(&y)
}

/// Write a drawing holding the named `SLAB` placements, and return its root.
///
/// Each placement is a reference designator, an anchor and an angle. The probe
/// name must differ per check: the harness writes to a name-keyed path and two
/// checks of one binary run in parallel.
fn slabs(name: &str, placed: &[(&str, (f64, f64), &str)]) -> PathBuf {
    let mut probe = Probe::new(name, scratch());
    probe.define(slab());
    for (reference, anchor, angle) in placed {
        let (x, y) = (millimetres(anchor.0), millimetres(anchor.1));
        let mut placement = Placed::new("SLAB", reference, (&x, &y), &[]);
        placement.angle = angle;
        probe.place_symbol(&placement);
    }
    probe.write()
}

/// Every `KI-OVL-001` finding of a drawing on disk, in report order.
fn findings_of(root: &Path) -> Vec<Finding> {
    let hierarchy = Hierarchy::load(root).expect("the hierarchy loads");
    let drawings: Vec<Drawing<'_>> = hierarchy
        .placements
        .iter()
        .map(|placement| {
            let file = &hierarchy.files[placement.file];
            Drawing::read(&file.doc, &file.schematic, &placement.path)
        })
        .collect();
    Engine::of_every_rule()
        .examine_all(&drawings)
        .into_iter()
        .filter(|finding| finding.rule == RULE)
        .collect()
}

#[test]
fn a_shared_edge_does_not_fire_and_one_internal_unit_of_overlap_does() {
    // The second body's left edge meets the first body's right edge when its
    // anchor is here. Derived from BASE and WIDE, which this file writes.
    let meeting = BASE.0 + WIDE;
    let first = slab_body(BASE, false);

    let mut answers = Vec::new();
    for step in -2_i32..=2 {
        let anchor = (meeting + f64::from(step) * ONE_IU, BASE.1);
        let second = slab_body(anchor, false);

        // The control, in this file's own arithmetic: the two boxes share an
        // area below the meeting point and not at or above it.
        assert_eq!(
            share_an_area(first, second),
            step < 0,
            "the derivation says step {step} shares an area iff it is negative"
        );

        let name = format!("x-edge{step}");
        let findings = findings_of(&slabs(&name, &[("R1", BASE, "0"), ("R2", anchor, "0")]));
        answers.push((step, findings.len()));
        assert_eq!(
            findings.len(),
            usize::from(step < 0),
            "step {step} at anchor {anchor:?}: {:?}",
            findings.iter().map(|f| &f.message).collect::<Vec<_>>()
        );
    }
    // The sweep found both answers, so neither assertion above was vacuous.
    assert_eq!(
        answers,
        vec![(-2, 1), (-1, 1), (0, 0), (1, 0), (2, 0)],
        "the answer changes exactly where the two edges meet"
    );
}

#[test]
fn the_same_boundary_holds_on_the_other_axis() {
    // A rule strict on one axis only passes the check above and fails this.
    // The second body's top edge meets the first body's bottom edge when its
    // anchor is TALL below BASE.
    let meeting = BASE.1 + TALL;
    let first = slab_body(BASE, false);

    let mut answers = Vec::new();
    for step in -2_i32..=2 {
        let anchor = (BASE.0, meeting + f64::from(step) * ONE_IU);
        assert_eq!(
            share_an_area(first, slab_body(anchor, false)),
            step < 0,
            "the derivation says step {step} shares an area iff it is negative"
        );

        let name = format!("y-edge{step}");
        let findings = findings_of(&slabs(&name, &[("R1", BASE, "0"), ("R2", anchor, "0")]));
        answers.push((step, findings.len()));
        assert_eq!(findings.len(), usize::from(step < 0), "step {step}");
    }
    assert_eq!(answers, vec![(-2, 1), (-1, 1), (0, 0), (1, 0), (2, 0)]);
}

#[test]
fn a_corner_touch_is_not_an_overlap() {
    let anchor = (BASE.0 + WIDE, BASE.1 + TALL);
    assert!(
        !share_an_area(slab_body(BASE, false), slab_body(anchor, false)),
        "the derivation says a corner touch shares nothing"
    );
    let findings = findings_of(&slabs("corner", &[("R1", BASE, "0"), ("R2", anchor, "0")]));
    assert!(findings.is_empty(), "{findings:?}");

    // One unit in on both axes does share, so the check above is not vacuous.
    let inside = (anchor.0 - ONE_IU, anchor.1 - ONE_IU);
    assert!(share_an_area(
        slab_body(BASE, false),
        slab_body(inside, false)
    ));
    assert_eq!(
        findings_of(&slabs("corner-in", &[("R1", BASE, "0"), ("R2", inside, "0")])).len(),
        1
    );
}

#[test]
fn a_turned_body_is_compared_where_it_is_actually_drawn() {
    // Two anchors, each chosen so that the turned body and the unturned body
    // give OPPOSITE answers. A rule that measured an unrotated box would get
    // both of these exactly wrong, and no symmetric fixture can see that.
    let first = slab_body(BASE, false);

    for (name, anchor, turned_shares) in [
        ("turned-over", (101.6, 105.08), true),
        ("turned-clear", (99.06, 101.6), false),
    ] {
        let turned = slab_body(anchor, true);
        let unturned = slab_body(anchor, false);

        // The contrast, in this file's own arithmetic. If these two agreed the
        // check below would prove nothing about rotation.
        assert_eq!(
            share_an_area(first, turned),
            turned_shares,
            "{name}: the turned body"
        );
        assert_eq!(
            share_an_area(first, unturned),
            !turned_shares,
            "{name}: the unturned body gives the opposite answer"
        );

        let findings = findings_of(&slabs(
            name,
            &[("R1", BASE, "0"), ("R2", anchor, "90")],
        ));
        assert_eq!(
            findings.len(),
            usize::from(turned_shares),
            "{name}: the rule follows the turned body: {:?}",
            findings.iter().map(|f| &f.message).collect::<Vec<_>>()
        );
    }
}

#[test]
fn exactly_the_overlapping_pair_is_named() {
    // Three symbols in a row. R1 and R2 share one internal unit; R2 and R3
    // meet exactly; R1 and R3 are far apart. A fixture where every pair
    // overlapped would prove nothing about which pair is chosen, and a fixture
    // where the other pairs were merely distant would not test the boundary
    // and the pair walk at once.
    let r2 = (BASE.0 + WIDE - ONE_IU, BASE.1);
    let r3 = (r2.0 + WIDE, BASE.1);
    let (first, second, third) = (
        slab_body(BASE, false),
        slab_body(r2, false),
        slab_body(r3, false),
    );
    assert!(share_an_area(first, second), "R1 and R2 share an area");
    assert!(!share_an_area(second, third), "R2 and R3 only meet");
    assert!(!share_an_area(first, third), "R1 and R3 are apart");

    let findings = findings_of(&slabs(
        "three",
        &[("R1", BASE, "0"), ("R2", r2, "0"), ("R3", r3, "0")],
    ));
    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = &findings[0];
    assert!(
        found.message.contains("R1") && found.message.contains("R2"),
        "the finding names both symbols of the pair: {}",
        found.message
    );
    assert!(
        !found.message.contains("R3"),
        "and names no other: {}",
        found.message
    );
    assert_eq!(found.objects.len(), 2, "two symbols are named as objects");
    assert_ne!(found.objects[0], found.objects[1]);

    // The marker sits in the shared area rather than at the origin or at an
    // anchor. The shared area is one unit wide, so the first body holds it.
    assert!(
        holds(first, i64::from(found.pos.x.0), i64::from(found.pos.y.0)),
        "the finding points at the overlap: {}",
        found.pos
    );
}

#[test]
fn power_symbols_are_not_exempt() {
    // Two power symbols on top of each other, and nothing else on the sheet.
    let second = (BASE.0 + WIDE / 2.0, BASE.1);
    let mut probe = Probe::new("power", scratch());
    probe.define(power_slab());
    for (reference, anchor) in [("#PWR01", BASE), ("#PWR02", second)] {
        let (x, y) = (millimetres(anchor.0), millimetres(anchor.1));
        probe.place_symbol(&Placed::new("PWRSLAB", reference, (&x, &y), &["1"]));
    }
    let root = probe.write();

    let findings = findings_of(&root);
    assert_eq!(findings.len(), 1, "a power pair is reported: {findings:?}");
    assert!(
        findings[0].message.contains("#PWR01") && findings[0].message.contains("#PWR02"),
        "{}",
        findings[0].message
    );

    // The asymmetry this check exists to pin down: the scorer's own symbol
    // count EXCLUDES power symbols, so this sheet holds no symbols by that
    // measure and a finding all the same. A reader who later exempted power
    // symbols would be following the density count, and this is the line that
    // says not to.
    let hierarchy = Hierarchy::load(&root).expect("the hierarchy loads");
    let file = &hierarchy.files[hierarchy.placements[0].file];
    let drawing = Drawing::read(&file.doc, &file.schematic, &hierarchy.placements[0].path);
    assert_eq!(
        Density::of(&drawing).symbols(),
        0,
        "the density count excludes both symbols"
    );
    assert_eq!(
        drawing.schematic().symbols().count(),
        2,
        "and the sheet really holds two"
    );
}

#[test]
fn one_overlapping_pair_fails_the_gate_whatever_the_sheet_holds() {
    // The tier's whole mechanism, and the measurement behind this rule's
    // saturation declaration: the verdict does not depend on the share of the
    // sheet covered, so the declaration cannot change an outcome.
    let anchor = (BASE.0 + WIDE - ONE_IU, BASE.1);
    let findings = findings_of(&slabs("gate", &[("R1", BASE, "0"), ("R2", anchor, "0")]));
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].tier, kicli::lint::Tier::One);
    assert_eq!(findings[0].severity, kicli::lint::Severity::Error);
    assert_eq!(
        findings[0].penalty,
        kicli::lint::Penalty::ZERO,
        "a blocking rule does not move the score"
    );
    assert_eq!(
        findings[0].saturation,
        kicli::lint::Saturation::NEVER,
        "the declaration this rule makes, read back off a real finding"
    );

    // One finding fails the gate on a sheet of two symbols and on a sheet of
    // two thousand. The share is never consulted.
    for symbols in [2_u32, 2_000] {
        let gate = Gate::of(&findings, Density::of_counts(symbols, 0));
        assert!(!gate.passes(), "a sheet of {symbols} symbols fails");
        assert_eq!(gate.blockers().len(), 1);
        assert_eq!(gate.blockers()[0].word(), "blocking");
    }

    // And a clean sheet passes, so the gate is not failing on everything.
    let clean = findings_of(&slabs(
        "gate-clean",
        &[("R1", BASE, "0"), ("R2", (BASE.0 + WIDE, BASE.1), "0")],
    ));
    assert!(clean.is_empty());
    assert!(Gate::of(&clean, Density::of_counts(2, 0)).passes());
}
