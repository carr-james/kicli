//! Every object of a schematic file arrives in the typed model.
//!
//! The point of the test is that nothing is dropped. A token the model does not
//! name arrives as an unnamed item carrying its head token, so a file kicli
//! cannot fully interpret still reports what it holds.

use kicli::model::{Item, LabelKind, LineKind, Schematic, SheetPath};
use kicli_sexpr::Doc;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn read(relative: &str) -> (Doc, Schematic) {
    let path = fixture_root().join(relative);
    let source = std::fs::read_to_string(&path).expect("fixture is readable");
    let doc = Doc::parse(&source).expect("fixture parses");
    let schematic = Schematic::read(&doc).expect("fixture reads as a schematic");
    (doc, schematic)
}

/// The lists a schematic carries as a header rather than as an object.
const HEADER_TOKENS: &[&str] = &[
    "version",
    "generator",
    "generator_version",
    "uuid",
    "paper",
    "title_block",
    "lib_symbols",
    "sheet_instances",
    "embedded_fonts",
];

#[test]
fn every_object_of_a_file_arrives_in_the_model() {
    for fixture in [
        "sch/item_zoo.kicad_sch",
        "sch/nets/nets.kicad_sch",
        "sch/nets/nets_channel.kicad_sch",
        "sch/multi_instance/multi_instance.kicad_sch",
        "geometry/orientations.kicad_sch",
        "geometry/asymmetric.kicad_sch",
        "text/calibration.kicad_sch",
    ] {
        let (doc, schematic) = read(fixture);
        let root = doc.root().expect("a file has a root list");
        let objects = doc
            .children(root)
            .iter()
            .filter(|&&child| {
                doc.head(child)
                    .is_some_and(|head| !HEADER_TOKENS.contains(&head))
            })
            .count();
        assert_eq!(
            objects,
            schematic.items.len(),
            "{fixture}: every object of the file is in the model"
        );
        for item in &schematic.items {
            assert!(
                doc.children(item.node()).len() > 1,
                "{fixture}: every item keeps the node it came from"
            );
        }
    }
}

#[test]
fn the_item_zoo_names_every_kind_it_holds() {
    let (_doc, schematic) = read("sch/item_zoo.kicad_sch");

    assert_eq!(schematic.symbols().count(), 1, "one placed symbol");
    assert_eq!(schematic.junctions().count(), 1, "one junction");
    assert_eq!(schematic.sheets().count(), 1, "one child sheet");
    assert_eq!(
        schematic
            .lines()
            .filter(|line| line.kind == LineKind::Wire)
            .count(),
        2,
        "two wire segments"
    );
    assert_eq!(
        schematic
            .lines()
            .filter(|line| line.kind == LineKind::Bus)
            .count(),
        1,
        "one bus segment"
    );

    let label_kinds: BTreeSet<&str> = schematic
        .labels()
        .map(|label| match label.kind {
            LabelKind::Local => "local",
            LabelKind::Global => "global",
            LabelKind::Hierarchical => "hierarchical",
            LabelKind::NetclassFlag => "netclass",
        })
        .collect();
    assert_eq!(
        label_kinds,
        ["global", "hierarchical", "local", "netclass"]
            .into_iter()
            .collect::<BTreeSet<_>>(),
        "all four label tokens are told apart"
    );

    // Graphics and images have no typed form yet. They must still arrive,
    // named, rather than vanish.
    let unnamed: BTreeSet<&str> = schematic
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Other { token, .. } => Some(token.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        unnamed,
        ["arc", "circle", "image", "polyline", "rectangle"]
            .into_iter()
            .collect::<BTreeSet<_>>(),
        "the objects with no typed form keep their head token"
    );
}

#[test]
fn a_symbol_reports_the_reference_of_the_sheet_path_it_is_asked_about() {
    let (_doc, schematic) = read("sch/nets/nets_channel.kicad_sch");
    let root = "13000001-0000-4000-8000-030000000000";
    let sheet_a = SheetPath(format!("/{root}/1300006c-0000-4000-8000-03a000000001"));
    let sheet_b = SheetPath(format!("/{root}/1300006d-0000-4000-8000-03b000000001"));

    let symbol = schematic
        .symbols()
        .find(|symbol| symbol.reference_on(&sheet_a).is_some_and(|r| r.0 == "R100"))
        .expect("the child sheet holds R100 on its first placement");
    assert_eq!(
        symbol.reference_on(&sheet_b).map(|r| r.0.as_str()),
        Some("R200"),
        "the same symbol is R200 on the other placement"
    );
    assert_eq!(
        symbol.field("Reference").map(|field| field.value.as_str()),
        Some("R100"),
        "the cached field holds one of the two, which is why it is not the answer"
    );
}

#[test]
fn power_symbols_are_told_apart_from_parts() {
    let (_doc, schematic) = read("sch/nets/nets.kicad_sch");
    let power: Vec<&str> = schematic
        .symbols()
        .filter(|symbol| symbol.is_power())
        .filter_map(|symbol| symbol.field("Value").map(|field| field.value.as_str()))
        .collect();
    assert_eq!(power, ["GND", "GND"], "both ground symbols carry their net");
    assert!(
        schematic.symbols().filter(|s| !s.is_power()).count() >= 17,
        "the parts are not counted as power symbols"
    );
}

#[test]
fn a_sheet_names_its_child_file_and_its_pins() {
    let (_doc, schematic) = read("sch/nets/nets.kicad_sch");
    let mut sheets: Vec<&kicli::model::SheetItem> = schematic.sheets().collect();
    sheets.sort_by_key(|sheet| sheet.name().unwrap_or_default().to_owned());
    assert_eq!(sheets.len(), 2, "the child sheet is placed twice");
    assert_eq!(sheets[0].name(), Some("channel_a"));
    assert_eq!(sheets[1].name(), Some("channel_b"));
    for sheet in sheets {
        assert_eq!(sheet.file(), Some("nets_channel.kicad_sch"));
        assert_eq!(sheet.pins.len(), 1, "one sheet pin");
        assert_eq!(sheet.pins[0].name, "IN");
        assert_eq!(
            sheet.pins[0].direction, "input",
            "a sheet pin writes its direction as a bare token, not a shape list"
        );
    }
}

#[test]
fn hidden_fields_are_recognised_in_both_file_layouts() {
    // The `hide` token sits inside `effects` before stamp 20251028 and beside
    // `show_name` after it. Reading one place only makes an older file's hidden
    // fields look visible.
    let (_doc, current) = read("sch/nets/nets.kicad_sch");
    assert!(
        current
            .symbols()
            .filter(|symbol| symbol.is_power())
            .any(|symbol| symbol.field("Reference").is_some_and(|field| field.hidden)),
        "a power symbol's reference is hidden in a current file"
    );

    // The same field, written both ways, read under both stamps.
    let older = symbol_with_field(
        20_250_114,
        "(property \"Datasheet\" \"\"\n\t\t\t(at 0 0 0)\n\t\t\t(effects\n\t\t\t\t(font\n\t\t\t\t\t(size 1.27 1.27)\n\t\t\t\t)\n\t\t\t\t(hide yes)\n\t\t\t)\n\t\t)",
    );
    assert!(
        older.symbols().next().expect("one symbol").fields[0].hidden,
        "a v9 file hides a field from inside effects"
    );

    let newer = symbol_with_field(
        20_260_306,
        "(property \"Datasheet\" \"\"\n\t\t\t(at 0 0 0)\n\t\t\t(hide yes)\n\t\t\t(show_name no)\n\t\t\t(do_not_autoplace no)\n\t\t)",
    );
    assert!(
        newer.symbols().next().expect("one symbol").fields[0].hidden,
        "a v10 file hides a field beside show_name"
    );
}

/// A one-symbol schematic at a given stamp, carrying one field as written.
fn symbol_with_field(stamp: u32, property: &str) -> Schematic {
    let source = format!(
        "(kicad_sch\n\t(version {stamp})\n\t(symbol\n\t\t(lib_id \"Test:R\")\n\t\t(at 0 0 0)\n\t\t(uuid \"1234\")\n\t\t{property}\n\t)\n)\n"
    );
    let doc = Doc::parse(&source).expect("parses");
    Schematic::read(&doc).expect("reads")
}
