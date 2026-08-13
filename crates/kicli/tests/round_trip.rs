//! The round-trip properties, over this crate's schematic-level fixtures.
//!
//! `kicli-sexpr` owns the properties and its own fixtures. It must not reach
//! into this crate's directory, so the same properties run here, against this
//! crate's fixtures, through the public API.

use kicli_sexpr::{Doc, FormatMode, flatten, prettify};
use std::path::{Path, PathBuf};

struct Fixture {
    path: PathBuf,
    name: String,
    mode: FormatMode,
    canonical: bool,
}

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn fixtures() -> Vec<Fixture> {
    let root = fixture_root();
    let manifest = std::fs::read_to_string(root.join("MANIFEST")).expect("manifest is readable");
    manifest
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| {
            let fields: Vec<&str> = line.split_whitespace().collect();
            let mode = match fields[2] {
                "normal" => FormatMode::Normal,
                "compact" => FormatMode::CompactTextProperties,
                "library-table" => FormatMode::LibraryTable,
                // The project file is JSON, not an s-expression, and an
                // oracle is KiCad's answer about a fixture rather than a
                // schematic of its own.
                "json" | "oracle" => return None,
                other => panic!("unknown mode {other}"),
            };
            Some(Fixture {
                path: root.join(fields[0]),
                name: fields[0].to_owned(),
                mode,
                canonical: fields[3] == "yes",
            })
        })
        .collect()
}

fn read(fixture: &Fixture) -> String {
    std::fs::read_to_string(&fixture.path).expect("fixture is readable")
}

#[test]
fn emit_reproduces_input_bytes() {
    let canonical: Vec<Fixture> = fixtures().into_iter().filter(|f| f.canonical).collect();
    assert!(!canonical.is_empty(), "there are canonical fixtures");

    for fixture in &canonical {
        let source = read(fixture);
        let doc = Doc::parse(&source).expect("fixture parses");
        assert!(
            doc.is_canonical(),
            "{} is detected as canonical",
            fixture.name
        );
        assert_eq!(
            doc.emit(),
            source,
            "{} round-trips byte for byte",
            fixture.name
        );
    }
}

#[test]
fn reparse_preserves_tree() {
    for fixture in fixtures() {
        let source = read(&fixture);
        let first = Doc::parse(&source).expect("fixture parses");
        let second = Doc::parse(&first.emit()).expect("output parses");
        assert!(
            first.structurally_eq(&second),
            "{} keeps its tokens and shape",
            fixture.name
        );
    }
}

#[test]
fn prettify_reproduces_kicad_layout() {
    for fixture in fixtures().iter().filter(|f| f.canonical) {
        let source = read(fixture);
        assert_eq!(
            prettify(&flatten(&source), fixture.mode),
            source,
            "{} is reproduced from its tokens alone",
            fixture.name
        );
    }
}

/// A symbol placed on a sheet used twice carries two references, and the truth
/// is in the instance list rather than the cached property.
#[test]
fn a_twice_placed_sheet_keeps_both_references() {
    let path = fixture_root().join("sch/multi_instance/channel.kicad_sch");
    let source = std::fs::read_to_string(&path).expect("fixture is readable");
    let doc = Doc::parse(&source).expect("parses");

    let references: Vec<String> = doc
        .node_ids()
        .filter(|&id| doc.head_is(id, "reference"))
        .filter_map(|id| doc.children(id).get(1).copied())
        .filter_map(|atom| doc.atom_text(atom).map(str::to_owned))
        .collect();

    assert_eq!(references, ["\"R201\"", "\"R301\""]);

    let cached: Vec<String> = doc
        .node_ids()
        .filter(|&id| doc.head_is(id, "property"))
        .filter(|&id| doc.atom_text(doc.children(id)[1]) == Some("\"Reference\""))
        .filter_map(|id| doc.atom_text(doc.children(id)[2]).map(str::to_owned))
        .collect();

    // Two Reference properties: the library symbol's default, and the placed
    // symbol's cache. Neither holds R301, which is exactly why the instance
    // list is the truth and the property is not.
    assert_eq!(cached, ["\"R\"", "\"R201\""]);
    assert!(
        !cached.iter().any(|value| value == "\"R301\""),
        "the second reference exists only in the instance list"
    );
}
