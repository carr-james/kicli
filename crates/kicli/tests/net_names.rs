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

/// Two rails, each with a power flag on it. The flags must not join the rails.
const FLAGGED_RAILS: &str = r##"(kicad_sch
	(version 20260306)
	(uuid "00000000-0000-4000-8000-0f0000000000")
	(lib_symbols
		(symbol "power:+5V"
			(power)
			(symbol "+5V_1_1"
				(pin power_in line
					(at 0 0 90)
					(length 0)
					(name "+5V")
					(number "1")
				)
			)
		)
		(symbol "power:+3V3"
			(power)
			(symbol "+3V3_1_1"
				(pin power_in line
					(at 0 0 90)
					(length 0)
					(name "+3V3")
					(number "1")
				)
			)
		)
		(symbol "power:PWR_FLAG"
			(power)
			(symbol "PWR_FLAG_1_1"
				(pin power_out line
					(at 0 0 90)
					(length 0)
					(name "pwr_flag")
					(number "1")
				)
			)
		)
	)
	(symbol
		(lib_id "power:+5V")
		(at 50.8 50.8 0)
		(uuid "00000000-0000-4000-8000-0f0000000001")
		(property "Reference" "#PWR01" (at 50.8 50.8 0))
		(property "Value" "+5V" (at 50.8 50.8 0))
		(pin "1" (uuid "00000000-0000-4000-8000-0f0000000002"))
		(instances (project "flags" (path "/00000000-0000-4000-8000-0f0000000000" (reference "#PWR01") (unit 1))))
	)
	(symbol
		(lib_id "power:PWR_FLAG")
		(at 50.8 50.8 0)
		(uuid "00000000-0000-4000-8000-0f0000000003")
		(property "Reference" "#FLG01" (at 50.8 50.8 0))
		(property "Value" "PWR_FLAG" (at 50.8 50.8 0))
		(pin "1" (uuid "00000000-0000-4000-8000-0f0000000004"))
		(instances (project "flags" (path "/00000000-0000-4000-8000-0f0000000000" (reference "#FLG01") (unit 1))))
	)
	(symbol
		(lib_id "power:+3V3")
		(at 101.6 50.8 0)
		(uuid "00000000-0000-4000-8000-0f0000000005")
		(property "Reference" "#PWR02" (at 101.6 50.8 0))
		(property "Value" "+3V3" (at 101.6 50.8 0))
		(pin "1" (uuid "00000000-0000-4000-8000-0f0000000006"))
		(instances (project "flags" (path "/00000000-0000-4000-8000-0f0000000000" (reference "#PWR02") (unit 1))))
	)
	(symbol
		(lib_id "power:PWR_FLAG")
		(at 101.6 50.8 0)
		(uuid "00000000-0000-4000-8000-0f0000000007")
		(property "Reference" "#FLG02" (at 101.6 50.8 0))
		(property "Value" "PWR_FLAG" (at 101.6 50.8 0))
		(pin "1" (uuid "00000000-0000-4000-8000-0f0000000008"))
		(instances (project "flags" (path "/00000000-0000-4000-8000-0f0000000000" (reference "#FLG02") (unit 1))))
	)
)
"##;

#[test]
fn a_power_flag_marks_a_net_and_does_not_name_one() {
    // A power symbol names a net through a power INPUT: the rail says "I am
    // +5V". PWR_FLAG's pin is a power OUTPUT, which says only that something
    // drives the net it sits on. Merging power symbols by value without that
    // distinction joins every flagged rail in a project into one net: on
    // KiCad's own CM5 demo it made +5V, M2_3V3, +BATT and GPIO_VREF a single
    // conductor.
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join("flagged_rails");
    std::fs::create_dir_all(&directory).expect("the directory is made");
    std::fs::write(directory.join("flags.kicad_sch"), FLAGGED_RAILS).expect("the sheet writes");
    let hierarchy = Hierarchy::load(&directory.join("flags.kicad_sch")).expect("the project loads");
    let nets = extract_with(&hierarchy, MergeRules::ALL);

    // The rails are two nets, not one. The flags belong to the rail each sits
    // on, and carry no name of their own.
    let sets: Vec<Vec<String>> = nets
        .nets()
        .iter()
        .map(|net| net.pins.iter().map(NetPin::label).collect())
        .collect();
    assert_eq!(sets.len(), 2, "two rails, two nets: {sets:?}");
    assert!(
        sets.iter().all(|set| set.len() == 2),
        "each rail carries its own flag and nothing else: {sets:?}"
    );
    let first: BTreeSet<&String> = sets[0].iter().collect();
    let second: BTreeSet<&String> = sets[1].iter().collect();
    assert!(
        first.is_disjoint(&second),
        "the flags did not join the rails to each other: {sets:?}"
    );
}
