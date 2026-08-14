//! Labels: what adding, moving and deleting one does to the file and the nets.
//!
//! Every test mutates a copy of a fixture in a scratch directory. The
//! committed fixture tree is never written by a test.
//!
//! The wire from `25.4,88.9` to `50.8,88.9` carries `R12.2` and `R13.1`, and
//! `R11.1` sits on its interior at `38.1,88.9`. That one wire holds both cases
//! the label rule turns on: a free stretch, where a label joins the wire, and
//! an anchor a pin already occupies, where it does not.

use kicli::connectivity::{NetPin, Nets, extract};
use kicli::edit::label::{self, NewLabel, PortShape};
use kicli::geometry::{Angle, GRID, Point};
use kicli::model::{Hierarchy, LabelKind, Schematic, SheetPath, Target, WriteOptions};
use kicli_probe::oracle::Kicad;
use kicli_sexpr::Doc;
use std::collections::BTreeSet;
use std::path::PathBuf;

mod support;

/// A point on the wire's free stretch, where nothing else sits.
const FREE: Point = Point::new(304_800, 889_000);

/// The point where `R11.1` sits, on the same wire's interior.
const SHARED: Point = Point::new(381_000, 889_000);

/// The pins of the wire's own net.
const WIRE_PINS: [&str; 2] = ["R12.2", "R13.1"];

/// A copy of the net fixture project, with the paths a command needs.
struct Project {
    directory: PathBuf,
    root: PathBuf,
}

impl Project {
    /// Copy the committed project into a scratch directory of its own.
    fn new(name: &str) -> Self {
        let directory = support::scratch_directory(name, "sch/nets");
        Self {
            root: directory.join("nets.kicad_sch"),
            directory,
        }
    }

    /// The root sheet as it stands.
    fn bytes(&self) -> String {
        std::fs::read_to_string(&self.root).expect("the sheet reads")
    }

    /// The document and the sheet path a command needs.
    fn open(&self) -> (Doc, SheetPath) {
        let doc = Doc::parse(&self.bytes()).expect("the sheet parses");
        let schematic = Schematic::read(&doc).expect("the sheet reads");
        let path = SheetPath::root(schematic.uuid.as_ref().expect("the sheet has a uuid"));
        (doc, path)
    }

    /// Where a write lands, and under what rules.
    fn target<'a>(&'a self, path: &'a SheetPath) -> Target<'a> {
        Target {
            path: &self.root,
            project: &self.directory,
            sheet_path: path,
            grid: GRID,
            options: WriteOptions::default(),
        }
    }

    /// The nets of the project as it stands on disk.
    fn nets(&self) -> Nets {
        let hierarchy = Hierarchy::load(&self.root).expect("the hierarchy loads");
        extract(&hierarchy)
    }
}

fn local(text: &str, at: Point) -> NewLabel {
    NewLabel {
        kind: LabelKind::Local,
        text: text.to_owned(),
        at,
        angle: Angle(0),
        shape: PortShape::Passive,
    }
}

/// The pins of the net with a name, as the extractor reports them.
fn pins_of(nets: &Nets, name: &str) -> Vec<String> {
    nets.nets()
        .iter()
        .find(|net| net.name == name)
        .map(|net| net.pins.iter().map(NetPin::label).collect())
        .unwrap_or_default()
}

#[test]
fn adding_a_label_says_what_it_connected() {
    let project = Project::new("label_joins_a_wire");
    let before = pins_of(&project.nets(), "SPY");
    assert!(before.is_empty(), "no net is called SPY to start with");

    let (mut doc, path) = project.open();
    let added = label::add(
        &mut doc,
        &project.target(&path),
        &project.root,
        &local("SPY", FREE),
        "2026-01-02T03:04:05Z",
    )
    .expect("the label is added");

    assert!(added.mutation.invariants.passed());
    assert_eq!(
        added.nets.after.len(),
        1,
        "one net changed: {}",
        added.render()
    );
    let joined = &added.nets.after[0];
    assert_eq!(joined.name, "SPY");
    assert_eq!(
        joined.pins, WIRE_PINS,
        "the label joined the wire's own net"
    );
    assert_eq!(
        added.nets.before.len(),
        1,
        "and it names what that net was called: {}",
        added.render()
    );
    assert_eq!(added.nets.before[0].pins, WIRE_PINS);
    assert!(
        added.render().contains("net SPY: R12.2 R13.1"),
        "{}",
        added.render()
    );

    // The extractor agrees, reading the file kicli wrote.
    assert_eq!(pins_of(&project.nets(), "SPY"), WIRE_PINS);
}

#[test]
fn a_label_anchor_lands_on_the_grid() {
    let project = Project::new("label_on_the_grid");
    let (mut doc, path) = project.open();
    // One hundredth of a millimetre off the grid, which the anchor may not be.
    let asked = Point::new(FREE.x.0 + 100, FREE.y.0);
    let added = label::add(
        &mut doc,
        &project.target(&path),
        &project.root,
        &local("SPY", asked),
        "2026-01-02T03:04:05Z",
    )
    .expect("the label is added");

    assert_eq!(added.at, FREE, "the anchor moved onto the grid");
    assert_eq!(added.requested, asked);
    assert!(added.snapped());
    assert!(
        added.notes.iter().any(|note| note.contains("grid")),
        "and the report says so: {:?}",
        added.notes
    );
    assert!(project.bytes().contains("(at 30.48 88.9 0)"));
    // Which is what makes the on-grid invariant pass rather than fail.
    assert!(added.mutation.invariants.passed());
}

#[test]
fn a_label_that_shares_an_anchor_with_a_pin_does_not_join_the_wire() {
    let project = Project::new("label_shares_an_anchor");
    let (mut doc, path) = project.open();
    let added = label::add(
        &mut doc,
        &project.target(&path),
        &project.root,
        &local("SPY", SHARED),
        "2026-01-02T03:04:05Z",
    )
    .expect("the label is added");

    assert_eq!(
        added.nets.after.len(),
        1,
        "one net changed: {}",
        added.render()
    );
    assert_eq!(
        added.nets.after[0].pins,
        vec!["R11.1".to_owned()],
        "the label named the pin's net and left the wire out of it"
    );
    assert!(
        added
            .notes
            .iter()
            .any(|note| note.contains("pin") && note.contains("anchor")),
        "and the report warns about the shared anchor: {:?}",
        added.notes
    );
    assert_eq!(
        pins_of(&project.nets(), "SPY"),
        vec!["R11.1".to_owned()],
        "which is what the extractor reads back"
    );
}

#[test]
fn a_hierarchical_label_says_its_sheet_pin_is_missing() {
    let project = Project::new("label_hierarchical");
    let (mut doc, path) = project.open();
    let request = NewLabel {
        kind: LabelKind::Hierarchical,
        shape: PortShape::Input,
        ..local("PORT_IN", FREE)
    };
    let added = label::add(
        &mut doc,
        &project.target(&path),
        &project.root,
        &request,
        "2026-01-02T03:04:05Z",
    )
    .expect("the label is added");

    assert!(
        added
            .notes
            .iter()
            .any(|note| note.contains("sheet pin") || note.contains("sheet port")),
        "{:?}",
        added.notes
    );
    let written = project.bytes();
    assert!(
        written.contains("(hierarchical_label \"PORT_IN\""),
        "{written}"
    );
    assert!(written.contains("(shape input)"), "{written}");
}

#[test]
fn a_deleted_label_leaves_the_file_as_it_was() {
    let project = Project::new("label_add_and_delete");
    let start = project.bytes();

    let (mut doc, path) = project.open();
    let added = label::add(
        &mut doc,
        &project.target(&path),
        &project.root,
        &local("SPY", FREE),
        "2026-01-02T03:04:05Z",
    )
    .expect("the label is added");
    assert!(!added.mutation.reformatted, "the fixture arrived canonical");

    let (mut doc, path) = project.open();
    let removed = label::delete(
        &mut doc,
        &project.target(&path),
        &project.root,
        &added.uuid,
        "2026-01-02T03:04:06Z",
    )
    .expect("the label goes");

    assert_eq!(project.bytes(), start, "byte for byte");
    assert_eq!(
        removed.nets.before[0].name,
        "SPY",
        "and the report says the net lost the name: {}",
        removed.render()
    );
}

#[test]
fn a_moved_label_takes_its_name_off_the_wire() {
    let project = Project::new("label_move");
    let (mut doc, path) = project.open();
    let added = label::add(
        &mut doc,
        &project.target(&path),
        &project.root,
        &local("SPY", FREE),
        "2026-01-02T03:04:05Z",
    )
    .expect("the label is added");

    let (mut doc, path) = project.open();
    // Far from any wire, where the label names nothing.
    let moved = label::move_to(
        &mut doc,
        &project.target(&path),
        &project.root,
        &added.uuid,
        Point::new(2_540_000, 2_540_000),
        "2026-01-02T03:04:06Z",
    )
    .expect("the label moves");

    assert!(
        moved.nets.before.iter().any(|net| net.name == "SPY"),
        "{}",
        moved.render()
    );
    assert!(pins_of(&project.nets(), "SPY").is_empty());
    assert!(
        moved.notes.iter().any(|note| note.contains("No wire")),
        "{:?}",
        moved.notes
    );
}

#[test]
fn kicad_agrees_about_the_new_net() {
    let Some(tool) = Kicad::found_or_skip("run kicad-cli") else {
        return;
    };
    // A label on the wire's free stretch joins it. A label where a pin already
    // sits does not, and takes only the pin. KiCad decides which, not kicli.
    oracle_agrees(&tool, "label_netlist_oracle", FREE, &WIRE_PINS);
    oracle_agrees(&tool, "label_netlist_shared", SHARED, &["R11.1"]);
}

/// Add a label, then hold the report against KiCad's own netlist.
fn oracle_agrees(tool: &Kicad, name: &str, at: Point, pins: &[&str]) {
    let project = Project::new(name);
    let (mut doc, path) = project.open();
    let added = label::add(
        &mut doc,
        &project.target(&path),
        &project.root,
        &local("SPY", at),
        "2026-01-02T03:04:05Z",
    )
    .expect("the label is added");

    let netlist = tool.netlist(&project.root, &project.directory.join("after.net"));

    let claimed = &added.nets.after[0];
    assert_eq!(claimed.pins, pins, "the report claims what was measured");
    let named = netlist.named(&claimed.name);
    assert_eq!(
        named.len(),
        1,
        "KiCad's netlist carries the net the report claimed: {:?}",
        netlist.nets()
    );
    assert_eq!(
        named[0].pins, claimed.pins,
        "with the pins the report named"
    );

    // And the whole partition still matches, so the label moved nothing else.
    let theirs: BTreeSet<Vec<String>> = netlist.partition();
    let ours: BTreeSet<Vec<String>> = project
        .nets()
        .nets()
        .iter()
        .map(|net| {
            net.pins
                .iter()
                .filter(|pin| !pin.power)
                .map(NetPin::label)
                .collect::<Vec<String>>()
        })
        .filter(|pins| !pins.is_empty())
        .collect();
    assert_eq!(ours, theirs, "kicli's partition is KiCad's partition");
}
