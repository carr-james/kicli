//! Geometry alone does not make a netlist.
//!
//! The fixture plants the four shapes that matter: two wires crossing with no
//! junction, the same crossing with one, a pin on a wire's interior with no
//! junction, and the same pin with one. KiCad's own netlist is the oracle for
//! all four.

use kicli::connectivity::{MergeRules, NetPin, Nets, extract_with};
use kicli::model::Hierarchy;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sch/nets/nets.kicad_sch")
}

fn nets() -> Nets {
    let hierarchy = Hierarchy::load(&fixture()).expect("the fixture loads");
    extract_with(&hierarchy, MergeRules::GEOMETRY)
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

fn set(pins: &[&str]) -> Vec<String> {
    pins.iter().map(|pin| (*pin).to_owned()).collect()
}

#[test]
fn wire_geometry_alone_splits_named_nets() {
    let geometry = partition(&nets());

    // Two wires crossing with no junction are two nets. That is what junction
    // dots are for.
    assert!(geometry.contains(&set(&["R3.2", "R4.1"])));
    assert!(geometry.contains(&set(&["R5.2", "R6.1"])));

    // The same crossing with a junction is one net.
    assert!(geometry.contains(&set(&["R10.1", "R7.2", "R8.1", "R9.2"])));

    // Names have not been applied, so the two ends of the labelled net are
    // still apart, and so are the two ends of the global one and of ground.
    assert!(geometry.contains(&set(&["R1.1"])));
    assert!(geometry.contains(&set(&["R2.1"])));
    assert!(geometry.contains(&set(&["R101.2"])));
    assert!(geometry.contains(&set(&["R1.2"])));
    assert!(geometry.contains(&set(&["R2.2"])));
}

#[test]
fn a_pin_on_a_wire_interior_needs_a_junction() {
    let geometry = partition(&nets());

    // R11 pin 1 sits halfway along the wire between R12 and R13. There is no
    // junction, so KiCad leaves it out of that net and so does kicli.
    assert!(geometry.contains(&set(&["R12.2", "R13.1"])));
    assert!(geometry.contains(&set(&["R11.1"])));

    // R15 pin 1 sits halfway along the wire between R16 and R17, and there is
    // a junction. The junction is the whole of the difference.
    assert!(geometry.contains(&set(&["R15.1", "R16.2", "R17.1"])));
}

#[test]
fn a_pin_near_a_wire_does_not_merge_with_it() {
    // R11 pin 2 is one symbol length away from the wire that pin 1 lies on.
    // Nothing about it is near enough to connect.
    let geometry = partition(&nets());
    assert!(geometry.contains(&set(&["R11.2"])));
}

#[test]
fn a_bus_does_not_join_the_wires_that_enter_it() {
    // D0 and D1 reach the same bus through bus entries. They are members of
    // one bundle, not one net.
    let geometry = partition(&nets());
    assert!(geometry.contains(&set(&["R20.1"])));
    assert!(geometry.contains(&set(&["R21.1"])));
}

#[test]
fn a_sheet_placed_twice_has_two_copies_of_its_nets() {
    // The child sheet is drawn once and placed twice. Its wires are therefore
    // two independent conductors, one per placement.
    let geometry = partition(&nets());
    assert!(geometry.contains(&set(&["R100.1"])));
    assert!(geometry.contains(&set(&["R200.1"])));
    assert!(geometry.contains(&set(&["R101.1"])));
    assert!(geometry.contains(&set(&["R201.1"])));
}
