//! A field can be hidden, whoever owns it, in the form its file uses.
//!
//! `hide` moved. It sits inside `effects` in a file stamped before 20251028
//! and beside `show_name` after it. Writing the wrong one for the file is a
//! silent corruption: KiCad reads the field as visible and draws it again.

use kicli::edit::field::{FieldAddress, hide, show};
use kicli::geometry::GRID;
use kicli::model::{Field, Item, Schematic, SheetPath, Target, Uuid, WriteOptions};
use kicli_sexpr::Doc;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The objects of the fixture that own fields, one field each.
const OWNERS: [(&str, &str, &str); 4] = [
    ("symbol", "00000000-0000-4000-8000-000000000020", "Value"),
    ("sheet", "00000000-0000-4000-8000-000000000030", "Sheetname"),
    (
        "global label",
        "00000000-0000-4000-8000-00000000001b",
        "Intersheetrefs",
    ),
    (
        "netclass flag",
        "00000000-0000-4000-8000-00000000001d",
        "Netclass",
    ),
];

/// The sheet path of the modern fixture, which is a root sheet.
const ROOT: &str = "/00000000-0000-4000-8000-000000000002";

/// The sheet path of the older fixture.
const LEGACY_ROOT: &str = "/00000000-0000-4000-8000-0000000000e0";

/// The symbol of the older fixture, and its visible field.
const LEGACY_SYMBOL: &str = "00000000-0000-4000-8000-0000000000e1";

/// Copy a fixture into a scratch directory and hand back the copy.
///
/// A test never writes inside the committed fixture tree.
fn scratch_copy(name: &str, fixture: &str) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("the scratch directory is writable");
    let from = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sch")
        .join(fixture);
    let copy = directory.join(fixture);
    std::fs::copy(from, &copy).expect("the copy is writable");
    copy
}

fn read(path: &Path) -> Doc {
    Doc::parse(&std::fs::read_to_string(path).expect("the file reads")).expect("it parses")
}

fn address(owner: &str, field: &str) -> FieldAddress {
    FieldAddress {
        owner: Uuid(owner.to_owned()),
        name: field.to_owned(),
    }
}

fn target<'a>(file: &'a Path, project: &'a Path, sheet_path: &'a SheetPath) -> Target<'a> {
    Target {
        path: file,
        project,
        sheet_path,
        grid: GRID,
        options: WriteOptions::default(),
    }
}

/// Is the field hidden, as the written file reads back?
fn is_hidden(file: &Path, address: &FieldAddress) -> bool {
    let doc = read(file);
    let schematic = Schematic::read(&doc).expect("the file reads as a schematic");
    let item = schematic
        .items
        .iter()
        .find(|item| item.uuid() == Some(&address.owner))
        .expect("the owner is there");
    fields_of(item)
        .iter()
        .find(|field| field.name == address.name)
        .expect("the field is there")
        .hidden
}

/// The fields of an object, or nothing when it owns none.
fn fields_of(item: &Item) -> &[Field] {
    match item {
        Item::Symbol(symbol) => &symbol.fields,
        Item::Label(label) => &label.fields,
        Item::Sheet(sheet) => &sheet.fields,
        _ => &[],
    }
}

/// The lines of one property, as the file now holds them.
///
/// The name is not enough to find it: an embedded library symbol carries a
/// property of the same name, and it comes first in the file. The caller names
/// the value as well, which the placement and the definition do not share.
fn field_block<'a>(text: &'a str, start: &str) -> Vec<&'a str> {
    let mut lines = Vec::new();
    let mut depth = 0_usize;
    for line in text.lines() {
        if line.trim_start().starts_with(start) {
            depth = 1;
        }
        if depth == 0 {
            continue;
        }
        lines.push(line);
        depth += line.matches('(').count();
        depth -= line.matches(')').count().min(depth);
        if depth <= 1 && line.trim_start().starts_with(')') {
            break;
        }
    }
    assert!(!lines.is_empty(), "the file still holds {start}");
    lines
}

/// Where a token first appears in a block of lines.
fn line_with(block: &[&str], token: &str) -> Option<usize> {
    block.iter().position(|line| line.trim() == token)
}

#[test]
fn hiding_a_field_writes_the_form_the_file_uses() {
    // The current form: hide is a child of the property, beside show_name.
    let file = scratch_copy("edit_field_hide_current", "item_zoo.kicad_sch");
    let project = file.parent().expect("the copy has a directory").to_owned();
    let path = SheetPath(ROOT.to_owned());
    let address = address(OWNERS[0].1, OWNERS[0].2);
    let mut doc = read(&file);
    let mutation = hide(
        &mut doc,
        &target(&file, &project, &path),
        &address,
        "2026-01-02T03:04:05Z",
    )
    .expect("the field is hidden");
    assert!(
        mutation.invariants.passed(),
        "every invariant held: {:?}",
        mutation.invariants.failures().collect::<Vec<_>>()
    );

    let text = std::fs::read_to_string(&file).expect("the file reads");
    let block = field_block(&text, "(property \"Value\" \"10k\"");
    let flag = line_with(&block, "(hide yes)").expect("the file carries the flag");
    let effects = line_with(&block, "(effects").expect("the field has effects");
    assert!(
        flag < effects,
        "the flag sits beside show_name, not inside effects: {block:?}"
    );
    assert!(is_hidden(&file, &address), "and it reads back as hidden");

    // The older form: hide is a child of effects.
    let legacy = scratch_copy("edit_field_hide_legacy", "v9_legacy.kicad_sch");
    let project = legacy
        .parent()
        .expect("the copy has a directory")
        .to_owned();
    let path = SheetPath(LEGACY_ROOT.to_owned());
    let address = FieldAddress {
        owner: Uuid(LEGACY_SYMBOL.to_owned()),
        name: "Reference".to_owned(),
    };
    let mut doc = read(&legacy);
    hide(
        &mut doc,
        &target(&legacy, &project, &path),
        &address,
        "2026-01-02T03:04:05Z",
    )
    .expect("the field is hidden");

    let text = std::fs::read_to_string(&legacy).expect("the file reads");
    let block = field_block(&text, "(property \"Reference\" \"R1\"");
    let flag = line_with(&block, "(hide yes)").expect("the file carries the flag");
    let effects = line_with(&block, "(effects").expect("the field has effects");
    assert!(
        effects < flag,
        "the flag sits inside effects, as this stamp writes it: {block:?}"
    );
    assert!(is_hidden(&legacy, &address), "and it reads back as hidden");
}

#[test]
fn showing_a_field_takes_the_flag_away() {
    let file = scratch_copy("edit_field_show", "item_zoo.kicad_sch");
    let project = file.parent().expect("the copy has a directory").to_owned();
    let path = SheetPath(ROOT.to_owned());
    let address = address(OWNERS[2].1, OWNERS[2].2);
    assert!(is_hidden(&file, &address), "the fixture starts hidden");

    let mut doc = read(&file);
    show(
        &mut doc,
        &target(&file, &project, &path),
        &address,
        "2026-01-02T03:04:05Z",
    )
    .expect("the field is shown");

    let text = std::fs::read_to_string(&file).expect("the file reads");
    let block = field_block(&text, "(property \"Intersheetrefs\"");
    assert!(
        line_with(&block, "(hide yes)").is_none(),
        "KiCad writes no flag for a visible field: {block:?}"
    );
    assert!(!is_hidden(&file, &address), "and it reads back as visible");
}

#[test]
fn every_object_that_owns_fields_can_hide_one() {
    for (what, owner, field) in OWNERS {
        let file = scratch_copy(&format!("edit_field_hide_{owner}"), "item_zoo.kicad_sch");
        let project = file.parent().expect("the copy has a directory").to_owned();
        let path = SheetPath(ROOT.to_owned());
        let address = address(owner, field);
        let taken = "2026-01-02T03:04:05Z";

        let mut doc = read(&file);
        show(&mut doc, &target(&file, &project, &path), &address, taken)
            .expect("the field is shown");
        assert!(!is_hidden(&file, &address), "{what}: {field} is visible");

        let mut doc = read(&file);
        let mutation = hide(&mut doc, &target(&file, &project, &path), &address, taken)
            .expect("the field is hidden");
        assert!(
            mutation.invariants.passed(),
            "{what}: every invariant held: {:?}",
            mutation.invariants.failures().collect::<Vec<_>>()
        );
        assert!(is_hidden(&file, &address), "{what}: {field} is hidden");
    }
}

/// The `kicad-cli` to run, or nothing when the caller did not ask for it.
fn kicad_cli() -> Option<String> {
    std::env::var("KICLI_TEST_KICAD_CLI").ok()?;
    Some(std::env::var("KICLI_KICAD_CLI").unwrap_or_else(|_| "kicad-cli".to_owned()))
}

/// How often KiCad draws a piece of text on a sheet.
///
/// The tool's own output is dropped: the first run on a machine prints
/// fontconfig warnings that say nothing about the drawing.
fn times_drawn(tool: &str, file: &Path, text: &str) -> usize {
    let into = file.with_extension("svg-out");
    let status = Command::new(tool)
        .args(["sch", "export", "svg", "-o"])
        .arg(&into)
        .arg(file)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("kicad-cli runs");
    assert!(status.success(), "kicad-cli exported the sheet");

    let stem = file.file_stem().expect("the file has a name");
    let drawing = std::fs::read_to_string(into.join(stem).with_extension("svg"))
        .expect("the drawing is readable");
    drawing.matches(text).count()
}

#[test]
fn kicad_draws_no_hidden_field() {
    let Some(tool) = kicad_cli() else {
        eprintln!("skipped: set KICLI_TEST_KICAD_CLI to ask KiCad about the written file");
        return;
    };
    // One file of each form: the stamp decides where `hide` goes, and the
    // wrong place leaves the field drawn.
    for (fixture, owner, field, drawn, sheet_path) in [
        ("item_zoo.kicad_sch", OWNERS[0].1, OWNERS[0].2, "10k", ROOT),
        (
            "v9_legacy.kicad_sch",
            LEGACY_SYMBOL,
            "Reference",
            "R1",
            LEGACY_ROOT,
        ),
    ] {
        let file = scratch_copy(&format!("edit_field_hide_oracle_{drawn}"), fixture);
        let project = file.parent().expect("the copy has a directory").to_owned();
        let path = SheetPath(sheet_path.to_owned());
        let address = address(owner, field);

        assert!(
            times_drawn(&tool, &file, drawn) > 0,
            "KiCad draws {drawn} while the field is visible"
        );

        let mut doc = read(&file);
        hide(
            &mut doc,
            &target(&file, &project, &path),
            &address,
            "2026-01-02T03:04:05Z",
        )
        .expect("the field is hidden");

        assert_eq!(
            times_drawn(&tool, &file, drawn),
            0,
            "and none once kicli hid it, in the form {fixture} uses"
        );
    }
}
