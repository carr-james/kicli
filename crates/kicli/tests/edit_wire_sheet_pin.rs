//! Which edge a sheet pin's angle names, measured against the running tool.
//!
//! A **symbol** pin's angle points from its connection point towards the body,
//! so a wire leaves along the reverse of it. A **sheet** pin's angle means
//! something else: it names the edge of the sheet body the port sits on. That
//! rule was read out of KiCad's parser and, until this file, had never been put
//! to the tool.
//!
//! The instrument is connectivity, because connectivity is what a router breaks
//! if the rule is wrong. One drawing, four ports, one stub wire leaving each
//! port outwards to a resistor pin, and a hierarchical label in the child sheet
//! at the other end of each port. `kicad-cli sch export netlist` then says
//! which stubs reached their port.
//!
//! **The angle is the only variable across the first two arms.** Every port is
//! written at the same place in both, and every wire is drawn in the same place
//! in both. In the agreeing arm each angle names the edge the port is written
//! on; in the disagreeing arm each angle names the opposite edge. If the angle
//! named nothing that KiCad acts on, both arms would connect and this file
//! would measure nothing — which is exactly why the disagreeing arm is here.
//!
//! **Two arms are not enough to say what KiCad does with the angle**, and that
//! is why there is a third. Two readings predict the disagreeing arm's netlist
//! equally well: KiCad *moves* the port onto the edge the angle names, or KiCad
//! leaves the port where the file wrote it and merely accepts an approach along
//! the angle. Both leave the stub at the written position meeting nothing. The
//! third arm separates them — the same reflected angles, the same written port
//! positions, and each stub moved to where relocation says the port went:
//! opposite edge, along-edge coordinate kept. Relocation predicts all four
//! rejoin; the other reading predicts all four stay broken, because a port that
//! never moved is nowhere near the new stub.
//!
//! **The control is a symbol-pin net in the same drawing**, two resistor pins
//! joined by a plain wire, which must read as one net in every arm. A broken
//! port net beside a broken control is a broken instrument and not a finding.

use kicli::geometry::{GRID, Point};
use kicli::model::Hierarchy;
use kicli::route::terminal::{Heading, Terminal};
use kicli_probe::oracle::{Kicad, Partition, net, skipped};
use kicli_probe::{Port, Probe, millimetres};
use std::path::{Path, PathBuf};

/// The child sheet the ports lead into.
const CHILD: &str = "00000000-0000-4000-8000-cccccccccccc";

/// Where this binary writes the drawings it builds.
fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("sheet-pin-probes")
}

/// One edge of the sheet body, as the drawing writes it.
struct Edge {
    /// The port on it.
    port: &'static str,
    /// The angle the edge rule gives that edge.
    angle: &'static str,
    /// The angle that names the opposite edge.
    opposite: &'static str,
    /// Where the port is written, in millimetres.
    at: (&'static str, &'static str),
    /// Where the stub leaving the port ends, in millimetres.
    stub: (&'static str, &'static str),
    /// The resistor anchor that puts pin 1 on the end of the stub.
    anchor: (&'static str, &'static str),
    /// Where relocation says the port goes when it is given `opposite`: the
    /// far edge, with the along-edge coordinate kept.
    relocated: (&'static str, &'static str),
    /// Where a stub leaving the relocated port outwards ends.
    relocated_stub: (&'static str, &'static str),
    /// The resistor anchor for that stub, its pin 1 on the stub's end.
    relocated_anchor: (&'static str, &'static str),
    /// What the resistor on the parent side is called.
    parent: &'static str,
    /// What the resistor on the child side is called.
    child: &'static str,
    /// Which way a wire leaves the port, if the edge rule holds.
    escape: Heading,
}

/// The sheet body: `(101.6, 63.5)` to `(127, 88.9)`, a 25.4 mm square.
///
/// Every coordinate below is a whole number of 1.27 mm grid steps, and every
/// resistor sits clear of the body, so a stub that misses its port reaches
/// nothing else.
const EDGES: [Edge; 4] = [
    Edge {
        port: "RIGHT",
        angle: "0",
        opposite: "180",
        at: ("127", "71.12"),
        stub: ("137.16", "71.12"),
        anchor: ("137.16", "74.93"),
        relocated: ("101.6", "71.12"),
        relocated_stub: ("91.44", "71.12"),
        relocated_anchor: ("91.44", "74.93"),
        parent: "R1",
        child: "RC1",
        escape: Heading::PlusX,
    },
    Edge {
        port: "TOP",
        angle: "90",
        opposite: "270",
        at: ("111.76", "63.5"),
        stub: ("111.76", "53.34"),
        anchor: ("111.76", "57.15"),
        relocated: ("111.76", "88.9"),
        relocated_stub: ("111.76", "99.06"),
        relocated_anchor: ("111.76", "102.87"),
        parent: "R2",
        child: "RC2",
        escape: Heading::MinusY,
    },
    Edge {
        port: "LEFT",
        angle: "180",
        opposite: "0",
        at: ("101.6", "78.74"),
        stub: ("91.44", "78.74"),
        anchor: ("91.44", "82.55"),
        relocated: ("127", "78.74"),
        relocated_stub: ("137.16", "78.74"),
        relocated_anchor: ("137.16", "82.55"),
        parent: "R3",
        child: "RC3",
        escape: Heading::MinusX,
    },
    Edge {
        port: "BOTTOM",
        angle: "270",
        opposite: "90",
        at: ("116.84", "88.9"),
        stub: ("116.84", "99.06"),
        anchor: ("116.84", "102.87"),
        relocated: ("116.84", "63.5"),
        relocated_stub: ("116.84", "53.34"),
        relocated_anchor: ("116.84", "57.15"),
        parent: "R4",
        child: "RC4",
        escape: Heading::PlusY,
    },
];

/// Which of the three drawings to build.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Arm {
    /// Each angle names the edge its port is written on; stubs at the written
    /// positions.
    Agreeing,
    /// Each angle names the opposite edge; stubs unmoved, still at the written
    /// positions.
    Reflected,
    /// Each angle names the opposite edge, and each stub is drawn where
    /// relocation says the port went.
    Followed,
}

/// Build the drawing this arm asks for.
///
/// The port positions written into the file are the same in all three arms.
/// The angle changes between the first and the other two, and the stubs move
/// only in the third — so the first pair measures the angle and the second
/// pair measures where the angle put the port.
fn drawing(name: &str, arm: Arm) -> (Partition, PathBuf) {
    let mut probe = Probe::new(name, scratch());
    let mut child = Probe::child_of(&probe);

    let ports: Vec<Port<'_>> = EDGES
        .iter()
        .map(|edge| Port {
            name: edge.port,
            at: edge.at,
            angle: if arm == Arm::Agreeing {
                edge.angle
            } else {
                edge.opposite
            },
        })
        .collect();
    probe.sheet_of_size(CHILD, "child", ("101.6", "63.5"), ("25.4", "25.4"), &ports);

    for edge in &EDGES {
        // The stub leaves the port outwards and ends on a resistor pin. In the
        // followed arm it leaves the place relocation predicts instead, and
        // touches the written position nowhere along its length.
        let (from, to, anchor) = if arm == Arm::Followed {
            (edge.relocated, edge.relocated_stub, edge.relocated_anchor)
        } else {
            (edge.at, edge.stub, edge.anchor)
        };
        probe.wire(from, to);
        probe.place("R", edge.parent, anchor, &["1", "2"]);
    }
    // The control: two symbol pins joined by a wire, well away from the sheet.
    // Its behaviour is established, so it says whether the instrument works.
    probe.wire(("25.4", "127"), ("50.8", "127"));
    probe.place("R", "R5", ("25.4", "130.81"), &["1", "2"]);
    probe.place("R", "R6", ("50.8", "130.81"), &["1", "2"]);

    for (index, edge) in EDGES.iter().enumerate() {
        let wire_y = millimetres(25.4 + 12.7 * index as f64);
        let anchor_y = millimetres(29.21 + 12.7 * index as f64);
        child.strand_of_kind(
            "hierarchical_label",
            "(shape bidirectional)",
            edge.child,
            &wire_y,
            &anchor_y,
            edge.port,
        );
    }

    let path = probe.write_all(&[&child]);
    let kicad = Kicad::found().expect("the tool was asked for");
    (kicad.netlist_beside(&path).partition(), path)
}

#[test]
fn a_sheet_pin_leaves_the_edge_kicad_puts_it_on() {
    if Kicad::found().is_none() {
        skipped("measure which edge a sheet pin's angle names");
        return;
    }

    let (agreeing, path) = drawing("sheet-pin-edges", Arm::Agreeing);
    let (disagreeing, _) = drawing("sheet-pin-edges-reflected", Arm::Reflected);
    let (followed, _) = drawing("sheet-pin-edges-followed", Arm::Followed);

    // The control first, in every arm. Everything below stands on the
    // instrument working, and a port net that reads as broken beside a broken
    // control is not a finding.
    let control = net(&["R5.1", "R6.1"]);
    assert!(
        agreeing.contains(&control),
        "the control net is not joined, so the instrument is broken: {agreeing:?}"
    );
    assert!(
        disagreeing.contains(&control),
        "the control net is not joined in the reflected arm: {disagreeing:?}"
    );
    assert!(
        followed.contains(&control),
        "the control net is not joined in the followed arm: {followed:?}"
    );

    for edge in &EDGES {
        let (parent, child) = (format!("{}.1", edge.parent), format!("{}.1", edge.child));
        let joined = net(&[&parent, &child]);
        // A port written on the edge its angle names stays there, so the stub
        // drawn to meet it connects through to the child sheet.
        assert!(
            agreeing.contains(&joined),
            "the {} port did not carry its stub into the child sheet: {agreeing:?}",
            edge.port
        );
        // The same port, the same position, the same wire, the opposite angle:
        // the stub is left meeting nothing. On its own this says only that the
        // angle is load-bearing, not what the angle did with the port.
        assert!(
            !disagreeing.contains(&joined),
            "the {} port connects whatever its angle says, so this drawing \
             measures nothing about the angle: {disagreeing:?}",
            edge.port
        );
        // The same reflected angle again, the port still written where it was,
        // and the stub moved to the far edge. It connects — so KiCad moved the
        // port there. A port left where the file wrote it could not reach a
        // wire drawn 25.4 mm away across the body.
        assert!(
            followed.contains(&joined),
            "the {} port did not meet a stub drawn where its angle relocates \
             it, so the angle is not moving the port: {followed:?}",
            edge.port
        );
    }

    // The rule the router holds, read off the drawing that was just measured
    // rather than off a hand-built pin. A wire leaves the edge outwards, away
    // from the sheet body, which is the escape the drawn stubs took.
    let hierarchy = Hierarchy::load(&path).expect("the measured drawing loads");
    let pins: Vec<Terminal> = hierarchy.files[0]
        .schematic
        .sheets()
        .flat_map(|sheet| &sheet.pins)
        .map(Terminal::of_sheet_pin)
        .collect();
    assert_eq!(pins.len(), EDGES.len(), "every port was read back");
    for edge in &EDGES {
        let terminal = pins
            .iter()
            .find(|terminal| terminal.name == edge.port)
            .expect("the port was read back");
        assert_eq!(
            terminal.escape,
            Some(edge.escape),
            "the router leaves the {} port the wrong way",
            edge.port
        );
        // The stub the tool followed is the one the escape rule predicts.
        let stub = Point::new(millimetres_of(edge.stub.0), millimetres_of(edge.stub.1));
        let steps = (stub.x.0 - terminal.at.x.0 + stub.y.0 - terminal.at.y.0).abs() / GRID.0;
        assert_eq!(
            terminal.escape_point(GRID),
            edge.escape.step(terminal.at, GRID),
            "the escape point of the {} port",
            edge.port
        );
        assert_eq!(
            edge.escape
                .step(terminal.at, kicli::geometry::Iu(steps * GRID.0)),
            stub,
            "the {} stub does not run the way the escape rule says",
            edge.port
        );
    }
}

/// A millimetre reading of the drawing, in internal units.
fn millimetres_of(reading: &str) -> i32 {
    kicli::geometry::Iu::from_millimetres_text(reading)
        .expect("a coordinate is a number")
        .0
}
