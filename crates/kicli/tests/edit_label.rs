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
use kicli_sexpr::Doc;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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

/// The `kicad-cli` to run, or nothing when the caller did not ask for it.
fn kicad_cli() -> Option<String> {
    std::env::var("KICLI_TEST_KICAD_CLI").ok()?;
    Some(std::env::var("KICLI_KICAD_CLI").unwrap_or_else(|_| "kicad-cli".to_owned()))
}

/// Export a netlist of a project and read it back.
///
/// The tool's own output is dropped: the first run on a machine prints
/// fontconfig warnings that say nothing about the netlist.
fn export_netlist(tool: &str, root: &Path, into: &Path) -> Option<String> {
    let status = Command::new(tool)
        .args(["sch", "export", "netlist", "-o"])
        .arg(into)
        .arg(root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok()?;
    status
        .success()
        .then(|| std::fs::read_to_string(into).ok())?
}

/// One `(node (ref "R1") (pin "2") ...)` as `R1.2`.
fn node_label(doc: &Doc, node: kicli_sexpr::NodeId) -> Option<String> {
    let value = |head: &str| -> Option<String> {
        let list = doc
            .children(node)
            .iter()
            .copied()
            .find(|&child| doc.head_is(child, head))?;
        doc.children(list)
            .get(1)
            .and_then(|&id| doc.atom_as_str(id))
    };
    Some(format!("{}.{}", value("ref")?, value("pin")?))
}

/// The nets KiCad reports, as a name and a sorted pin list each.
fn kicad_nets(text: &str) -> Vec<(String, Vec<String>)> {
    let doc = Doc::parse(text).expect("the netlist parses");
    let root = doc.root().expect("the netlist has a root list");
    let mut found = Vec::new();
    for &child in doc.children(root) {
        if !doc.head_is(child, "nets") {
            continue;
        }
        for &net in doc.children(child) {
            if !doc.head_is(net, "net") {
                continue;
            }
            let name = doc
                .children(net)
                .iter()
                .copied()
                .find(|&list| doc.head_is(list, "name"))
                .and_then(|list| doc.children(list).get(1).copied())
                .and_then(|atom| doc.atom_as_str(atom))
                .unwrap_or_default();
            let mut pins: Vec<String> = doc
                .children(net)
                .iter()
                .filter(|&&node| doc.head_is(node, "node"))
                .filter_map(|&node| node_label(&doc, node))
                .collect();
            pins.sort();
            found.push((name, pins));
        }
    }
    assert!(!found.is_empty(), "the netlist reported no nets at all");
    found
}

#[test]
fn kicad_agrees_about_the_new_net() {
    let Some(tool) = kicad_cli() else {
        eprintln!("skipped: set KICLI_TEST_KICAD_CLI to run kicad-cli");
        return;
    };
    // A label on the wire's free stretch joins it. A label where a pin already
    // sits does not, and takes only the pin. KiCad decides which, not kicli.
    oracle_agrees(&tool, "label_netlist_oracle", FREE, &WIRE_PINS);
    oracle_agrees(&tool, "label_netlist_shared", SHARED, &["R11.1"]);
}

/// Add a label, then hold the report against KiCad's own netlist.
fn oracle_agrees(tool: &str, name: &str, at: Point, pins: &[&str]) {
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

    let netlist = export_netlist(tool, &project.root, &project.directory.join("after.net"))
        .expect("kicad-cli exported a netlist of the file kicli wrote");
    let kicad = kicad_nets(&netlist);

    let claimed = &added.nets.after[0];
    assert_eq!(claimed.pins, pins, "the report claims what was measured");
    let named: Vec<&(String, Vec<String>)> = kicad
        .iter()
        .filter(|(name, _)| name.ends_with(&claimed.name))
        .collect();
    assert_eq!(
        named.len(),
        1,
        "KiCad's netlist carries the net the report claimed: {kicad:?}"
    );
    assert_eq!(named[0].1, claimed.pins, "with the pins the report named");

    // And the whole partition still matches, so the label moved nothing else.
    let theirs: BTreeSet<Vec<String>> = kicad
        .iter()
        .map(|(_, pins)| pins.clone())
        .filter(|pins| !pins.is_empty())
        .collect();
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
