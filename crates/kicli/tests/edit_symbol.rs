//! Moving, turning and mirroring a placed symbol.
//!
//! Every test copies a fixture into a scratch directory and writes there. The
//! committed fixture tree is never written by a test.
//!
//! The live rule check runs `kicad-cli` and is off unless `KICLI_TEST_KICAD_CLI`
//! is set, so the default run needs no KiCad install.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::{Command, Stdio};

use kicli::edit::symbol::{
    Finding, Motion, Options, delete_symbol, mirror_symbol, move_symbol, rotate_symbol,
};
use kicli::geometry::{Angle, GRID, Iu, Point, resolve_pins};
use kicli::model::{
    Mirror, Mutation, Schematic, SheetPath, Symbol, Target, WriteOptions, commit, definition_of,
    read_library, state_before,
};
use kicli::view::snapshot::Snapshot;
use kicli_sexpr::{Doc, changed_line_count};

mod support;

use support::{copy_file, scratch};

/// Read a schematic file into a tree and the objects read from it.
fn read(file: &Path) -> (Doc, Schematic) {
    let source = std::fs::read_to_string(file).expect("the file reads");
    let doc = Doc::parse(&source).expect("the file parses");
    let schematic = Schematic::read(&doc).expect("the file is a schematic");
    (doc, schematic)
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
        .expect("the fixture has that symbol")
}

/// The state to compare a change against, taken before the change.
fn state(doc: &Doc, schematic: &Schematic) -> Snapshot {
    let path = SheetPath::root(schematic.uuid.as_ref().expect("the file has a uuid"));
    state_before(doc, schematic, &path, "before").expect("the state snapshots")
}

/// Write a changed tree over its file, with the invariants run.
fn write(
    doc: &Doc,
    file: &Path,
    project: &Path,
    schematic: &Schematic,
    before: &Snapshot,
) -> Mutation {
    let path = SheetPath::root(schematic.uuid.as_ref().expect("the file has a uuid"));
    commit(
        doc,
        &Target {
            path: file,
            project,
            sheet_path: &path,
            grid: GRID,
            options: WriteOptions::default(),
        },
        before,
        "after",
    )
    .expect("the change is written")
}

/// Every pin of a placed symbol, by number, through the embedded definition.
fn pins_of(doc: &Doc, schematic: &Schematic, symbol: &Symbol) -> BTreeMap<String, Point> {
    let library = read_library(doc, &schematic.library_symbols, schematic.version);
    let definition = definition_of(&library, symbol).expect("the definition is embedded");
    resolve_pins(symbol, definition)
        .into_iter()
        .map(|pin| (pin.number, pin.position))
        .collect()
}

/// The offsets from a symbol's anchor to each of its fields, by field name.
fn field_offsets(symbol: &Symbol) -> BTreeMap<String, Point> {
    symbol
        .fields
        .iter()
        .map(|field| (field.name.clone(), field.at - symbol.at))
        .collect()
}

/// The angle of each field, by field name.
fn field_angles(symbol: &Symbol) -> BTreeMap<String, Angle> {
    symbol
        .fields
        .iter()
        .map(|field| (field.name.clone(), field.angle))
        .collect()
}

#[test]
fn a_moved_symbol_takes_its_fields_with_it() {
    let project = scratch("edit_symbol_move");
    let file = copy_file(&project, "sch/multi_instance/channel.kicad_sch");

    let (mut doc, schematic) = read(&file);
    let symbol = symbol_of(&schematic, "R201");
    let anchor = symbol.at;
    let offsets = field_offsets(symbol);
    let angles = field_angles(symbol);
    assert!(
        offsets.values().any(|offset| *offset != Point::default()),
        "the fixture has a field away from its anchor, or the test proves nothing"
    );

    let step = Point::new(3 * GRID.0, -2 * GRID.0);
    let before = state(&doc, &schematic);
    let edited = move_symbol(&mut doc, symbol, Motion::By(step), GRID, Options::default())
        .expect("the symbol moves");
    assert!(
        edited.findings.is_empty(),
        "a move by whole grid steps snaps nothing"
    );
    let mutation = write(&doc, &file, &project, &schematic, &before);
    assert!(mutation.invariants.passed());
    assert_eq!(
        mutation.delta.lines.len(),
        1,
        "the fields moved with the symbol, so one object changed"
    );

    let (mut doc, moved) = read(&file);
    let symbol = symbol_of(&moved, "R201");
    assert_eq!(symbol.at, anchor + step, "the anchor moved by the step");
    assert_eq!(
        field_offsets(symbol),
        offsets,
        "and every field kept its offset from the anchor"
    );
    assert_eq!(field_angles(symbol), angles, "and its own angle");

    // A turn carries the field positions about the anchor and leaves each
    // field's own angle alone.
    let anchor = symbol.at;
    let before = state(&doc, &moved);
    rotate_symbol(&mut doc, symbol, Angle(90), Options::default()).expect("the symbol turns");
    write(&doc, &file, &project, &moved, &before);

    let (_, turned) = read(&file);
    let symbol = symbol_of(&turned, "R201");
    assert_eq!(symbol.angle, Angle(90));
    assert_eq!(symbol.at, anchor, "a turn leaves the anchor alone");
    let expected: BTreeMap<String, Point> = offsets
        .iter()
        .map(|(name, offset)| {
            (
                name.clone(),
                (anchor + *offset).rotated(anchor, Angle(90)) - anchor,
            )
        })
        .collect();
    assert_eq!(
        field_offsets(symbol),
        expected,
        "each field offset is the turned one"
    );
    assert_eq!(
        field_angles(symbol),
        angles,
        "and each field keeps its own angle"
    );
}

#[test]
fn kept_field_positions_stay_where_they_were() {
    let project = scratch("edit_symbol_keep_fields");
    let file = copy_file(&project, "sch/multi_instance/channel.kicad_sch");

    let (mut doc, schematic) = read(&file);
    let symbol = symbol_of(&schematic, "R201");
    let places: BTreeMap<String, Point> = symbol
        .fields
        .iter()
        .map(|field| (field.name.clone(), field.at))
        .collect();

    let options = Options {
        keep_field_positions: true,
        ..Options::default()
    };
    let before = state(&doc, &schematic);
    move_symbol(
        &mut doc,
        symbol,
        Motion::By(Point::new(GRID.0, GRID.0)),
        GRID,
        options,
    )
    .expect("the symbol moves");
    write(&doc, &file, &project, &schematic, &before);

    let (_, moved) = read(&file);
    let symbol = symbol_of(&moved, "R201");
    let after: BTreeMap<String, Point> = symbol
        .fields
        .iter()
        .map(|field| (field.name.clone(), field.at))
        .collect();
    assert_eq!(after, places, "the fields stayed where they were");
}

/// A sheet whose only symbol lets KiCad place its fields.
const AUTOPLACED: &str = concat!(
    "(kicad_sch (version 20260306) (uuid \"root\") (paper \"A4\")",
    " (symbol (lib_id \"Test:R\") (at 25.4 25.4 0) (unit 1) (dnp no)",
    " (fields_autoplaced yes) (uuid \"one\")",
    " (property \"Reference\" \"R1\" (at 27.94 25.4 0))",
    " (instances (project \"\" (path \"/root\" (reference \"R1\") (unit 1))))))"
);

#[test]
fn carrying_a_field_clears_the_autoplace_flag() {
    let mut doc = Doc::parse(AUTOPLACED).expect("the sheet parses");
    let schematic = Schematic::read(&doc).expect("the sheet is a schematic");
    let symbol = schematic.symbols().next().expect("the sheet has a symbol");

    move_symbol(
        &mut doc,
        symbol,
        Motion::By(Point::new(GRID.0, 0)),
        GRID,
        Options::default(),
    )
    .expect("the symbol moves");
    assert!(
        !doc.emit().contains("fields_autoplaced"),
        "kicli placed the fields, so KiCad must not place them again"
    );

    // The flag survives when the caller keeps the field positions, because
    // kicli then set none of them.
    let mut doc = Doc::parse(AUTOPLACED).expect("the sheet parses");
    let schematic = Schematic::read(&doc).expect("the sheet is a schematic");
    let symbol = schematic.symbols().next().expect("the sheet has a symbol");
    let options = Options {
        keep_field_positions: true,
        ..Options::default()
    };
    rotate_symbol(&mut doc, symbol, Angle(180), options).expect("the symbol turns");
    assert!(doc.emit().contains("fields_autoplaced"));
}

#[test]
fn turning_a_symbol_clears_the_autoplace_flag() {
    for angle in [Angle(90), Angle(180), Angle(270)] {
        let mut doc = Doc::parse(AUTOPLACED).expect("the sheet parses");
        let schematic = Schematic::read(&doc).expect("the sheet is a schematic");
        let symbol = schematic.symbols().next().expect("the sheet has a symbol");
        rotate_symbol(&mut doc, symbol, angle, Options::default()).expect("the symbol turns");
        assert!(!doc.emit().contains("fields_autoplaced"), "{angle}");
    }

    let mut doc = Doc::parse(AUTOPLACED).expect("the sheet parses");
    let schematic = Schematic::read(&doc).expect("the sheet is a schematic");
    let symbol = schematic.symbols().next().expect("the sheet has a symbol");
    mirror_symbol(&mut doc, symbol, Mirror::Y, Options::default()).expect("the symbol mirrors");
    assert!(!doc.emit().contains("fields_autoplaced"));
}

#[test]
fn a_placement_lands_on_the_grid() {
    let project = scratch("edit_symbol_grid");
    let file = copy_file(&project, "sch/multi_instance/channel.kicad_sch");

    // Half a grid step past a grid line, on both axes. Halves round away from
    // zero, so the snap goes to the further line.
    let asked = Point::new(30 * GRID.0 + GRID.0 / 2, 20 * GRID.0 + GRID.0 / 2);
    let snapped = Point::new(31 * GRID.0, 21 * GRID.0);

    let (mut doc, schematic) = read(&file);
    let symbol = symbol_of(&schematic, "R201");
    let before = state(&doc, &schematic);
    let edited = move_symbol(
        &mut doc,
        symbol,
        Motion::To(asked),
        GRID,
        Options::default(),
    )
    .expect("the symbol moves");
    assert_eq!(
        edited.findings,
        vec![Finding::Snapped {
            asked,
            placed: snapped
        }],
        "the report says it snapped"
    );
    write(&doc, &file, &project, &schematic, &before);
    let (_, moved) = read(&file);
    assert_eq!(symbol_of(&moved, "R201").at, snapped);

    // The override places the symbol exactly, and is itself a finding.
    let file = copy_file(&project, "sch/multi_instance/channel.kicad_sch");
    let (mut doc, schematic) = read(&file);
    let symbol = symbol_of(&schematic, "R201");
    let options = Options {
        off_grid: true,
        ..Options::default()
    };
    let before = state(&doc, &schematic);
    let edited =
        move_symbol(&mut doc, symbol, Motion::To(asked), GRID, options).expect("the symbol moves");
    assert_eq!(
        edited.findings,
        vec![Finding::OffGrid { placed: asked }],
        "the override shouts about itself"
    );
    write(&doc, &file, &project, &schematic, &before);
    let (_, moved) = read(&file);
    assert_eq!(
        symbol_of(&moved, "R201").at,
        asked,
        "the symbol is exactly where it was asked for"
    );
}

#[test]
fn a_mirror_reflects_the_pins_about_the_anchor() {
    let project = scratch("edit_symbol_mirror");
    let file = copy_file(&project, "geometry/asymmetric.kicad_sch");

    let (mut doc, schematic) = read(&file);
    let symbol = symbol_of(&schematic, "A1");
    let anchor = symbol.at;
    let before = pins_of(&doc, &schematic, symbol);

    let before_state = state(&doc, &schematic);
    mirror_symbol(&mut doc, symbol, Mirror::X, Options::default()).expect("the symbol mirrors");
    write(&doc, &file, &project, &schematic, &before_state);

    let (doc, mirrored) = read(&file);
    let symbol = symbol_of(&mirrored, "A1");
    let after = pins_of(&doc, &mirrored, symbol);
    let expected: BTreeMap<String, Point> = before
        .iter()
        .map(|(number, at)| {
            (
                number.clone(),
                Point {
                    x: at.x,
                    y: Iu(2 * anchor.y.0 - at.y.0),
                },
            )
        })
        .collect();
    assert_eq!(
        after, expected,
        "a mirror about the X axis reflects every pin about the anchor's own line"
    );
}

#[test]
fn a_symbol_command_changes_only_its_own_lines() {
    let project = scratch("edit_symbol_locality");

    for name in ["move", "rotate", "mirror", "delete"] {
        let file = copy_file(&project, "geometry/asymmetric.kicad_sch");
        let before = std::fs::read_to_string(&file).expect("the fixture reads");
        let (mut doc, schematic) = read(&file);
        let symbol = symbol_of(&schematic, "A1");
        let uuid = symbol.uuid.0.clone();
        let state = state(&doc, &schematic);

        match name {
            "move" => {
                move_symbol(
                    &mut doc,
                    symbol,
                    Motion::By(Point::new(0, -2 * GRID.0)),
                    GRID,
                    Options::default(),
                )
                .expect("the symbol moves");
            }
            "rotate" => {
                rotate_symbol(&mut doc, symbol, Angle(90), Options::default())
                    .expect("the symbol turns");
            }
            "mirror" => {
                mirror_symbol(&mut doc, symbol, Mirror::Y, Options::default())
                    .expect("the symbol mirrors");
            }
            _ => {
                delete_symbol(&mut doc, &schematic, symbol).expect("the symbol goes");
            }
        }
        write(&doc, &file, &project, &schematic, &state);
        let after = std::fs::read_to_string(&file).expect("the written file reads");

        assert_eq!(
            outside_the_block(&before, &uuid),
            outside_the_block(&after, &uuid),
            "{name}: every line outside the symbol is byte-identical"
        );
        // The bound each command documents: the symbol's own lines. A delete
        // takes all of them and the other three stay inside them.
        let bound = block_length(&before, &uuid);
        let changed = changed_line_count(&before, &after);
        assert!(
            changed <= bound,
            "{name}: {changed} lines changed, which is past the symbol's own {bound}"
        );
    }
}

/// How many lines the top-level symbol carrying a uuid occupies.
fn block_length(text: &str, uuid: &str) -> usize {
    text.lines().count() - outside_the_block(text, uuid).len()
}

/// Every line of a file except those of the top-level symbol carrying a uuid.
fn outside_the_block(text: &str, uuid: &str) -> Vec<String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut start = None;
    let mut found = None;
    for (index, line) in lines.iter().enumerate() {
        if *line == "\t(symbol" {
            start = Some(index);
        }
        if line.contains(uuid) && start.is_some() {
            found = start;
            break;
        }
    }
    let Some(start) = found else {
        return lines.iter().map(|line| (*line).to_owned()).collect();
    };
    let end = lines[start..]
        .iter()
        .position(|line| *line == "\t)")
        .map_or(lines.len(), |offset| start + offset + 1);
    lines
        .iter()
        .enumerate()
        .filter(|(index, _)| *index < start || *index >= end)
        .map(|(_, line)| (*line).to_owned())
        .collect()
}

/// The `kicad-cli` binary, when the environment asks for the live tests.
fn kicad_cli() -> Option<String> {
    std::env::var("KICLI_TEST_KICAD_CLI").ok()?;
    Some(std::env::var("KICLI_KICAD_CLI").unwrap_or_else(|_| "kicad-cli".to_owned()))
}

/// Run KiCad's own rule check and read the pin positions out of its report.
fn rule_check(binary: &str, file: &Path) -> BTreeMap<(String, String), Point> {
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
    read_report(&report)
}

/// Read the pin positions out of KiCad's plain-text rule-check report.
///
/// A violation line reads `@(25.40 mm, 21.59 mm): Symbol R1 Pin 1 [Passive, Line]`.
fn read_report(path: &Path) -> BTreeMap<(String, String), Point> {
    let text = std::fs::read_to_string(path).expect("the report reads");
    let mut found = BTreeMap::new();
    for line in text.lines().map(str::trim) {
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
        let (Some(reference), Some("Pin"), Some(number)) =
            (words.next(), words.next(), words.next())
        else {
            continue;
        };
        let (x, y) = position.split_once(", ").expect("two coordinates");
        found.insert(
            (reference.to_owned(), number.to_owned()),
            Point {
                x: millimetres(x),
                y: millimetres(y),
            },
        );
    }
    found
}

/// Read a `12.34 mm` reading as internal units, without going through a float.
fn millimetres(reading: &str) -> Iu {
    Iu::from_millimetres_text(reading.trim_end_matches(" mm")).expect("a coordinate is a number")
}

#[test]
fn a_rotated_symbol_is_where_kicad_draws_it() {
    let Some(binary) = kicad_cli() else {
        eprintln!("skipped: set KICLI_TEST_KICAD_CLI to run the rule check");
        return;
    };
    let project = scratch("edit_symbol_oracle");
    let file = copy_file(&project, "geometry/asymmetric.kicad_sch");

    let (mut doc, schematic) = read(&file);
    let turned = symbol_of(&schematic, "A1");
    let turn_anchor = turned.at;
    let before_turn = pins_of(&doc, &schematic, turned);
    // A5 arrives mirrored, so mirroring it again takes the mirror away. Both
    // directions of the mirror are then covered.
    let flipped = symbol_of(&schematic, "A5");
    let flip_anchor = flipped.at;
    let before_flip = pins_of(&doc, &schematic, flipped);
    assert_eq!(flipped.mirror, Some(Mirror::X));

    let state = state(&doc, &schematic);
    rotate_symbol(&mut doc, turned, Angle(90), Options::default()).expect("the symbol turns");
    mirror_symbol(&mut doc, flipped, Mirror::X, Options::default()).expect("the symbol mirrors");
    write(&doc, &file, &project, &schematic, &state);

    let (doc, after) = read(&file);
    assert_eq!(
        symbol_of(&after, "A5").mirror,
        None,
        "mirroring a mirrored symbol takes the mirror away"
    );

    // What kicli says, what the operation means, and what KiCad draws.
    let mut kicli: BTreeMap<(String, String), Point> = BTreeMap::new();
    let mut predicted: BTreeMap<(String, String), Point> = BTreeMap::new();
    for (reference, at) in pins_of(&doc, &after, symbol_of(&after, "A1")) {
        kicli.insert(("A1".to_owned(), reference), at);
    }
    for (number, at) in &before_turn {
        predicted.insert(
            ("A1".to_owned(), number.clone()),
            at.rotated(turn_anchor, Angle(90)),
        );
    }
    for (reference, at) in pins_of(&doc, &after, symbol_of(&after, "A5")) {
        kicli.insert(("A5".to_owned(), reference), at);
    }
    for (number, at) in &before_flip {
        predicted.insert(
            ("A5".to_owned(), number.clone()),
            Point {
                x: at.x,
                y: Iu(2 * flip_anchor.y.0 - at.y.0),
            },
        );
    }
    assert_eq!(
        kicli, predicted,
        "the turn and the mirror carried every pin about its anchor"
    );

    let kicad: BTreeMap<(String, String), Point> = rule_check(&binary, &file)
        .into_iter()
        .filter(|((reference, _), _)| reference == "A1" || reference == "A5")
        .collect();
    assert_eq!(
        kicli, kicad,
        "KiCad draws the pins where kicli says they are"
    );
    assert!(!kicad.is_empty(), "the rule check reported the pins");
}
