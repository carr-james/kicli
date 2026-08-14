//! A measurement kicli cannot represent stops the load and says where.
//!
//! kicli's reader accepts millimetres with at most four decimals, which is what
//! KiCad writes. It once answered a value outside that with zero, because the
//! caller of a failed parse reached for a default. A zero coordinate is not a
//! refusal: it moves the item to the origin, joins it to whatever else landed
//! there, and reports a confident net list about a drawing nobody drew. It cost
//! this project two false findings before it was caught.
//!
//! So the value is refused, the file does not load, and the message carries the
//! text as the file spells it and where to find it.

use kicli::model::{Hierarchy, Schematic};
use kicli_sexpr::Doc;
use std::path::{Path, PathBuf};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sch")
}

#[test]
fn a_coordinate_kicli_cannot_read_stops_the_load() {
    let path = fixtures().join("unreadable_coordinate.kicad_sch");
    let Err(error) = Hierarchy::load(&path) else {
        panic!("the file must not load");
    };
    let message = error.to_string();

    // The offending text, exactly as the file spells it.
    assert!(
        message.contains("76.19999999999999"),
        "the message must name the value: {message}"
    );
    // And where to find it. The fixture puts it in the wire on line 10.
    let offset = std::fs::read_to_string(&path)
        .expect("the fixture is readable")
        .find("76.19999999999999")
        .expect("the fixture carries the value");
    assert!(
        message.contains(&offset.to_string()),
        "the message must locate the value at byte {offset}: {message}"
    );
}

#[test]
fn the_same_drawing_loads_once_the_value_is_one_kicad_writes() {
    // The control. Only the number changes, so the refusal above is about the
    // number and not about anything else in the drawing.
    let text = std::fs::read_to_string(fixtures().join("unreadable_coordinate.kicad_sch"))
        .expect("the fixture is readable")
        .replace("76.19999999999999", "76.2");
    let doc = Doc::parse(&text).expect("the text parses");
    let schematic = Schematic::read(&doc).expect("the rounded drawing reads");
    assert_eq!(schematic.items.len(), 1, "the wire survives");
}

#[test]
fn a_whole_number_is_not_a_measurement() {
    // The version stamp is far larger than any coordinate kicli can hold. It is
    // not a millimetre value, so it is not checked as one, and a file carrying
    // it reads.
    let doc = Doc::parse("(kicad_sch (version 20260306) (uuid \"u\"))").expect("parses");
    assert!(doc.check_measurements(&[]).is_ok());
    assert!(Schematic::read(&doc).is_ok());
}

#[test]
fn text_that_looks_like_a_number_is_still_text() {
    // A property value is quoted, so it is never read as a measurement however
    // many decimals somebody typed into it.
    let doc = Doc::parse(
        "(kicad_sch (version 20260306) (uuid \"u\")\n\
         (property \"Value\" \"1.2345678\"))",
    )
    .expect("parses");
    assert!(doc.check_measurements(&[]).is_ok());
    assert!(Schematic::read(&doc).is_ok());
}
