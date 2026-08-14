//! Placing a symbol from a library, and deleting one.
//!
//! Every test copies a fixture into a scratch directory and writes there. The
//! committed fixture tree is never written by a test.
//!
//! The live rule check runs `kicad-cli` and is off unless `KICLI_TEST_KICAD_CLI`
//! is set, so the default run needs no KiCad install.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::{Command, Stdio};

use kicli::edit::symbol::{Instance, Options, Placement, delete_symbol, place_symbol};
use kicli::geometry::{Angle, GRID, Point, resolve_pins};
use kicli::model::{
    Hierarchy, LibId, Mutation, Refdes, Schematic, SheetPath, Symbol, Target, Uuid, WriteOptions,
    commit, definition_of, read_library, state_before,
};
use kicli_sexpr::Doc;

mod support;

use support::{copy_file, scratch};

/// A two-pin part, as a library file writes it.
///
/// Library coordinates are Y-up, so the reference sits above the body and the
/// pins run left and right from the anchor.
const PLACED: &str = concat!(
    "(symbol \"PLACED\"\n",
    "  (pin_names (offset 0))\n",
    "  (exclude_from_sim no) (in_bom yes) (on_board yes) (in_pos_files yes)\n",
    "  (duplicate_pin_numbers_are_jumpers no)\n",
    "  (property \"Reference\" \"U\" (at 0 5.08 0) (show_name no) (do_not_autoplace no)\n",
    "    (effects (font (size 1.27 1.27))))\n",
    "  (property \"Value\" \"PLACED\" (at 0 -5.08 0) (show_name no) (do_not_autoplace no)\n",
    "    (effects (font (size 1.27 1.27))))\n",
    "  (property \"Footprint\" \"\" (at 0 0 0) (show_name no) (do_not_autoplace no) (hide yes)\n",
    "    (effects (font (size 1.27 1.27))))\n",
    "  (symbol \"PLACED_0_1\"\n",
    "    (rectangle (start -2.54 -2.54) (end 2.54 2.54)\n",
    "      (stroke (width 0.254) (type default)) (fill (type none))))\n",
    "  (symbol \"PLACED_1_1\"\n",
    "    (pin passive line (at -7.62 0 0) (length 5.08)\n",
    "      (name \"\" (effects (font (size 1.27 1.27))))\n",
    "      (number \"1\" (effects (font (size 1.27 1.27)))))\n",
    "    (pin passive line (at 7.62 0 180) (length 5.08)\n",
    "      (name \"\" (effects (font (size 1.27 1.27))))\n",
    "      (number \"2\" (effects (font (size 1.27 1.27))))))\n",
    "  (embedded_fonts no))\n"
);

/// Read a schematic file into a tree and the objects read from it.
fn read(file: &Path) -> (Doc, Schematic) {
    let source = std::fs::read_to_string(file).expect("the file reads");
    let doc = Doc::parse(&source).expect("the file parses");
    let schematic = Schematic::read(&doc).expect("the file is a schematic");
    (doc, schematic)
}

/// Write a changed tree over its file, with the invariants run.
fn write(doc: &Doc, file: &Path, project: &Path, schematic: &Schematic) -> Mutation {
    let path = SheetPath::root(schematic.uuid.as_ref().expect("the file has a uuid"));
    let before = state_before(doc, schematic, &path, "before").expect("the state snapshots");
    commit(
        doc,
        &Target {
            path: file,
            project,
            sheet_path: &path,
            grid: GRID,
            options: WriteOptions::default(),
        },
        &before,
        "after",
    )
    .expect("the change is written")
}

/// Identifiers a test can predict, in place of the random ones a run makes.
fn identifiers() -> impl Iterator<Item = Uuid> {
    (0..u32::MAX).map(|index| Uuid(format!("00000000-0000-4000-8000-03{index:010}")))
}

/// The symbol carrying a reference designator.
fn symbol_of<'a>(schematic: &'a Schematic, reference: &str) -> &'a Symbol {
    schematic
        .symbols()
        .find(|symbol| {
            symbol
                .field("Reference")
                .is_some_and(|field| field.value == reference)
        })
        .expect("the file has that symbol")
}

/// A request to place the two-pin part at a position, once.
fn request<'a>(lib_id: &'a LibId, at: Point, instances: &'a [Instance]) -> Placement<'a> {
    Placement {
        lib_id,
        definition: PLACED,
        at,
        angle: Angle(0),
        mirror: None,
        unit: 1,
        body_style: 1,
        value: None,
        instances,
    }
}

#[test]
fn a_placed_symbol_carries_its_definition() {
    let project = scratch("edit_symbol_place");
    let file = copy_file(&project, "geometry/asymmetric.kicad_sch");

    let (mut doc, schematic) = read(&file);
    assert!(
        !schematic
            .library_symbols
            .iter()
            .any(|(name, _)| name == "Test:PLACED"),
        "the definition is not embedded yet, or the test proves nothing"
    );
    let root = SheetPath::root(schematic.uuid.as_ref().expect("the file has a uuid"));
    let lib_id = LibId("Test:PLACED".to_owned());
    let instances = [Instance {
        project: String::new(),
        path: root,
        reference: Refdes("U1".to_owned()),
        unit: 1,
    }];
    let at = Point::new(1_905_000, 762_000);
    let placed = place_symbol(
        &mut doc,
        &schematic,
        &request(&lib_id, at, &instances),
        GRID,
        Options::default(),
        &mut identifiers(),
    )
    .expect("the symbol is placed");
    assert!(placed.findings.is_empty(), "the position is on the grid");

    let mutation = write(&doc, &file, &project, &schematic);
    assert!(
        mutation.invariants.passed(),
        "{:?}",
        mutation.invariants.failures().collect::<Vec<_>>()
    );

    let (doc, after) = read(&file);
    assert!(
        after
            .library_symbols
            .iter()
            .any(|(name, _)| name == "Test:PLACED"),
        "the sheet's lib_symbols gained the definition KiCad draws"
    );

    let symbol = symbol_of(&after, "U1");
    let library = read_library(&doc, &after.library_symbols, after.version);
    let definition =
        definition_of(&library, symbol).expect("the placement resolves through the cache");
    let pins: Vec<Point> = resolve_pins(symbol, definition)
        .into_iter()
        .map(|pin| pin.position)
        .collect();
    assert_eq!(
        pins,
        vec![
            Point::new(1_905_000 - 76_200, 762_000),
            Point::new(1_905_000 + 76_200, 762_000),
        ],
        "the written file's pins are where the geometry says"
    );

    // The placement's fields come from the definition, mapped out of library
    // space, and the reference is the one the instance data names.
    let reference = symbol.field("Reference").expect("it has a reference");
    assert_eq!(reference.value, "U1");
    assert_eq!(
        reference.at,
        Point::new(1_905_000, 762_000 - 50_800),
        "a library field is Y-up and relative to the anchor"
    );
    assert_eq!(
        symbol.field("Value").map(|field| field.value.as_str()),
        Some("PLACED")
    );
}

#[test]
fn a_placement_on_a_twice_placed_sheet_gets_two_references() {
    let project = scratch("edit_symbol_two_paths");
    let root_file = copy_file(&project, "sch/multi_instance/multi_instance.kicad_sch");
    let file = copy_file(&project, "sch/multi_instance/channel.kicad_sch");

    let hierarchy = Hierarchy::load(&root_file).expect("the hierarchy loads");
    let child = hierarchy
        .files
        .iter()
        .position(|loaded| loaded.path.file_name() == file.file_name())
        .expect("the child file is in the tree");
    let paths: Vec<SheetPath> = hierarchy
        .placements
        .iter()
        .filter(|placement| placement.file == child)
        .map(|placement| placement.path.clone())
        .collect();
    assert_eq!(paths.len(), 2, "the sheet is placed twice");

    let (mut doc, schematic) = read(&file);
    let lib_id = LibId("Test:PLACED".to_owned());
    let instances: Vec<Instance> = paths
        .iter()
        .zip(["U201", "U301"])
        .map(|(path, reference)| Instance {
            project: "multi_instance".to_owned(),
            path: path.clone(),
            reference: Refdes(reference.to_owned()),
            unit: 1,
        })
        .collect();
    place_symbol(
        &mut doc,
        &schematic,
        &request(&lib_id, Point::new(762_000, 762_000), &instances),
        GRID,
        Options::default(),
        &mut identifiers(),
    )
    .expect("the symbol is placed");
    let mutation = write(&doc, &file, &project, &schematic);
    assert!(
        mutation.invariants.passed(),
        "{:?}",
        mutation.invariants.failures().collect::<Vec<_>>()
    );

    let (_, after) = read(&file);
    let symbol = symbol_of(&after, "U201");
    assert_eq!(symbol.placements.len(), 2, "one reference per placement");
    assert_eq!(
        symbol
            .reference_on(&paths[0])
            .map(|refdes| refdes.0.clone()),
        Some("U201".to_owned())
    );
    assert_eq!(
        symbol
            .reference_on(&paths[1])
            .map(|refdes| refdes.0.clone()),
        Some("U301".to_owned())
    );

    // The whole project still resolves: no symbol carries instance data for a
    // placement that is not there.
    let hierarchy = Hierarchy::load(&root_file).expect("the hierarchy loads");
    let outcome = kicli::model::check_hierarchy(&hierarchy);
    assert!(outcome.passed(), "{:?}", outcome.faults);
}

#[test]
fn a_deleted_symbol_leaves_no_trace() {
    let project = scratch("edit_symbol_delete");
    let file = copy_file(&project, "geometry/asymmetric.kicad_sch");

    let (mut doc, schematic) = read(&file);
    let symbol = symbol_of(&schematic, "A1");
    let uuid = symbol.uuid.0.clone();
    let pin_uuids: Vec<String> = symbol.pins.iter().map(|pin| pin.uuid.0.clone()).collect();
    delete_symbol(&mut doc, &schematic, symbol).expect("the symbol goes");
    let mutation = write(&doc, &file, &project, &schematic);
    assert!(
        mutation.invariants.passed(),
        "{:?}",
        mutation.invariants.failures().collect::<Vec<_>>()
    );

    let source = std::fs::read_to_string(&file).expect("the written file reads");
    assert!(!source.contains(&uuid), "the symbol is gone");
    for pin in &pin_uuids {
        assert!(!source.contains(pin), "and so is its pin data");
    }
    let (_, after) = read(&file);
    assert!(after.symbols().all(|other| other.uuid.0 != uuid));
    assert!(
        after.symbols().all(|other| !other
            .placements
            .iter()
            .any(|place| place.reference.0 == "A1")),
        "the instance data went with the symbol"
    );
    assert!(
        after
            .library_symbols
            .iter()
            .any(|(name, _)| name == "Test:ASYM"),
        "seven placements still draw through the definition, so it stays"
    );

    // The last placement takes the definition with it.
    let file = copy_file(&project, "sch/multi_instance/channel.kicad_sch");
    let (mut doc, schematic) = read(&file);
    let only = symbol_of(&schematic, "R201");
    delete_symbol(&mut doc, &schematic, only).expect("the symbol goes");
    let mutation = write(&doc, &file, &project, &schematic);
    assert!(mutation.invariants.passed());

    let (_, empty) = read(&file);
    assert_eq!(empty.symbols().count(), 0);
    assert!(
        empty.library_symbols.is_empty(),
        "no placement draws through the definition any more"
    );
}

/// The `kicad-cli` binary, when the environment asks for the live tests.
fn kicad_cli() -> Option<String> {
    std::env::var("KICLI_TEST_KICAD_CLI").ok()?;
    Some(std::env::var("KICLI_KICAD_CLI").unwrap_or_else(|_| "kicad-cli".to_owned()))
}

/// Run KiCad's own rule check and hand back the report it wrote.
fn rule_check(binary: &str, file: &Path) -> String {
    let directory = file.parent().expect("the file sits in a directory");
    let report = directory.join("rule-check.txt");
    let status = Command::new(binary)
        .current_dir(directory)
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
        .arg(&report)
        .arg(file.file_name().expect("the file has a name"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("kicad-cli runs");
    assert!(status.success(), "the rule check ran");
    std::fs::read_to_string(&report).expect("the report reads")
}

/// The kinds of violation a report carries, such as `pin_not_connected`.
fn violation_kinds(report: &str) -> BTreeSet<String> {
    report
        .lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix('['))
        .filter_map(|line| line.split_once(']'))
        .map(|(kind, _)| kind.to_owned())
        .collect()
}

/// The pin positions of one symbol, as the report gives them.
fn reported_pins(report: &str, reference: &str) -> Vec<(String, Point)> {
    let mut found = Vec::new();
    for line in report.lines().map(str::trim) {
        let Some(rest) = line.strip_prefix("@(") else {
            continue;
        };
        let Some((position, description)) = rest.split_once("): ") else {
            continue;
        };
        let Some(symbol) = description.strip_prefix("Symbol ") else {
            continue;
        };
        let mut words = symbol.split_whitespace();
        let (Some(found_reference), Some("Pin"), Some(number)) =
            (words.next(), words.next(), words.next())
        else {
            continue;
        };
        if found_reference != reference {
            continue;
        }
        let (x, y) = position.split_once(", ").expect("two coordinates");
        found.push((
            number.to_owned(),
            Point {
                x: millimetres(x),
                y: millimetres(y),
            },
        ));
    }
    found.sort();
    found
}

/// Read a `12.34 mm` reading as internal units, without going through a float.
fn millimetres(reading: &str) -> kicli::geometry::Iu {
    kicli::geometry::Iu::from_millimetres_text(reading.trim_end_matches(" mm"))
        .expect("a coordinate is a number")
}

#[test]
fn kicad_reads_what_place_wrote() {
    let Some(binary) = kicad_cli() else {
        eprintln!("skipped: set KICLI_TEST_KICAD_CLI to run the rule check");
        return;
    };
    let project = scratch("edit_symbol_place_oracle");
    let file = copy_file(&project, "geometry/asymmetric.kicad_sch");
    let before = violation_kinds(&rule_check(&binary, &file));

    let (mut doc, schematic) = read(&file);
    let root = SheetPath::root(schematic.uuid.as_ref().expect("the file has a uuid"));
    let lib_id = LibId("Test:PLACED".to_owned());
    let instances = [Instance {
        project: String::new(),
        path: root,
        reference: Refdes("U1".to_owned()),
        unit: 1,
    }];
    let at = Point::new(1_905_000, 762_000);
    place_symbol(
        &mut doc,
        &schematic,
        &request(&lib_id, at, &instances),
        GRID,
        Options::default(),
        &mut identifiers(),
    )
    .expect("the symbol is placed");
    write(&doc, &file, &project, &schematic);

    let (doc, after) = read(&file);
    let symbol = symbol_of(&after, "U1");
    let library = read_library(&doc, &after.library_symbols, after.version);
    let definition = definition_of(&library, symbol).expect("the definition is embedded");
    let mut kicli: Vec<(String, Point)> = resolve_pins(symbol, definition)
        .into_iter()
        .map(|pin| (pin.number, pin.position))
        .collect();
    kicli.sort();

    let report = rule_check(&binary, &file);
    assert_eq!(
        reported_pins(&report, "U1"),
        kicli,
        "KiCad reports the new symbol's pins where kicli says they are"
    );
    assert_eq!(
        violation_kinds(&report),
        before,
        "and the placement brought no fault of its own"
    );
}
