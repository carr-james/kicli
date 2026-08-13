//! Junctions and no-connects, and the two refusals that come with them.
//!
//! Every test copies a fixture into a scratch directory and mutates the copy.
//! The committed fixture tree is never written by a test.

use kicli::connectivity::{NetPin, Nets, extract};
use kicli::edit::mark;
use kicli::geometry::{GRID, Point};
use kicli::model::{Hierarchy, Refdes, SheetPath, Target, Uuid, WriteOptions};
use std::path::{Path, PathBuf};

/// The root sheet of the connectivity fixture.
const NETS_ROOT: &str = "00000000-0000-4000-8000-030000000000";

/// The mid-span pin cluster: a wire from `(25.4,88.9)` to `(50.8,88.9)` with
/// R11 pin 1 at its middle, and no junction to join them.
const MID_SPAN: Point = Point::new(381_000, 889_000);

fn scratch(name: &str) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("the scratch directory is made");
    directory
}

/// Copy the connectivity fixture into a scratch directory, and name its root.
fn nets_project(name: &str) -> PathBuf {
    let from = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sch/nets");
    let into = scratch(name);
    for entry in std::fs::read_dir(&from).expect("the fixture directory reads") {
        let path = entry.expect("a directory entry reads").path();
        if path.is_file() {
            let name = path.file_name().expect("a file has a name");
            std::fs::copy(&path, into.join(name)).expect("the copy is writable");
        }
    }
    into.join("nets.kicad_sch")
}

/// A schematic whose only objects are four wire ends meeting at one point.
fn crossroads() -> String {
    let mut sheet =
        String::from("(kicad_sch\n\t(version 20260306)\n\t(uuid \"root\")\n\t(paper \"A4\")\n");
    for (index, (from, to)) in [
        ((38.1, 50.8), (50.8, 50.8)),
        ((50.8, 50.8), (63.5, 50.8)),
        ((50.8, 38.1), (50.8, 50.8)),
        ((50.8, 50.8), (50.8, 63.5)),
    ]
    .into_iter()
    .enumerate()
    {
        sheet.push_str(&format!(
            "\t(wire\n\t\t(pts\n\t\t\t(xy {} {}) (xy {} {})\n\t\t)\n\t\t(uuid \"wire{index}\")\n\t)\n",
            from.0, from.1, to.0, to.1
        ));
    }
    sheet.push_str(")\n");
    sheet
}

/// The pins of the net one pin is on, as a netlist would list them.
fn net_of(nets: &Nets, reference: &str, number: &str) -> Vec<String> {
    nets.net_of(reference, number)
        .map(|net| net.pins.iter().map(NetPin::label).collect())
        .unwrap_or_default()
}

/// Load a hierarchy, read its nets, and let it go again.
fn nets_now(root: &Path) -> Nets {
    extract(&Hierarchy::load(root).expect("the hierarchy loads"))
}

fn target<'a>(file: &'a Path, project: &'a Path, sheet: &'a SheetPath) -> Target<'a> {
    Target {
        path: file,
        project,
        sheet_path: sheet,
        grid: GRID,
        options: WriteOptions::default(),
    }
}

#[test]
fn a_four_way_junction_is_refused() {
    let project = scratch("four_way_junction");
    let file = project.join("board.kicad_sch");
    let source = crossroads();
    std::fs::write(&file, &source).expect("the sheet is written");

    let mut hierarchy = Hierarchy::load(&file).expect("the hierarchy loads");
    let sheet = SheetPath::root(&Uuid("root".to_owned()));
    let refused = mark::add_junction(
        &mut hierarchy,
        Point::new(508_000, 508_000),
        &Uuid("new".to_owned()),
        &target(&file, &project, &sheet),
        "2026-01-02T03:04:05Z",
    )
    .expect_err("a junction where four wire ends meet is refused");

    let said = refused.to_string();
    assert!(said.contains("four wire ends meet"), "{said}");
    for wire in ["wire0", "wire1", "wire2", "wire3"] {
        assert!(
            said.contains(wire),
            "the refusal names every wire end: {said}"
        );
    }
    assert!(
        said.contains("one grid step"),
        "the refusal says what the fix is: {said}"
    );
    assert_eq!(
        std::fs::read_to_string(&file).expect("the sheet reads"),
        source,
        "the refusal wrote nothing"
    );
}

#[test]
fn a_three_way_junction_is_allowed() {
    let project = scratch("three_way_junction");
    let file = project.join("board.kicad_sch");
    // The same crossroads with one arm removed leaves three wire ends.
    let source = crossroads().replace(
        "\t(wire\n\t\t(pts\n\t\t\t(xy 50.8 50.8) (xy 50.8 63.5)\n\t\t)\n\t\t(uuid \"wire3\")\n\t)\n",
        "",
    );
    std::fs::write(&file, &source).expect("the sheet is written");

    let mut hierarchy = Hierarchy::load(&file).expect("the hierarchy loads");
    let sheet = SheetPath::root(&Uuid("root".to_owned()));
    let mutation = mark::add_junction(
        &mut hierarchy,
        Point::new(508_000, 508_000),
        &Uuid("new".to_owned()),
        &target(&file, &project, &sheet),
        "2026-01-02T03:04:05Z",
    )
    .expect("three wire ends are not a four-way junction");

    assert!(mutation.invariants.passed(), "{:?}", mutation.invariants);
    assert!(
        std::fs::read_to_string(&file)
            .expect("the sheet reads")
            .contains("(junction"),
        "the junction was written"
    );
}

#[test]
fn a_junction_joins_what_it_sits_on() {
    let root = nets_project("junction_joins");
    let project = root.parent().expect("the root has a directory").to_owned();
    let sheet = SheetPath::root(&Uuid(NETS_ROOT.to_owned()));

    assert_eq!(
        net_of(&nets_now(&root), "R11", "1"),
        ["R11.1"],
        "a pin on a wire's interior does not connect without a junction"
    );

    let mut hierarchy = Hierarchy::load(&root).expect("the hierarchy loads");
    mark::add_junction(
        &mut hierarchy,
        MID_SPAN,
        &Uuid("00000000-0000-4000-8000-03000000ff01".to_owned()),
        &target(&root, &project, &sheet),
        "2026-01-02T03:04:05Z",
    )
    .expect("the junction is added");

    assert_eq!(
        net_of(&nets_now(&root), "R11", "1"),
        ["R11.1", "R12.2", "R13.1"],
        "the junction joined the pin to the wire it sits on"
    );

    let mut hierarchy = Hierarchy::load(&root).expect("the hierarchy loads again");
    mark::delete_junction(
        &mut hierarchy,
        MID_SPAN,
        &target(&root, &project, &sheet),
        "2026-01-02T03:04:06Z",
    )
    .expect("the junction is deleted");

    assert_eq!(
        net_of(&nets_now(&root), "R11", "1"),
        ["R11.1"],
        "deleting the junction separated them again"
    );
}

#[test]
fn a_no_connect_on_a_connected_pin_is_refused() {
    let root = nets_project("no_connect_refused");
    let project = root.parent().expect("the root has a directory").to_owned();
    let sheet = SheetPath::root(&Uuid(NETS_ROOT.to_owned()));
    let before = std::fs::read_to_string(&root).expect("the sheet reads");

    // R1 pin 1 sits on a wire that carries the label NET_A, which joins it
    // to R2 pin 1.
    let mut hierarchy = Hierarchy::load(&root).expect("the hierarchy loads");
    let refused = mark::add_no_connect(
        &mut hierarchy,
        &mark::PinAddress::new(Refdes("R1".to_owned()), "1"),
        &Uuid("00000000-0000-4000-8000-03000000ff02".to_owned()),
        &target(&root, &project, &sheet),
        "2026-01-02T03:04:05Z",
    )
    .expect_err("a no-connect on a joined pin is refused");

    let said = refused.to_string();
    assert!(said.contains("R1.1"), "the refusal names the pin: {said}");
    assert!(
        said.contains("R2.1"),
        "and what the pin is joined to: {said}"
    );
    assert!(
        said.contains("NET_A"),
        "and the net that joins them: {said}"
    );
    assert!(said.contains("wire"), "and the wire it sits on: {said}");
    assert_eq!(
        std::fs::read_to_string(&root).expect("the sheet reads"),
        before,
        "the refusal wrote nothing"
    );
}

#[test]
fn a_no_connect_lands_on_its_pin() {
    let root = nets_project("no_connect_lands");
    let project = root.parent().expect("the root has a directory").to_owned();
    let sheet = SheetPath::root(&Uuid(NETS_ROOT.to_owned()));

    // R11 sits at (38.1, 92.71) and its pin 2 hangs below, joined to nothing.
    let free = mark::PinAddress::new(Refdes("R11".to_owned()), "2");
    let mut hierarchy = Hierarchy::load(&root).expect("the hierarchy loads");
    let mutation = mark::add_no_connect(
        &mut hierarchy,
        &free,
        &Uuid("00000000-0000-4000-8000-03000000ff03".to_owned()),
        &target(&root, &project, &sheet),
        "2026-01-02T03:04:05Z",
    )
    .expect("a free pin takes a no-connect");

    assert!(
        mutation.invariants.passed(),
        "every invariant held: {:?}",
        mutation.invariants.failures().collect::<Vec<_>>()
    );
    let written = std::fs::read_to_string(&root).expect("the sheet reads");
    assert!(
        written.contains("(no_connect\n\t\t(at 38.1 96.52)"),
        "the marker is at the pin's connection point: {written}"
    );
    assert!(
        Point::new(381_000, 965_200).is_on_grid(),
        "and that point is on the grid"
    );

    // A second marker on one pin says nothing the first does not.
    let mut hierarchy = Hierarchy::load(&root).expect("the hierarchy loads again");
    let refused = mark::add_no_connect(
        &mut hierarchy,
        &free,
        &Uuid("00000000-0000-4000-8000-03000000ff04".to_owned()),
        &target(&root, &project, &sheet),
        "2026-01-02T03:04:06Z",
    )
    .expect_err("one no-connect per pin is enough");
    assert!(refused.to_string().contains("already"), "{refused}");

    // Deleting it puts the pin back where it started.
    let mut hierarchy = Hierarchy::load(&root).expect("the hierarchy loads again");
    mark::delete_no_connect(
        &mut hierarchy,
        &free,
        &target(&root, &project, &sheet),
        "2026-01-02T03:04:07Z",
    )
    .expect("the marker is deleted");
    assert!(
        !std::fs::read_to_string(&root)
            .expect("the sheet reads")
            .contains("no_connect"),
        "the marker is gone"
    );
}
