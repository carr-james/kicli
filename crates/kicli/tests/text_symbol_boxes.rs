//! A symbol's body box holds its drawing, and its full box holds its text.
//!
//! The orientation fixture places the same resistor eight ways, so the body box
//! must keep its size and swap its axes at a quarter turn. Every field of every
//! placement is visible, so hiding them must leave the body box alone and must
//! take exactly their own boxes off the full one.

use std::path::{Path, PathBuf};

use kicli::geometry::text::{TextStyle, text_box};
use kicli::geometry::{Rect, symbol_boxes};
use kicli::model::items::Field;
use kicli::model::{Schematic, definition_of, read_library};
use kicli_sexpr::Doc;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/geometry")
        .join(name)
}

#[test]
fn symbol_body_box_excludes_field_text() {
    let source = std::fs::read_to_string(fixture("orientations.kicad_sch"))
        .expect("the fixture is readable");
    let doc = Doc::parse(&source).expect("the fixture parses");
    let schematic = Schematic::read(&doc).expect("the fixture reads");
    let library = read_library(&doc, &schematic.library_symbols, schematic.version);
    let box_of = |field: &Field| {
        let style = TextStyle::read(&doc, field.node);
        text_box(&field.value, field.at, field.angle, &style).axis_aligned()
    };

    let mut sizes = Vec::new();
    let mut grew = 0;
    for symbol in schematic.symbols() {
        let definition = definition_of(&library, symbol).expect("the placement resolves");
        let boxes = symbol_boxes(&doc, symbol, definition);
        for field in &symbol.fields {
            assert!(!field.hidden, "the fixture shows every field");
        }

        // The full box holds the body, whatever the text does.
        assert_eq!(boxes.full, boxes.full.union(boxes.body));
        assert!(!boxes.approximate, "the fixture uses the stroke font");

        // The body is the drawing alone: hiding every field leaves it exactly
        // as it was, and the full box loses the text.
        let mut bare = symbol.clone();
        for field in &mut bare.fields {
            field.hidden = true;
        }
        let hidden = symbol_boxes(&doc, &bare, definition);
        assert_eq!(hidden.body, boxes.body);
        assert_eq!(
            boxes.full,
            symbol
                .fields
                .iter()
                .fold(hidden.full, |so_far, field| so_far.union(box_of(field))),
            "the full box is not the body and the visible fields"
        );

        // Showing one field again adds that field's own box and nothing else.
        let mut shown = bare.clone();
        let reference = shown
            .fields
            .iter_mut()
            .find(|field| field.name == "Reference")
            .expect("the placement has a reference field");
        reference.hidden = false;
        let reference_box = box_of(reference);
        let regrown = symbol_boxes(&doc, &shown, definition);
        assert_eq!(regrown.full, hidden.full.union(reference_box));
        if regrown.full != hidden.full {
            grew += 1;
        }

        sizes.push((symbol.angle.0, symbol.mirror, boxes.body));
    }

    assert_eq!(sizes.len(), 8, "the fixture places the symbol eight ways");
    assert!(
        grew > 0,
        "no reference text reaches outside its symbol, so this fixture proves nothing"
    );

    // The body box is the same size at every orientation, with its axes swapped
    // at a quarter turn.
    let first = sizes
        .iter()
        .find(|(angle, ..)| *angle == 0)
        .map(|(.., rect)| *rect)
        .expect("one placement is upright");
    let upright = |rect: Rect| (rect.width(), rect.height());
    let turned = |rect: Rect| (rect.height(), rect.width());
    for (angle, mirror, body) in &sizes {
        let expected = if angle % 180 == 0 {
            upright(first)
        } else {
            turned(first)
        };
        assert_eq!(
            upright(*body),
            expected,
            "the body box changed size at {angle} degrees mirror {mirror:?}"
        );
    }
    println!("symbol boxes: 8 placements, {grew} with a reference outside the body");
}
