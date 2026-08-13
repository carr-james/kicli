//! A placement resolves to the definition KiCad would draw.

use kicli::model::{Schematic, definition_of, read_library};
use kicli_sexpr::Doc;
use std::path::Path;

fn read(source: &str) -> (Schematic, Vec<kicli::model::LibrarySymbol>) {
    let doc = Doc::parse(source).expect("parses");
    let schematic = Schematic::read(&doc).expect("reads");
    let library = read_library(&doc, &schematic.library_symbols, schematic.version);
    (schematic, library)
}

/// A two-unit part: unit 0 holds the shared graphics and one shared pin, and
/// units 1 and 2 hold one pin each.
const MULTI_UNIT: &str = r#"(kicad_sch
	(version 20260306)
	(lib_symbols
		(symbol "Test:DUAL"
			(symbol "DUAL_0_1"
				(rectangle
					(start -5.08 5.08)
					(end 5.08 -5.08)
				)
				(pin power_in line
					(at 0 7.62 270)
					(length 2.54)
					(name "VCC")
					(number "8")
				)
			)
			(symbol "DUAL_1_1"
				(pin input line
					(at -7.62 2.54 0)
					(length 2.54)
					(name "A")
					(number "1")
				)
			)
			(symbol "DUAL_2_1"
				(pin input line
					(at -7.62 -2.54 0)
					(length 2.54)
					(name "B")
					(number "2")
				)
			)
		)
	)
	(symbol
		(lib_id "Test:DUAL")
		(at 50.8 50.8 0)
		(unit 2)
		(body_style 1)
		(uuid "aaaa")
		(property "Reference" "U1"
			(at 50.8 50.8 0)
		)
	)
)
"#;

#[test]
fn unit_and_body_style_select_pins() {
    let (schematic, library) = read(MULTI_UNIT);
    let symbol = schematic.symbols().next().expect("one placement");
    let definition = definition_of(&library, symbol).expect("it resolves");

    let mut numbers: Vec<&str> = definition
        .pins_for(symbol.unit, symbol.body_style)
        .map(|pin| pin.number.as_str())
        .collect();
    numbers.sort_unstable();
    assert_eq!(
        numbers,
        ["2", "8"],
        "unit two draws its own pin and the pin common to every unit, not unit one's"
    );

    let mut unit_one: Vec<&str> = definition
        .pins_for(1, 1)
        .map(|pin| pin.number.as_str())
        .collect();
    unit_one.sort_unstable();
    assert_eq!(
        unit_one,
        ["1", "8"],
        "unit one draws its own and the common one"
    );
}

#[test]
fn library_coordinates_are_negated_exactly_once() {
    let (_schematic, library) = read(MULTI_UNIT);
    let definition = &library[0];
    let common = definition
        .pins_for(1, 1)
        .find(|pin| pin.number == "8")
        .expect("the shared pin");
    // The file says (at 0 7.62), Y-up. In schematic sense that is -76200.
    assert_eq!(common.at.y.0, -76_200, "read once, negated once");

    let unit_one = definition
        .pins_for(1, 1)
        .find(|pin| pin.number == "1")
        .expect("pin one");
    assert_eq!(unit_one.at.x.0, -76_200);
    assert_eq!(unit_one.at.y.0, -25_400);
}

#[test]
fn a_lib_name_redirect_wins_over_the_lib_id() {
    // A symbol edited in place gets a uniquified cache entry, and `lib_name`
    // points at it. Keying by `lib_id` alone finds the wrong definition, or
    // none at all.
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sch/lib_name_redirect.kicad_sch");
    let source = std::fs::read_to_string(path).expect("fixture is readable");
    let (schematic, library) = read(&source);

    let symbol = schematic.symbols().next().expect("one placement");
    assert!(
        symbol.lib_name.is_some(),
        "the fixture exists to carry a redirect"
    );
    let definition = definition_of(&library, symbol).expect("it resolves");
    assert_eq!(
        Some(definition.name.as_str()),
        symbol.lib_name.as_deref(),
        "the definition is the one lib_name names"
    );
    assert_ne!(
        definition.name, symbol.lib_id.0,
        "which is not the one lib_id names"
    );
}

#[test]
fn a_power_symbol_definition_says_so() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sch/nets/nets.kicad_sch");
    let source = std::fs::read_to_string(path).expect("fixture is readable");
    let (_schematic, library) = read(&source);
    let ground = library
        .iter()
        .find(|definition| definition.name == "Test:GND")
        .expect("the ground symbol is embedded");
    assert!(
        ground.is_power,
        "a power symbol is marked in its definition"
    );
    assert!(
        ground
            .pins_for(1, 1)
            .all(|pin| pin.electrical == "power_in"),
        "its pin is a power input, which is what makes the net name carry"
    );
}
