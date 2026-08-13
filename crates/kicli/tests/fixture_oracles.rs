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

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Internal units per millimetre in a schematic.
const UNITS_PER_MM: f64 = 10_000.0;

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// One row of a `.expected` table: where a pin should be, in integer units.
#[derive(Debug, PartialEq, Eq)]
struct Pin {
    x: i32,
    y: i32,
}

/// Read a `.expected` table, keyed by `(refdes, pin number)` and by pin uuid.
fn read_expected(path: &Path) -> (BTreeMap<(String, String), Pin>, BTreeMap<String, Pin>) {
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
        by_pin.insert((fields[0].to_owned(), fields[3].to_owned()), Pin { x, y });
        by_uuid.insert(fields[6].to_owned(), Pin { x, y });
    }
    (by_pin, by_uuid)
}

/// Read the pin positions out of KiCad's plain-text rule-check report.
///
/// A violation line reads `@(25.40 mm, 21.59 mm): Symbol R1 Pin 1 [Passive, Line]`.
fn read_text_report(path: &Path) -> BTreeMap<(String, String), Pin> {
    let text = std::fs::read_to_string(path).expect("report is readable");
    let mut found = BTreeMap::new();
    for line in text.lines().map(str::trim) {
        let Some(rest) = line.strip_prefix("@(") else {
            continue;
        };
        let Some((position, description)) = rest.split_once("): ") else {
            continue;
        };
        let Some(refdes) = description.strip_prefix("Symbol ") else {
            continue;
        };
        let mut words = refdes.split_whitespace();
        let (Some(refdes), Some("Pin"), Some(number)) = (words.next(), words.next(), words.next())
        else {
            continue;
        };
        let (x, y) = position.split_once(", ").expect("two coordinates");
        found.insert(
            (refdes.to_owned(), number.to_owned()),
            Pin {
                x: millimetres_to_units(x),
                y: millimetres_to_units(y),
            },
        );
    }
    found
}

/// Convert a `12.34 mm` reading to integer internal units.
fn millimetres_to_units(reading: &str) -> i32 {
    let value: f64 = reading
        .trim_end_matches(" mm")
        .parse()
        .expect("coordinate is a number");
    // The reading has four decimals at most, so this rounds exactly.
    #[allow(clippy::cast_possible_truncation)] // schematic coordinates are int32
    let units = (value * UNITS_PER_MM).round() as i32;
    units
}

#[test]
fn predicted_pin_positions_match_the_rule_check() {
    let root = fixture_root().join("geometry");
    for fixture in ["orientations", "asymmetric"] {
        let (expected, _by_uuid) = read_expected(&root.join(format!("{fixture}.expected")));
        let measured = read_text_report(&root.join(format!("{fixture}.erc.txt")));
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
                        Pin {
                            x: millimetres_to_units(&format!("{}", x * 100.0)),
                            y: millimetres_to_units(&format!("{}", y * 100.0)),
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

/// The `kicad-cli` binary, when the environment asks for the live tests.
///
/// Returns `None` unless `KICLI_TEST_KICAD_CLI` is set, so the default run
/// needs no KiCad install.
fn kicad_cli() -> Option<String> {
    std::env::var("KICLI_TEST_KICAD_CLI").ok()?;
    Some(std::env::var("KICLI_KICAD_CLI").unwrap_or_else(|_| "kicad-cli".to_owned()))
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

/// Drop the lines that carry a timestamp or the caller's own path.
fn without_the_run_specific_lines(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim_end)
        .filter(|line| {
            let trimmed = line.trim_start();
            !trimmed.starts_with("ERC report (")
                && !trimmed.starts_with("(date ")
                && !trimmed.starts_with("(source ")
                && !trimmed.starts_with("(tool ")
        })
        .collect()
}

#[test]
fn oracles_are_current() {
    let Some(binary) = kicad_cli() else {
        eprintln!("skipped: set KICLI_TEST_KICAD_CLI to regenerate the oracles");
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
        let directory = geometry.clone();
        let fresh = scratch.join(format!("{fixture}.erc.txt"));
        let status = std::process::Command::new(&binary)
            .current_dir(&directory)
            .args([
                "sch",
                "erc",
                "--format",
                "report",
                "--units",
                "mm",
                "--severity-all",
                "-o",
            ])
            .arg(&fresh)
            .arg(format!("{fixture}.kicad_sch"))
            .status()
            .expect("kicad-cli runs");
        assert!(status.success(), "{fixture}: the rule check ran");

        let committed =
            std::fs::read_to_string(root.join("geometry").join(format!("{fixture}.erc.txt")))
                .expect("committed report reads");
        let regenerated = std::fs::read_to_string(&fresh).expect("fresh report reads");
        assert_eq!(
            without_the_run_specific_lines(&committed),
            without_the_run_specific_lines(&regenerated),
            "{fixture}: the committed rule-check oracle is current"
        );
    }

    let fresh = scratch.join("nets.netlist");
    let status = std::process::Command::new(&binary)
        .current_dir(&nets)
        .args(["sch", "export", "netlist", "-o"])
        .arg(&fresh)
        .arg("nets.kicad_sch")
        .status()
        .expect("kicad-cli runs");
    assert!(status.success(), "the netlist export ran");
    let committed =
        std::fs::read_to_string(root.join("sch/nets/nets.netlist")).expect("oracle reads");
    let regenerated = std::fs::read_to_string(&fresh).expect("fresh netlist reads");
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
