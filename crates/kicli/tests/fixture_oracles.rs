//! The fixtures agree with KiCad's own answers about them.
//!
//! Two of the M2 fixtures carry an oracle: a file KiCad wrote about the fixture
//! beside it. The geometry fixtures carry KiCad's electrical rule check, which
//! reports the position of every unconnected pin, and the connectivity fixture
//! carries KiCad's netlist. These tests keep the fixture, the prediction and
//! KiCad's answer in step.
//!
//! The default run reads the committed oracles only, so it needs no KiCad. The
//! regeneration test runs `kicad-cli` and is off unless `KICLI_TEST_KICAD_CLI`
//! is set.

use kicli::geometry::{Iu, Point};
use kicli_probe::oracle::{Kicad, Report, without_the_run_specific_lines};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Internal units per millimetre in a schematic.
const UNITS_PER_MM: f64 = 10_000.0;

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Read a `.expected` table, keyed by `(refdes, pin number)` and by pin uuid.
fn read_expected(path: &Path) -> (BTreeMap<(String, String), Point>, BTreeMap<String, Point>) {
    let text = std::fs::read_to_string(path).expect("expected table is readable");
    let mut by_pin = BTreeMap::new();
    let mut by_uuid = BTreeMap::new();
    for line in text.lines().filter(|line| !line.starts_with('#')) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(fields.len(), 7, "row needs seven fields: {line}");
        let (x, y) = (
            fields[4].parse().expect("x is an integer"),
            fields[5].parse().expect("y is an integer"),
        );
        let at = Point { x: Iu(x), y: Iu(y) };
        by_pin.insert((fields[0].to_owned(), fields[3].to_owned()), at);
        by_uuid.insert(fields[6].to_owned(), at);
    }
    (by_pin, by_uuid)
}

#[test]
fn predicted_pin_positions_match_the_rule_check() {
    let root = fixture_root().join("geometry");
    for fixture in ["orientations", "asymmetric"] {
        let (expected, _by_uuid) = read_expected(&root.join(format!("{fixture}.expected")));
        let measured = Report::read(&root.join(format!("{fixture}.erc.txt"))).pin_positions();
        assert_eq!(
            expected, measured,
            "{fixture}: the predicted pin positions are KiCad's own"
        );
        assert!(
            !expected.is_empty(),
            "{fixture}: the table has rows to compare"
        );
    }
}

#[test]
fn the_rule_check_json_reports_coordinates_a_hundred_times_small() {
    // KiCad 10.0.5 builds the JSON exporter's units provider with the board
    // scale of 1e6 units per millimetre instead of the schematic scale of 1e4,
    // so schematic coordinates come out 100x too small while the file still
    // says "mm". The plain-text report is correct. See
    // eeschema/erc/erc_report.cpp:161 against :63.
    //
    // This test expects the bug. When KiCad fixes it, this test fails, and the
    // correction elsewhere in kicli must be removed rather than doubled.
    let root = fixture_root().join("geometry");
    for fixture in ["orientations", "asymmetric"] {
        let (_by_pin, by_uuid) = read_expected(&root.join(format!("{fixture}.expected")));
        let text = std::fs::read_to_string(root.join(format!("{fixture}.erc.json")))
            .expect("report is readable");
        let report: serde_json::Value = serde_json::from_str(&text).expect("report is JSON");
        assert_eq!(
            report["coordinate_units"], "mm",
            "the file claims millimetres"
        );

        let mut compared = 0;
        for sheet in report["sheets"].as_array().expect("sheets is a list") {
            for violation in sheet["violations"]
                .as_array()
                .expect("violations is a list")
            {
                for item in violation["items"].as_array().expect("items is a list") {
                    let uuid = item["uuid"].as_str().expect("item carries a uuid");
                    let Some(pin) = by_uuid.get(uuid) else {
                        continue;
                    };
                    let x = item["pos"]["x"].as_f64().expect("x is a number");
                    let y = item["pos"]["y"].as_f64().expect("y is a number");
                    assert_eq!(
                        Point {
                            x: millimetres_to_units(x * 100.0),
                            y: millimetres_to_units(y * 100.0),
                        },
                        *pin,
                        "{fixture}: JSON coordinate times 100 is the text coordinate"
                    );
                    compared += 1;
                }
            }
        }
        assert!(
            compared > 0,
            "{fixture}: the JSON report has pins to compare"
        );
    }
}

/// Convert a reading in millimetres to internal units.
///
/// The JSON canary below reads a float, because the number it checks is one
/// KiCad computed with the wrong scale. Every other reading in this file is an
/// exact integer.
fn millimetres_to_units(value: f64) -> Iu {
    // The reading has four decimals at most, so this rounds exactly.
    #[allow(clippy::cast_possible_truncation, reason = "schematic units are int32")]
    Iu((value * UNITS_PER_MM).round() as i32)
}

/// Copy a fixture directory's files into a scratch directory.
fn copy_fixture_files(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("scratch directory is writable");
    for entry in std::fs::read_dir(from).expect("fixture directory reads") {
        let path = entry.expect("directory entry reads").path();
        if path.is_file() {
            let name = path.file_name().expect("a file has a name");
            std::fs::copy(&path, to.join(name)).expect("copy succeeds");
        }
    }
}

#[test]
fn oracles_are_current() {
    let Some(tool) = Kicad::found_or_skip("regenerate the oracles") else {
        return;
    };
    let root = fixture_root();
    let scratch = std::env::temp_dir().join("kicli-oracle-check");
    std::fs::create_dir_all(&scratch).expect("scratch directory is writable");

    // KiCad writes a .kicad_prl beside any project it opens, so the fixtures
    // are copied out and the tool runs on the copies. The fixture tree stays
    // exactly as committed.
    let geometry = scratch.join("geometry");
    copy_fixture_files(&root.join("geometry"), &geometry);
    let nets = scratch.join("nets");
    copy_fixture_files(&root.join("sch/nets"), &nets);

    for fixture in ["orientations", "asymmetric"] {
        let sheet = geometry.join(format!("{fixture}.kicad_sch"));
        let regenerated = tool
            .rule_check_into(&sheet, &scratch.join(format!("{fixture}.erc.txt")))
            .text()
            .to_owned();
        let committed =
            std::fs::read_to_string(root.join("geometry").join(format!("{fixture}.erc.txt")))
                .expect("committed report reads");
        assert_eq!(
            without_the_run_specific_lines(&committed),
            without_the_run_specific_lines(&regenerated),
            "{fixture}: the committed rule-check oracle is current"
        );
    }

    let regenerated = tool
        .netlist(&nets.join("nets.kicad_sch"), &scratch.join("nets.netlist"))
        .text()
        .to_owned();
    let committed =
        std::fs::read_to_string(root.join("sch/nets/nets.netlist")).expect("oracle reads");
    assert_eq!(
        without_the_run_specific_lines(&committed),
        without_the_run_specific_lines(&regenerated),
        "the committed netlist oracle is current"
    );
}

#[test]
fn the_net_partition_oracle_covers_every_pin_of_the_fixture() {
    let root = fixture_root().join("sch/nets");
    let partition = std::fs::read_to_string(root.join("nets.partition")).expect("partition reads");
    let mut nets = 0;
    let mut pins = Vec::new();
    for line in partition.lines().filter(|line| !line.starts_with('#')) {
        let (set, _kicad_name) = line.split_once(" = ").expect("a net names KiCad's name");
        nets += 1;
        pins.extend(set.split_whitespace().map(str::to_owned));
    }
    let unique: std::collections::BTreeSet<&String> = pins.iter().collect();
    assert_eq!(unique.len(), pins.len(), "a pin belongs to one net only");
    assert!(nets > 1, "the fixture has more than one net");

    // The two clusters that differ only by a junction are the point of the
    // fixture: a pin on a wire's interior connects only when a junction sits
    // there. See research/notes/pin-on-wire-interior.md.
    let has = |set: &str| partition.lines().any(|line| line.starts_with(set));
    assert!(
        has("R12.2 R13.1 ="),
        "no junction, so the mid-span pin stays out"
    );
    assert!(
        has("R15.1 R16.2 R17.1 ="),
        "a junction takes the mid-span pin in"
    );
}
