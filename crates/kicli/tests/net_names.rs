//! Names are part of connectivity, not decoration.
//!
//! A ground symbol is joined to every other ground symbol by its value, and a
//! sheet placed twice keeps its local names apart while it shares its global
//! ones. Geometry alone gets none of that right, so the committed partition is
//! the check.

use kicli::connectivity::{MergeRules, NetPin, Nets, extract_with};
use kicli::model::Hierarchy;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sch/nets")
        .join(name)
}

fn extract(rules: MergeRules) -> Nets {
    let hierarchy = Hierarchy::load(&fixture("nets.kicad_sch")).expect("the fixture loads");
    extract_with(&hierarchy, rules)
}

/// The pin sets, the way a netlist reports them: power symbols left out.
fn partition(nets: &Nets) -> BTreeSet<Vec<String>> {
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

/// The committed table, which was derived from KiCad's own netlist.
fn committed() -> BTreeSet<Vec<String>> {
    let text = std::fs::read_to_string(fixture("nets.partition")).expect("the table is readable");
    text.lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .map(|line| {
            let pins = line.split(" = ").next().expect("every row has a pin list");
            pins.split_whitespace().map(str::to_owned).collect()
        })
        .collect()
}

fn set(pins: &[&str]) -> Vec<String> {
    pins.iter().map(|pin| (*pin).to_owned()).collect()
}

#[test]
fn names_merge_nets_across_the_hierarchy() {
    let full = partition(&extract(MergeRules::ALL));
    let expected = committed();

    let missing: Vec<&Vec<String>> = expected.difference(&full).collect();
    let extra: Vec<&Vec<String>> = full.difference(&expected).collect();
    assert!(
        missing.is_empty() && extra.is_empty(),
        "nets kicli did not find: {missing:?}\nnets kicli invented: {extra:?}"
    );
}

#[test]
fn the_power_rule_is_what_makes_ground_one_net() {
    let full = extract(MergeRules::ALL);
    let without = extract(MergeRules {
        power: false,
        ..MergeRules::ALL
    });
    assert!(without.nets().len() > full.nets().len());

    // With the rule, the four ground symbols are one net across two sheets
    // and two placements of one of them.
    let ground = partition(&full);
    assert!(ground.contains(&set(&["R1.2", "R100.2", "R2.2", "R200.2"])));
}

#[test]
fn a_local_label_is_local_to_one_placement_of_its_sheet() {
    // The child sheet is placed twice and carries one local label. That is two
    // nets, one per placement.
    let full = partition(&extract(MergeRules::ALL));
    assert!(full.contains(&set(&["R101.1"])));
    assert!(full.contains(&set(&["R201.1"])));

    // Its global label is one net across both placements and the root sheet.
    assert!(full.contains(&set(&["R101.2", "R14.1", "R201.2"])));
}

#[test]
fn a_hierarchical_label_meets_the_sheet_pin_above_it() {
    // The child's hierarchical label reaches the root through the sheet pin,
    // where a local label names it. Each placement gets its own net.
    let full = partition(&extract(MergeRules::ALL));
    assert!(full.contains(&set(&["R100.1"])));
    assert!(full.contains(&set(&["R200.1"])));

    let labels_only = partition(&extract(MergeRules {
        power: false,
        ..MergeRules::ALL
    }));
    assert!(labels_only.contains(&set(&["R100.1"])));
}
