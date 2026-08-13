//! A snapshot hash covers what an object means, not where it sits in the file.
//!
//! KiCad rewrites the item order of a schematic on every save. A hash over file
//! bytes would therefore report a total rewrite after a one-symbol edit. The
//! tests below pin the two properties that make the snapshot useful: the hash
//! ignores item order, and it separates position from content.

use kicli::model::{Schematic, SheetPath};
use kicli::view::snapshot::Snapshot;
use kicli_sexpr::Doc;
use std::path::{Path, PathBuf};

/// The symbol of the item fixture, and the fields it owns.
const SYMBOL: &str = "00000000-0000-4000-8000-000000000020";

/// A timestamp the test supplies, so no run reads the clock.
const TAKEN: &str = "2026-01-02T03:04:05Z";

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn fixture(relative: &str) -> String {
    std::fs::read_to_string(fixture_root().join(relative)).expect("fixture is readable")
}

/// Take a snapshot of one source text, as the root sheet of its own file.
fn snapshot(source: &str) -> Snapshot {
    let doc = Doc::parse(source).expect("fixture parses");
    let schematic = Schematic::read(&doc).expect("fixture reads as a schematic");
    let path = SheetPath::root(schematic.uuid.as_ref().expect("the fixture has a uuid"));
    Snapshot::take("base", TAKEN, &path, &doc, &schematic).expect("the snapshot is taken")
}

#[test]
fn snapshot_hashes_ignore_item_order() {
    let source = fixture("sch/item_zoo.kicad_sch");
    let doc = Doc::parse(&source).expect("fixture parses");
    let mut schematic = Schematic::read(&doc).expect("fixture reads as a schematic");
    let path = SheetPath::root(schematic.uuid.as_ref().expect("the fixture has a uuid"));

    let first = Snapshot::take("base", TAKEN, &path, &doc, &schematic).expect("snapshot");
    schematic.items.reverse();
    let reversed = Snapshot::take("base", TAKEN, &path, &doc, &schematic).expect("snapshot");
    schematic.items.rotate_left(3);
    let rotated = Snapshot::take("base", TAKEN, &path, &doc, &schematic).expect("snapshot");

    assert_eq!(first.render(), reversed.render());
    assert_eq!(first.render(), rotated.render());
    assert!(
        first.objects.len() > 10,
        "the fixture holds more than ten objects"
    );
}

#[test]
fn moving_a_symbol_changes_where_it_is_and_not_what_it_is() {
    let base = fixture("sch/item_zoo.kicad_sch");
    let moved = base.replace(
        "(lib_id \"Test:R\")\n\t\t(at 50.8 50.8 0)",
        "(lib_id \"Test:R\")\n\t\t(at 63.5 50.8 0)",
    );
    assert_ne!(base, moved, "the move edits the fixture text");

    let before = snapshot(&base);
    let after = snapshot(&moved);
    let before_symbol = before
        .object(SYMBOL)
        .expect("the symbol is in the snapshot");
    let after_symbol = after.object(SYMBOL).expect("the symbol is in the snapshot");

    assert_ne!(before_symbol.geometry, after_symbol.geometry);
    assert_eq!(before_symbol.data, after_symbol.data);
}

#[test]
fn editing_a_field_value_changes_what_it_is_and_not_where_it_is() {
    let base = fixture("sch/item_zoo.kicad_sch");
    let edited = base.replace("(property \"Value\" \"10k\"", "(property \"Value\" \"22k\"");
    assert_ne!(base, edited, "the edit changes the fixture text");

    let before = snapshot(&base);
    let after = snapshot(&edited);
    let key = format!("{SYMBOL}.Value");
    let before_field = before.object(&key).expect("the field is in the snapshot");
    let after_field = after.object(&key).expect("the field is in the snapshot");

    assert_eq!(before_field.geometry, after_field.geometry);
    assert_ne!(before_field.data, after_field.data);

    let before_symbol = before
        .object(SYMBOL)
        .expect("the symbol is in the snapshot");
    let after_symbol = after.object(SYMBOL).expect("the symbol is in the snapshot");
    assert_eq!(before_symbol.geometry, after_symbol.geometry);
    assert_eq!(
        before_symbol.data, after_symbol.data,
        "a field carries its own hashes, so its owner is untouched"
    );
}

#[test]
fn a_field_keeps_its_offset_when_its_symbol_moves() {
    let base = fixture("sch/item_zoo.kicad_sch");
    // KiCad rewrites the absolute position of every field when a symbol moves.
    // The hash covers the offset from the symbol, so the fields report nothing.
    let moved = base
        .replace(
            "(lib_id \"Test:R\")\n\t\t(at 50.8 50.8 0)",
            "(lib_id \"Test:R\")\n\t\t(at 50.8 63.5 0)",
        )
        .replace("(at 52.832 50.8 90)", "(at 52.832 63.5 90)")
        .replace("(at 50.8 50.8 90)", "(at 50.8 63.5 90)")
        .replace("(at 50.8 50.8 0)", "(at 50.8 63.5 0)");

    let before = snapshot(&base);
    let after = snapshot(&moved);
    for name in [
        "Reference",
        "Value",
        "Footprint",
        "Datasheet",
        "Description",
    ] {
        let key = format!("{SYMBOL}.{name}");
        let before_field = before.object(&key).expect("the field is in the snapshot");
        let after_field = after.object(&key).expect("the field is in the snapshot");
        assert_eq!(
            before_field.geometry, after_field.geometry,
            "{name} moved with its symbol, so its own position did not change"
        );
    }
}

#[test]
fn a_snapshot_file_is_the_same_bytes_on_every_run() {
    let source = fixture("sch/item_zoo.kicad_sch");
    assert_eq!(snapshot(&source).render(), snapshot(&source).render());

    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join("snapshot_writes");
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("the test directory is made");

    let first = snapshot(&source).write_in(&directory).expect("writes");
    let first_bytes = std::fs::read(&first).expect("the file reads back");
    let second = snapshot(&source).write_in(&directory).expect("writes");
    let second_bytes = std::fs::read(&second).expect("the file reads back");

    assert_eq!(first, second);
    assert!(first.ends_with("snapshots/base.snap"), "{first:?}");
    assert_eq!(first_bytes, second_bytes);
}

#[test]
fn a_snapshot_survives_a_write_and_a_read() {
    let taken = snapshot(&fixture("sch/item_zoo.kicad_sch"));
    let read = Snapshot::parse(&taken.render()).expect("the file parses");

    assert_eq!(read.name, taken.name);
    assert_eq!(read.sheet_path, taken.sheet_path);
    assert_eq!(read.taken, taken.taken);
    assert_eq!(read.tool, taken.tool);
    assert_eq!(read.objects.len(), taken.objects.len());
    for (read_object, taken_object) in read.objects.iter().zip(&taken.objects) {
        assert_eq!(read_object.key, taken_object.key);
        assert_eq!(read_object.kind, taken_object.kind);
        assert_eq!(read_object.geometry, taken_object.geometry);
        assert_eq!(read_object.data, taken_object.data);
    }
    assert_eq!(read.render(), taken.render());
}

#[test]
fn every_object_of_a_file_reaches_its_snapshot() {
    let source = fixture("sch/item_zoo.kicad_sch");
    let doc = Doc::parse(&source).expect("fixture parses");
    let schematic = Schematic::read(&doc).expect("fixture reads as a schematic");
    let path = SheetPath::root(schematic.uuid.as_ref().expect("the fixture has a uuid"));
    let taken = Snapshot::take("base", TAKEN, &path, &doc, &schematic).expect("snapshot");

    let owned: usize = schematic
        .items
        .iter()
        .map(|item| match item {
            kicli::model::Item::Symbol(symbol) => symbol.fields.len(),
            kicli::model::Item::Label(label) => label.fields.len(),
            kicli::model::Item::Sheet(sheet) => sheet.fields.len() + sheet.pins.len(),
            _ => 0,
        })
        .sum();
    assert_eq!(taken.objects.len(), schematic.items.len() + owned);
}

#[test]
fn a_snapshot_name_that_would_leave_the_cache_is_refused() {
    let source = fixture("sch/item_zoo.kicad_sch");
    let doc = Doc::parse(&source).expect("fixture parses");
    let schematic = Schematic::read(&doc).expect("fixture reads as a schematic");
    let path = SheetPath::root(schematic.uuid.as_ref().expect("the fixture has a uuid"));

    for name in ["../escape", "two words", "with/slash", ""] {
        assert!(
            Snapshot::take(name, TAKEN, &path, &doc, &schematic).is_err(),
            "{name:?} is not a snapshot name"
        );
    }
    assert!(Snapshot::take("base", "not a stamp", &path, &doc, &schematic).is_err());
}
