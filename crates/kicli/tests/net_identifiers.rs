//! A net keeps its name and its handle when the drawing around it changes.
//!
//! KiCad names a net after one of its pins, so renumbering that symbol renames
//! the net. kicli carries that name as an attribute and addresses nets by its
//! own, which is a property of the design and not of the numbering.

use kicli::connectivity::{Net, NetPin, Nets, extract};
use kicli::model::Hierarchy;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sch/nets")
}

fn nets_of(root: &Path) -> Nets {
    let hierarchy = Hierarchy::load(root).expect("the fixture loads");
    extract(&hierarchy)
}

fn fixture_nets() -> Nets {
    nets_of(&fixtures().join("nets.kicad_sch"))
}

/// The net one pin is on.
fn net<'a>(nets: &'a Nets, reference: &str, number: &str) -> &'a Net {
    nets.net_of(reference, number).expect("the pin is on a net")
}

fn pins(net: &Net) -> Vec<String> {
    net.pins
        .iter()
        .filter(|pin| !pin.power)
        .map(NetPin::label)
        .collect()
}

/// Every net kicli named itself, by the handle it gave it.
fn synthetic(nets: &Nets) -> BTreeMap<String, Vec<String>> {
    nets.nets()
        .iter()
        .filter(|net| net.synthetic)
        .map(|net| (net.name.clone(), pins(net)))
        .collect()
}

/// Copy the fixture into a scratch directory, with one symbol renumbered.
fn renumbered(from: &str, to: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "kicli-nets-{}-{}",
        std::process::id(),
        from.to_lowercase()
    ));
    std::fs::create_dir_all(&directory).expect("the scratch directory is writable");
    for name in ["nets.kicad_sch", "nets_channel.kicad_sch"] {
        let text = std::fs::read_to_string(fixtures().join(name)).expect("the fixture is readable");
        let renamed = text.replace(&format!("\"{from}\""), &format!("\"{to}\""));
        std::fs::write(directory.join(name), renamed).expect("the copy is writable");
    }
    directory.join("nets.kicad_sch")
}

#[test]
fn net_ids_survive_an_unrelated_rename() {
    let before = fixture_nets();
    // R21 becomes R22, which sorts in the same place, so nothing else about
    // the design changes.
    let after = nets_of(&renumbered("R21", "R22"));

    let renamed: BTreeMap<String, Vec<String>> = synthetic(&before)
        .into_iter()
        .map(|(id, pins)| {
            let pins = pins
                .into_iter()
                .map(|pin| pin.replace("R21.", "R22."))
                .collect();
            (id, pins)
        })
        .collect();
    assert_eq!(renamed, synthetic(&after), "a synthetic handle moved");

    // KiCad's own name for the renumbered net does change, which is why it is
    // an attribute and never a handle.
    assert_eq!(
        net(&before, "R21", "2").kicad_name,
        "unconnected-(R21-Pad2)"
    );
    assert_eq!(net(&after, "R22", "2").kicad_name, "unconnected-(R22-Pad2)");
}

#[test]
fn two_runs_over_one_project_agree() {
    let first = fixture_nets();
    let second = fixture_nets();
    let names: Vec<(&str, &str)> = first
        .nets()
        .iter()
        .map(|net| (net.name.as_str(), net.kicad_name.as_str()))
        .collect();
    let again: Vec<(&str, &str)> = second
        .nets()
        .iter()
        .map(|net| (net.name.as_str(), net.kicad_name.as_str()))
        .collect();
    assert_eq!(names, again);
}

#[test]
fn a_power_symbol_value_names_its_net() {
    let nets = fixture_nets();
    let ground = net(&nets, "R1", "2");
    assert_eq!(ground.name, "GND");
    assert!(!ground.synthetic);
    assert_eq!(ground.kicad_name, "GND");
}

#[test]
fn a_label_names_its_net_and_the_global_one_wins() {
    let nets = fixture_nets();

    let global = net(&nets, "R14", "1");
    assert_eq!(global.name, "GLOBAL_A");
    assert_eq!(global.kicad_name, "GLOBAL_A");

    // A local label names the net, and KiCad prefixes it with the sheet path.
    let local = net(&nets, "R1", "1");
    assert_eq!(local.name, "NET_A");
    assert_eq!(local.kicad_name, "/NET_A");

    let in_child = net(&nets, "R101", "1");
    assert_eq!(in_child.name, "CHAN_LOCAL");
    assert_eq!(in_child.kicad_name, "/channel_a/CHAN_LOCAL");

    // A hierarchical label outranks the local label the parent gives the same
    // net, and KiCad picks the other way round.
    let crossing = net(&nets, "R100", "1");
    assert_eq!(crossing.name, "IN");
    assert_eq!(crossing.kicad_name, "/CH_A_IN");
}

#[test]
fn a_net_with_no_label_gets_a_synthetic_handle() {
    let nets = fixture_nets();

    let pair = net(&nets, "R12", "2");
    assert!(pair.synthetic);
    assert!(
        pair.name.starts_with("#n"),
        "a synthetic handle reads {}",
        pair.name
    );
    assert_eq!(pair.kicad_name, "Net-(R12-Pad2)");

    // A pin on its own is unconnected, in KiCad's words as well as in fact.
    let alone = net(&nets, "R11", "1");
    assert!(alone.synthetic);
    assert_eq!(alone.kicad_name, "unconnected-(R11-Pad1)");
}

#[test]
fn the_committed_table_is_reproduced_row_for_row() {
    // The table beside the fixture was derived from KiCad's own netlist: the
    // pin sets, in kicli's order, with KiCad's name for each. Reproducing it
    // exactly checks the partition, the order and both names at once.
    let text =
        std::fs::read_to_string(fixtures().join("nets.partition")).expect("the table is readable");
    let expected: Vec<&str> = text
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .collect();

    let nets = fixture_nets();
    let rendered: Vec<String> = nets
        .nets()
        .iter()
        .map(|net| (pins(net), net))
        .filter(|(listed, _)| !listed.is_empty())
        .map(|(listed, net)| format!("{} = {}", listed.join(" "), net.kicad_name))
        .collect();

    assert_eq!(rendered, expected);
}

#[test]
fn nets_are_ordered_by_pin_count_then_by_pin_list() {
    let nets = fixture_nets();
    let listed: Vec<Vec<String>> = nets.nets().iter().map(pins).collect();
    for pair in listed.windows(2) {
        let (left, right) = (&pair[0], &pair[1]);
        assert!(
            left.len() > right.len() || (left.len() == right.len() && left <= right),
            "{left:?} is not ordered before {right:?}"
        );
    }
}
