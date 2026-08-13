//! kicli puts pins where KiCad puts them.
//!
//! The fixtures place a resistor and an asymmetric four-pin part at all eight
//! orientations. KiCad's own rule check reported every pin's position, and
//! those 48 rows are committed beside the fixtures. This is the gate: integer
//! comparison, no tolerance.

use kicli::geometry::{Point, resolve_pins};
use kicli::model::{Schematic, definition_of, read_library};
use kicli_sexpr::Doc;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/geometry")
        .join(name)
}

/// The committed table: pin uuid to position, plus the refdes and pin number.
fn expected(name: &str) -> BTreeMap<String, (String, String, Point)> {
    let text = std::fs::read_to_string(fixture(name)).expect("table is readable");
    text.lines()
        .filter(|line| !line.starts_with('#'))
        .map(|line| {
            let fields: Vec<&str> = line.split_whitespace().collect();
            let position = Point::new(
                fields[4].parse().expect("x is an integer"),
                fields[5].parse().expect("y is an integer"),
            );
            (
                fields[6].to_owned(),
                (fields[0].to_owned(), fields[3].to_owned(), position),
            )
        })
        .collect()
}

/// Resolve every pin of a fixture, keyed by the pin instance uuid.
fn resolved(name: &str) -> BTreeMap<String, (String, String, Point)> {
    let source = std::fs::read_to_string(fixture(name)).expect("fixture is readable");
    let doc = Doc::parse(&source).expect("fixture parses");
    let schematic = Schematic::read(&doc).expect("fixture reads");
    let library = read_library(&doc, &schematic.library_symbols, schematic.version);

    let mut found = BTreeMap::new();
    for symbol in schematic.symbols() {
        let definition = definition_of(&library, symbol).expect("the placement resolves");
        let reference = symbol
            .field("Reference")
            .map(|field| field.value.clone())
            .unwrap_or_default();
        for pin in resolve_pins(symbol, definition) {
            let uuid = pin.uuid.expect("every placed pin carries a uuid").0;
            found.insert(uuid, (reference.clone(), pin.number, pin.position));
        }
    }
    found
}

#[test]
fn pin_positions_match_kicad() {
    let mut checked = 0;
    for (fixture, table) in [
        ("orientations.kicad_sch", "orientations.expected"),
        ("asymmetric.kicad_sch", "asymmetric.expected"),
    ] {
        let want = expected(table);
        let got = resolved(fixture);
        assert_eq!(
            want.keys().collect::<Vec<_>>(),
            got.keys().collect::<Vec<_>>(),
            "{fixture}: the same pins are resolved as KiCad reported"
        );
        for (uuid, (reference, number, position)) in &want {
            assert_eq!(
                &got[uuid],
                &(reference.clone(), number.clone(), *position),
                "{fixture}: {reference} pin {number} is where KiCad says it is"
            );
        }
        checked += want.len();
    }
    assert_eq!(checked, 48, "sixteen resistor pins and thirty-two others");
}

#[test]
fn every_resolved_pin_is_on_the_grid() {
    // Off-grid pins are a blocking finding later. The fixtures are drawn on
    // grid, so anything off it here is an arithmetic error rather than a
    // drawing one.
    for fixture in ["orientations.kicad_sch", "asymmetric.kicad_sch"] {
        for (uuid, (reference, number, position)) in resolved(fixture) {
            assert!(
                position.is_on_grid(),
                "{fixture}: {reference} pin {number} ({uuid}) is off grid at {position}"
            );
        }
    }
}

#[test]
fn an_asymmetric_part_tells_the_mirrors_apart() {
    // The whole point of the second fixture: with the mirror composed in the
    // wrong order, mirror X and mirror Y swap at 90 degrees. On a symmetric
    // part that is invisible. Here the two placements must differ.
    let got = resolved("asymmetric.kicad_sch");
    let positions = |reference: &str| {
        let mut found: Vec<Point> = got
            .values()
            .filter(|(candidate, _, _)| candidate == reference)
            .map(|(_, _, position)| *position)
            .collect();
        found.sort();
        found
    };
    // A7 is 90 degrees with mirror X, A8 is 90 degrees with mirror Y.
    assert_ne!(
        positions("A7"),
        positions("A8"),
        "the two mirrors of a rotated asymmetric part are different placements"
    );
}
