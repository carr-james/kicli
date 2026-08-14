//! Renaming a net is renaming its labels, and a net with no label has no name.
//!
//! Every test copies the connectivity fixture into a scratch directory and
//! mutates the copy. The committed fixture tree is never written by a test.

use kicli::connectivity::{Net, NetPin, Nets, extract};
use kicli::edit::net::{self, Scope};
use kicli::geometry::GRID;
use kicli::model::{Hierarchy, WriteOptions};
use kicli_sexpr::Doc;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

mod support;

/// A net partition: one sorted pin list per net.
type Partition = BTreeSet<Vec<String>>;

/// Copy the connectivity fixture into a scratch directory, and name its root.
fn nets_project(name: &str) -> PathBuf {
    support::scratch_directory(name, "sch/nets").join("nets.kicad_sch")
}

fn scope(project: &Path) -> Scope<'_> {
    Scope {
        project,
        grid: GRID,
        options: WriteOptions::default(),
    }
}

fn nets_now(root: &Path) -> Nets {
    extract(&Hierarchy::load(root).expect("the hierarchy loads"))
}

/// The pins of one net, as a netlist would list them.
fn pins_of(nets: &Nets, name: &str) -> Vec<String> {
    nets.nets()
        .iter()
        .find(|net| net.name == name)
        .map(|net| net.pins.iter().map(NetPin::label).collect())
        .unwrap_or_default()
}

/// The whole partition kicli reads, power pins left out as a netlist leaves
/// them out.
fn partition_of(nets: &Nets) -> Partition {
    nets.nets()
        .iter()
        .map(|net| {
            net.pins
                .iter()
                .filter(|pin| !pin.power)
                .map(NetPin::label)
                .collect::<Vec<String>>()
        })
        .filter(|pins| !pins.is_empty())
        .collect()
}

#[test]
fn renaming_a_net_renames_every_label_it_has() {
    let root = nets_project("rename_every_label");
    let project = root.parent().expect("the root has a directory").to_owned();

    // GLOBAL_A is drawn on the root sheet and on both placements of the child
    // sheet, so its labels live in two files.
    let before = pins_of(&nets_now(&root), "GLOBAL_A");
    assert_eq!(before, ["R101.2", "R14.1", "R201.2"]);

    let mut hierarchy = Hierarchy::load(&root).expect("the hierarchy loads");
    let renamed = net::rename(
        &mut hierarchy,
        "GLOBAL_A",
        "GLOBAL_Z",
        &scope(&project),
        "2026-01-02T03:04:05Z",
    )
    .expect("the net is renamed");

    assert_eq!(renamed.labels.len(), 2, "both global labels changed");
    assert_eq!(
        renamed.mutations.len(),
        2,
        "one report per file the net reaches"
    );
    for mutation in &renamed.mutations {
        assert!(
            mutation.invariants.passed(),
            "every invariant held: {:?}",
            mutation.invariants.failures().collect::<Vec<_>>()
        );
    }

    let after = nets_now(&root);
    assert_eq!(pins_of(&after, "GLOBAL_Z"), before, "the net kept its pins");
    assert!(
        pins_of(&after, "GLOBAL_A").is_empty(),
        "and nothing is called GLOBAL_A now"
    );
    for file in ["nets.kicad_sch", "nets_channel.kicad_sch"] {
        let text = std::fs::read_to_string(project.join(file)).expect("the file reads");
        assert!(text.contains("GLOBAL_Z"), "{file} carries the new name");
        assert!(!text.contains("GLOBAL_A"), "{file} carries the old name");
    }
}

#[test]
fn an_unnamed_net_cannot_be_renamed() {
    let root = nets_project("rename_unnamed");
    let project = root.parent().expect("the root has a directory").to_owned();
    let before = std::fs::read_to_string(&root).expect("the sheet reads");

    let nets = nets_now(&root);
    let handle = nets
        .nets()
        .iter()
        .find(|net| net.synthetic)
        .map(|net: &Net| net.name.clone())
        .expect("the fixture has a net that no label names");

    let mut hierarchy = Hierarchy::load(&root).expect("the hierarchy loads");
    let refused = net::rename(
        &mut hierarchy,
        &handle,
        "SOMETHING",
        &scope(&project),
        "2026-01-02T03:04:05Z",
    )
    .expect_err("a net with no label has no name to change");

    let said = refused.to_string();
    assert!(
        said.contains(&handle),
        "the refusal names the handle: {said}"
    );
    assert!(
        said.contains("label add"),
        "and says how to name the net: {said}"
    );
    assert_eq!(
        std::fs::read_to_string(&root).expect("the sheet reads"),
        before,
        "the refusal wrote nothing"
    );
}

#[test]
fn renaming_a_power_net_edits_its_symbols() {
    let root = nets_project("rename_power");
    let project = root.parent().expect("the root has a directory").to_owned();
    let before = pins_of(&nets_now(&root), "GND");

    let mut hierarchy = Hierarchy::load(&root).expect("the hierarchy loads");
    let renamed = net::rename(
        &mut hierarchy,
        "GND",
        "GNDA",
        &scope(&project),
        "2026-01-02T03:04:05Z",
    )
    .expect("the power net is renamed");

    assert_eq!(
        renamed.power_symbols,
        ["#PWR01", "#PWR02", "#PWR03"],
        "the command says which symbols it changed"
    );
    assert!(
        renamed.labels.is_empty(),
        "a power net has no label to change"
    );
    assert!(
        renamed.render().contains("#PWR01"),
        "and the report names them: {}",
        renamed.render()
    );

    let after = nets_now(&root);
    assert_eq!(pins_of(&after, "GNDA"), before, "the net kept its pins");
    assert!(pins_of(&after, "GND").is_empty());
}

#[test]
fn a_name_two_nets_share_is_not_a_handle() {
    let root = nets_project("rename_ambiguous");
    let project = root.parent().expect("the root has a directory").to_owned();
    let before = std::fs::read_to_string(&root).expect("the sheet reads");

    // The child sheet is placed twice, and its local label names one net per
    // placement. Two nets are called CHAN_LOCAL.
    let mut hierarchy = Hierarchy::load(&root).expect("the hierarchy loads");
    let refused = net::rename(
        &mut hierarchy,
        "CHAN_LOCAL",
        "CHAN_Z",
        &scope(&project),
        "2026-01-02T03:04:05Z",
    )
    .expect_err("a name two nets share does not identify one of them");
    assert!(refused.to_string().contains("2 nets"), "{refused}");
    assert_eq!(
        std::fs::read_to_string(&root).expect("the sheet reads"),
        before,
        "the refusal wrote nothing"
    );
}

#[test]
fn a_name_another_net_already_has_is_refused() {
    let root = nets_project("rename_taken");
    let project = root.parent().expect("the root has a directory").to_owned();

    let mut hierarchy = Hierarchy::load(&root).expect("the hierarchy loads");
    let refused = net::rename(
        &mut hierarchy,
        "NET_A",
        "GLOBAL_A",
        &scope(&project),
        "2026-01-02T03:04:05Z",
    )
    .expect_err("renaming one net onto another would join them");
    assert!(refused.to_string().contains("GLOBAL_A"), "{refused}");
}

#[test]
fn kicad_agrees_about_the_rename() {
    let Some(tool) = kicad_cli() else {
        eprintln!("skipped: set KICLI_TEST_KICAD_CLI to compare against KiCad");
        return;
    };
    let root = nets_project("rename_oracle");
    let project = root.parent().expect("the root has a directory").to_owned();

    let first = export_netlist(&tool, &root, &project.join("before.net"))
        .expect("kicad-cli exported a netlist");
    let partition_before = kicad_partition(&first);
    assert!(names_in(&first).contains("/NET_A"));

    // A local label, a global label across two files, and a power value.
    for (from, to) in [
        ("NET_A", "NET_Z"),
        ("GLOBAL_A", "GLOBAL_Z"),
        ("GND", "GNDA"),
    ] {
        let mut hierarchy = Hierarchy::load(&root).expect("the hierarchy loads");
        net::rename(
            &mut hierarchy,
            from,
            to,
            &scope(&project),
            "2026-01-02T03:04:05Z",
        )
        .unwrap_or_else(|error| panic!("renaming {from} to {to}: {error}"));
    }

    let second = export_netlist(&tool, &root, &project.join("after.net"))
        .expect("kicad-cli exported a netlist");
    let names = names_in(&second);
    for wanted in ["/NET_Z", "GLOBAL_Z", "GNDA"] {
        assert!(names.contains(wanted), "KiCad reports {wanted}: {names:?}");
    }
    for gone in ["/NET_A", "GLOBAL_A", "GND"] {
        assert!(
            !names.contains(gone),
            "KiCad reports {gone} still: {names:?}"
        );
    }
    assert_eq!(
        kicad_partition(&second),
        partition_before,
        "the rename changed the names and not the partition"
    );
    assert_eq!(
        partition_of(&nets_now(&root)),
        partition_before,
        "and kicli reads the same partition KiCad does"
    );
}

/// The `kicad-cli` to run, or nothing when the caller did not ask for it.
fn kicad_cli() -> Option<String> {
    std::env::var("KICLI_TEST_KICAD_CLI").ok()?;
    Some(std::env::var("KICLI_KICAD_CLI").unwrap_or_else(|_| "kicad-cli".to_owned()))
}

/// Export a netlist and read it back.
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
    if !status.success() {
        return None;
    }
    std::fs::read_to_string(into).ok()
}

/// Every net's own name, as KiCad wrote it.
fn names_in(netlist: &str) -> BTreeSet<String> {
    let doc = Doc::parse(netlist).expect("the netlist parses");
    let root = doc.root().expect("the netlist has a root list");
    let mut names = BTreeSet::new();
    for &child in doc.children(root) {
        if !doc.head_is(child, "nets") {
            continue;
        }
        for &net in doc.children(child) {
            if !doc.head_is(net, "net") {
                continue;
            }
            if let Some(name) = value_of(&doc, net, "name") {
                names.insert(name);
            }
        }
    }
    assert!(!names.is_empty(), "the netlist reported no nets at all");
    names
}

/// The partition KiCad reports, read out of a netlist it wrote.
fn kicad_partition(netlist: &str) -> Partition {
    let doc = Doc::parse(netlist).expect("the netlist parses");
    let root = doc.root().expect("the netlist has a root list");
    let mut found = Partition::new();
    for &child in doc.children(root) {
        if !doc.head_is(child, "nets") {
            continue;
        }
        for &net in doc.children(child) {
            if !doc.head_is(net, "net") {
                continue;
            }
            let mut pins: Vec<String> = doc
                .children(net)
                .iter()
                .filter(|&&node| doc.head_is(node, "node"))
                .filter_map(|&node| {
                    Some(format!(
                        "{}.{}",
                        value_of(&doc, node, "ref")?,
                        value_of(&doc, node, "pin")?
                    ))
                })
                .collect();
            pins.sort();
            if !pins.is_empty() {
                found.insert(pins);
            }
        }
    }
    assert!(!found.is_empty(), "the netlist reported no nets at all");
    found
}

/// The first value of a named child list.
fn value_of(doc: &Doc, node: kicli_sexpr::NodeId, head: &str) -> Option<String> {
    let list = doc
        .children(node)
        .iter()
        .copied()
        .find(|&child| doc.head_is(child, head))?;
    doc.children(list)
        .get(1)
        .and_then(|&id| doc.atom_as_str(id))
}
