//! Deleting one wire segment, by the identifier a report writes.
//!
//! The command removes the record it was asked for and **nothing else**. It
//! does not cascade into junctions: a junction that now sits on two ends is
//! legal, and removing it is a second decision that belongs to whoever asks for
//! it. It **reports** every junction the removal left joining fewer than three
//! wire ends, so the caller can make that decision.
//!
//! Every drawing here is built by the probe harness. The harness numbers its
//! identifiers in their last twelve digits, so every object of a probe drawing
//! carries the same eight-character handle: a drawing that needs two handles
//! apart has one identifier rewritten in its text, and the check that does it
//! says so.
//!
//! The netlist oracle runs only with `KICLI_TEST_KICAD_CLI` set. Without it the
//! connectivity claim is kicli's own extractor, and with it KiCad is asked
//! about the same bytes.

use kicli::connectivity::extract;
use kicli::edit::wire::delete;
use kicli::geometry::{GRID, Iu, Point};
use kicli::model::{Hierarchy, LineKind, SheetPath, Target, WriteOptions};
use kicli_probe::Probe;
use kicli_probe::oracle::{Kicad, Partition, kicli_partition, net};
use std::path::{Path, PathBuf};

/// Where this binary writes the drawings it builds.
fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("wire-delete")
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

/// What KiCad says about a file, when the environment asked for the tool.
fn oracle(path: &Path) -> Option<Partition> {
    Kicad::found().map(|kicad| kicad.netlist_beside(path).partition())
}

/// The identifier of the one segment that runs between two points.
///
/// A caller addresses a segment by what a view printed, so the checks below
/// find theirs by looking at the drawing rather than by knowing what the
/// harness wrote.
fn segment_between(hierarchy: &Hierarchy, from: Point, to: Point) -> String {
    let found: Vec<String> = hierarchy.files[0]
        .schematic
        .lines()
        .filter(|line| (line.from == from && line.to == to) || (line.from == to && line.to == from))
        .map(|line| line.uuid.0.clone())
        .collect();
    assert_eq!(found.len(), 1, "one segment runs between those points");
    found[0].clone()
}

/// Every junction of a drawing, by where it sits.
fn junctions_at(hierarchy: &Hierarchy) -> Vec<(Point, String)> {
    hierarchy.files[0]
        .schematic
        .junctions()
        .map(|junction| (junction.at, junction.uuid.0.clone()))
        .collect()
}

/// The identifier of the one junction of a drawing, and where it sits.
fn only_junction(hierarchy: &Hierarchy) -> (String, Point) {
    let found = junctions_at(hierarchy);
    assert_eq!(found.len(), 1, "the drawing holds one junction");
    (found[0].1.clone(), found[0].0)
}

/// Two resistors 25.4 mm apart, pin 1 of each at `y = 50.8`.
fn two_resistors(name: &str) -> Probe {
    let mut probe = Probe::new(name, scratch());
    probe.place("R", "R1", ("50.8", "54.61"), &["1", "2"]);
    probe.place("R", "R2", ("76.2", "54.61"), &["1", "2"]);
    probe
}

/// How many `wire` records a file holds.
fn wire_records(text: &str) -> usize {
    text.matches("(wire").count()
}

#[test]
fn deleting_a_wire_splits_what_it_joined() {
    // R1.1 up, across, and down into R2.1: three records for one connection.
    // Taking the middle one out leaves a stub at each end and two nets where
    // there was one.
    let mut probe = two_resistors("splits-what-it-joined");
    probe.wire(("50.8", "50.8"), ("50.8", "45.72"));
    probe.wire(("50.8", "45.72"), ("76.2", "45.72"));
    probe.wire(("76.2", "45.72"), ("76.2", "50.8"));
    let path = probe.write();
    let project = path.parent().expect("the drawing sits in a directory");

    // The control: the instrument reports one net before anything is deleted.
    // Without it a check that saw two nets afterwards could not tell a split
    // from a drawing that was never joined.
    let joined = net(&["R1.1", "R2.1"]);
    let mut hierarchy = loaded(&path);
    assert!(
        kicli_partition(&extract(&hierarchy)).contains(&joined),
        "the drawing does not start with R1.1 joined to R2.1"
    );
    let asked_before = oracle(&path).is_some_and(|kicad| {
        assert!(
            kicad.contains(&joined),
            "KiCad does not start with R1.1 joined to R2.1: {kicad:?}"
        );
        true
    });

    let identifier = segment_between(&hierarchy, at("50.8", "45.72"), at("76.2", "45.72"));
    let sheet = hierarchy.placements[0].path.clone();
    let before = std::fs::read_to_string(&path).expect("the drawing reads");
    let deleted = delete(
        &mut hierarchy,
        &identifier,
        &target(&path, project, &sheet),
        "after",
    )
    .expect("the segment is deletable");

    assert_eq!(deleted.removed.0, identifier, "the report names what went");
    assert!(
        deleted.stranded.is_empty(),
        "the drawing holds no junction, so nothing was stranded: {:?}",
        deleted.stranded
    );
    assert!(
        deleted.mutation.invariants.passed(),
        "the invariants did not hold: {}",
        deleted.mutation.render()
    );

    // One record went, and only one.
    let after = std::fs::read_to_string(&path).expect("the written file reads");
    assert_eq!(
        wire_records(&before),
        3,
        "the drawing was written with three"
    );
    assert_eq!(wire_records(&after), 2, "exactly one record was removed");
    assert!(
        !after.contains(&identifier),
        "the removed segment is still in the file"
    );

    // Two nets where there was one, and each end keeps its stub.
    let reloaded = loaded(&path);
    let partition = kicli_partition(&extract(&reloaded));
    assert!(
        !partition.contains(&joined),
        "kicli still reports R1.1 joined to R2.1: {partition:?}"
    );
    assert!(
        partition.contains(&net(&["R1.1"])) && partition.contains(&net(&["R2.1"])),
        "kicli does not report the two nets the split made: {partition:?}"
    );

    // And KiCad says the same about the same bytes, when the tool was asked
    // for. The arm above ran on the same drawing before the delete, so a tool
    // that answered nothing either way cannot pass the pair.
    if let Some(kicad) = oracle(&path) {
        assert!(
            asked_before,
            "the tool answered after the delete but not before"
        );
        assert!(
            !kicad.contains(&joined),
            "KiCad still reports R1.1 joined to R2.1: {kicad:?}"
        );
        assert!(
            kicad.contains(&net(&["R1.1"])) && kicad.contains(&net(&["R2.1"])),
            "KiCad does not report the two nets the split made: {kicad:?}"
        );
    }
}

#[test]
fn a_deleted_wire_reports_the_junctions_it_stranded() {
    // A T: R1.1 and R2.1 along y = 50.8, R3.2 dropping onto the same point,
    // and a junction where the three ends meet. Taking the dropper out leaves
    // the junction sitting on two ends, which is legal — so it is reported and
    // it stays.
    //
    // A second junction sits at the dropper's other end, on R3.2, where it
    // was already joining one end and nothing else. Two junctions on one
    // removed segment are what make the point each is reported at load-bearing:
    // one point cannot be right for both.
    let mut probe = two_resistors("stranded-junction");
    probe.place("R", "R3", ("63.5", "33.02"), &["1", "2"]);
    probe.wire(("50.8", "50.8"), ("63.5", "50.8"));
    probe.wire(("63.5", "50.8"), ("76.2", "50.8"));
    probe.wire(("63.5", "50.8"), ("63.5", "36.83"));
    probe.junction(("63.5", "50.8"));
    probe.junction(("63.5", "36.83"));
    let path = probe.write();
    let project = path.parent().expect("the drawing sits in a directory");

    // The control: the three ends are one net before anything is deleted.
    let mut hierarchy = loaded(&path);
    let all_three = net(&["R1.1", "R2.1", "R3.2"]);
    assert!(
        kicli_partition(&extract(&hierarchy)).contains(&all_three),
        "the drawing does not start with the three pins on one net"
    );

    let meeting = at("63.5", "50.8");
    let far = at("63.5", "36.83");
    let junctions = junctions_at(&hierarchy);
    let dropper = segment_between(&hierarchy, meeting, far);
    let sheet = hierarchy.placements[0].path.clone();
    let deleted = delete(
        &mut hierarchy,
        &dropper,
        &target(&path, project, &sheet),
        "after",
    )
    .expect("the dropper is deletable");

    // The report names each junction, where it is, and what it is left
    // joining: the meeting is down to the two ends that still run through it,
    // and the one on R3.2 now joins nothing at all.
    assert_eq!(
        deleted.stranded.len(),
        2,
        "both junctions of the removed segment were reported: {:?}",
        deleted.stranded
    );
    for (point, ends) in [(meeting, 2), (far, 0)] {
        let stranded = deleted
            .stranded
            .iter()
            .find(|stranded| stranded.at == point)
            .unwrap_or_else(|| {
                panic!(
                    "no junction was reported at ({point}): {:?}",
                    deleted.stranded
                )
            });
        let named = junctions
            .iter()
            .find(|(seen, _)| *seen == point)
            .map(|(_, uuid)| uuid.clone())
            .expect("the drawing was written with a junction there");
        assert_eq!(stranded.junction.0, named, "the report names the junction");
        assert_eq!(
            stranded.ends.len(),
            ends,
            "what the junction at ({point}) is left joining: {:?}",
            stranded.ends
        );
    }
    assert!(
        deleted.mutation.invariants.passed(),
        "the invariants did not hold: {}",
        deleted.mutation.render()
    );

    // They were reported, not removed. The command does not cascade.
    let after = std::fs::read_to_string(&path).expect("the written file reads");
    for (_, junction) in &junctions {
        assert!(
            after.contains(junction),
            "a junction was removed as well as reported:\n{after}"
        );
    }
    assert_eq!(
        after.matches("(junction").count(),
        2,
        "the drawing still holds both its junctions"
    );
    assert_eq!(wire_records(&after), 2, "exactly one record was removed");

    // What the junction still joins, it still joins.
    let reloaded = loaded(&path);
    let partition = kicli_partition(&extract(&reloaded));
    assert!(
        partition.contains(&net(&["R1.1", "R2.1"])),
        "kicli lost the connection the junction still makes: {partition:?}"
    );
    if let Some(kicad) = oracle(&path) {
        assert!(
            kicad.contains(&net(&["R1.1", "R2.1"])),
            "KiCad lost the connection the junction still makes: {kicad:?}"
        );
        assert!(
            !kicad.contains(&all_three),
            "KiCad still reports the dropper's pin on the net: {kicad:?}"
        );
    }
}

#[test]
fn a_junction_still_joining_three_ends_is_not_reported() {
    // The boundary the rule is stated at. Four wire ends meet at one point,
    // which is a defect a file may hold and which `mark junction add` refuses
    // to make. Taking one wire away leaves three ends, which is what a
    // junction is for — so nothing is reported, and an implementation that
    // named every junction on the removed segment would fail here.
    let mut probe = Probe::new("three-ends-is-not-stranded", scratch());
    probe.wire(("50.8", "50.8"), ("63.5", "50.8"));
    probe.wire(("63.5", "50.8"), ("76.2", "50.8"));
    probe.wire(("63.5", "50.8"), ("63.5", "38.1"));
    probe.wire(("63.5", "50.8"), ("63.5", "63.5"));
    probe.junction(("63.5", "50.8"));
    let path = probe.write();
    let project = path.parent().expect("the drawing sits in a directory");

    let mut hierarchy = loaded(&path);
    let (junction, _) = only_junction(&hierarchy);
    let fourth = segment_between(&hierarchy, at("63.5", "50.8"), at("63.5", "63.5"));
    let sheet = hierarchy.placements[0].path.clone();
    let deleted = delete(
        &mut hierarchy,
        &fourth,
        &target(&path, project, &sheet),
        "after",
    )
    .expect("the fourth wire is deletable");

    assert!(
        deleted.stranded.is_empty(),
        "a junction joining three ends is doing its job: {:?}",
        deleted.stranded
    );
    let after = std::fs::read_to_string(&path).expect("the written file reads");
    assert!(after.contains(&junction), "the junction stayed");
    assert_eq!(wire_records(&after), 3, "exactly one record was removed");
}

#[test]
fn a_segment_is_named_by_its_identifier_or_its_handle() {
    // A view prints the eight-character handle, so a caller can type one back.
    // The harness cannot write two handles apart, so this drawing's first wire
    // is given an identifier of its own after it is written — the one thing
    // here that is not the harness's own bytes, and only because no probe
    // drawing can reach this behaviour.
    let mut probe = Probe::new("named-by-handle", scratch());
    probe.wire(("50.8", "50.8"), ("63.5", "50.8"));
    probe.wire(("63.5", "50.8"), ("76.2", "50.8"));
    probe.wire(("76.2", "50.8"), ("88.9", "50.8"));
    let path = probe.write();
    let project = path.parent().expect("the drawing sits in a directory");

    let written = std::fs::read_to_string(&path).expect("the drawing reads");
    let first = segment_between(&loaded(&path), at("50.8", "50.8"), at("63.5", "50.8"));
    let apart = "deadbeef-0000-4000-8000-000000000001";
    std::fs::write(&path, written.replace(&first, apart)).expect("the drawing is rewritten");

    // The ambiguous half first: the handle the two other records share matches
    // more than one, and the refusal lists what it matched rather than
    // choosing one of them.
    let mut hierarchy = loaded(&path);
    let sheet = hierarchy.placements[0].path.clone();
    let before = std::fs::read_to_string(&path).expect("the drawing reads");
    let refusal = delete(
        &mut hierarchy,
        "00000000",
        &target(&path, project, &sheet),
        "after",
    )
    .expect_err("more than one object carries that handle")
    .to_string();
    assert!(
        refusal.contains("00000000"),
        "the refusal names the handle it was given: {refusal}"
    );
    assert_eq!(
        before,
        std::fs::read_to_string(&path).expect("the drawing reads"),
        "the file was written anyway"
    );

    // And the unambiguous half: eight characters name one segment, and that
    // segment is the one that goes.
    let deleted = delete(
        &mut hierarchy,
        "deadbeef",
        &target(&path, project, &sheet),
        "after",
    )
    .expect("one segment carries that handle");
    assert_eq!(
        deleted.removed.0, apart,
        "the handle named the right segment"
    );
    let after = std::fs::read_to_string(&path).expect("the written file reads");
    assert!(
        !after.contains(apart),
        "the named segment is still in the file"
    );
    assert_eq!(wire_records(&after), 2, "exactly one record was removed");
}

#[test]
fn an_identifier_that_names_no_wire_is_refused() {
    // Two ways to name nothing: an identifier no object carries, and one that
    // a bundle carries. Neither writes a byte — a delete that half-happened
    // would leave a file no report describes.
    let mut probe = Probe::new("names-no-wire", scratch());
    probe.wire(("50.8", "50.8"), ("63.5", "50.8"));
    probe.bus(("50.8", "63.5"), ("63.5", "63.5"));
    let path = probe.write();
    let project = path.parent().expect("the drawing sits in a directory");
    let mut hierarchy = loaded(&path);
    let sheet = hierarchy.placements[0].path.clone();
    let bundle = hierarchy.files[0]
        .schematic
        .lines()
        .find(|line| line.kind == LineKind::Bus)
        .map(|line| line.uuid.0.clone())
        .expect("the drawing holds a bundle");
    let before = std::fs::read_to_string(&path).expect("the drawing reads");

    for (identifier, expected) in [("deadbeef", "deadbeef"), (bundle.as_str(), "bus")] {
        let refusal = delete(
            &mut hierarchy,
            identifier,
            &target(&path, project, &sheet),
            "after",
        )
        .expect_err("that identifier names no wire")
        .to_string();
        assert!(
            refusal.contains(expected),
            "the refusal does not say {expected}: {refusal}"
        );
        assert_eq!(
            before,
            std::fs::read_to_string(&path).expect("the drawing reads"),
            "the file was written anyway"
        );
    }
}
