//! How much room a placed symbol takes on the page.
//!
//! A symbol has two boxes, because two kinds of rule want different answers.
//! The **body box** holds the graphics and the pins and no text: it answers
//! "do these two symbols overlap?". The **full box** adds every piece of text
//! the symbol draws, which is the visible fields and the pin names and numbers:
//! it answers "does this text collide with something?".
//!
//! The body is composed in library space, transformed by the placement's
//! orientation and moved to the placement's position. Only the two corners are
//! transformed, which is correct for the eight orientations a symbol can take
//! and for nothing else. Field boxes are already absolute, so they are merged
//! afterwards.
//!
//! Ported from `SCH_SYMBOL::doGetBoundingBox` (`eeschema/sch_symbol.cpp:2619-
//! 2643`), `LIB_SYMBOL::GetBodyBoundingBox` (`eeschema/lib_symbol.cpp:1423-
//! 1464`), `EDA_SHAPE::getBoundingBox` (`common/eda_shape.cpp:1344-1396`) and
//! `PIN_LAYOUT_CACHE` (`eeschema/pin_layout_cache.cpp:312-593`) at tag 10.0.5.
//! KiCad is GPL-3.0-or-later, as is kicli.
//!
//! Three parts of KiCad's own pin box are left out, and all three only ever
//! make a box bigger: the half pen width every shape is inflated by, the
//! decoration of an inverted or clocked pin, and the marker KiCad draws on a
//! pin that is not connected. A box here is therefore a lower bound on KiCad's,
//! never an upper one.

use kicli_sexpr::{Doc, NodeId};

use crate::geometry::font::{DEFAULT_PEN_WIDTH, string_extents};
use crate::geometry::text::{TextStyle, text_box};
use crate::geometry::{Angle, Iu, Point, Rect, Size, Transform};
use crate::model::items::Symbol;
use crate::model::library::{LibraryPin, LibrarySymbol, Shape};

/// How far pin text sits from the pin it belongs to.
///
/// `PIN_LAYOUT_CACHE::getPinTextOffset` (`eeschema/pin_layout_cache.cpp:512-
/// 517`) is 24 mil times the schematic's text offset ratio, which defaults to
/// 0.15 (`eeschema/default_values.h:72`). That is 4 mil.
const PIN_TEXT_OFFSET: Iu = Iu(1_016);

/// The boxes of one placed symbol, in schematic coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SymbolBoxes {
    /// The graphics and the pins, without any text.
    pub body: Rect,
    /// The body, the visible fields and the pin text.
    pub full: Rect,
    /// Is any part of the full box a guess?
    ///
    /// It is a guess when a field or a pin names an outline font, whose widths
    /// live in a font file kicli does not read.
    pub approximate: bool,
}

/// The boxes of a placed symbol.
///
/// The document is needed because a field's own text effects live in the tree,
/// beside the value the item model carries.
#[must_use]
pub fn symbol_boxes(doc: &Doc, symbol: &Symbol, definition: &LibrarySymbol) -> SymbolBoxes {
    let transform = Transform::from_file(symbol.angle, symbol.mirror);
    let place = |rect: Rect| rect.transformed(transform).offset(symbol.at);

    let mut body: Option<Rect> = None;
    let mut drawn: Option<Rect> = None;
    let mut approximate = false;

    for unit in definition.units_for(symbol.unit, symbol.body_style) {
        for shape in &unit.shapes {
            body = Some(merge(body, shape_box(shape)));
        }
        for pin in unit.pins.iter().filter(|pin| !pin.hidden) {
            body = Some(merge(body, pin_segment_box(pin)));
            for (style, box_in_pin) in pin_text_boxes(doc, definition, pin) {
                approximate |= !style.is_stroke();
                drawn = Some(merge(drawn, box_in_pin));
            }
        }
    }

    // A symbol with no graphics and no visible pin still has a position.
    let body = place(body.unwrap_or_else(|| Rect::around(Point::default())));
    let mut full = match drawn {
        Some(text) => body.union(place(text)),
        None => body,
    };

    for field in symbol.fields.iter().filter(|field| !field.hidden) {
        let style = TextStyle::read(doc, field.node);
        approximate |= !style.is_stroke();
        full = full.union(text_box(&field.value, field.at, field.angle, &style).axis_aligned());
    }

    SymbolBoxes {
        body,
        full,
        approximate,
    }
}

/// Merge a box into one that may not exist yet.
fn merge(box_so_far: Option<Rect>, next: Rect) -> Rect {
    match box_so_far {
        Some(rect) => rect.union(next),
        None => next,
    }
}

/// The box of one drawn shape, in library space.
///
/// A bezier is measured by its control points, which is never smaller than the
/// curve. KiCad measures the flattened curve instead
/// (`common/eda_shape.cpp:1379-1385`).
fn shape_box(shape: &Shape) -> Rect {
    match shape {
        Shape::Rectangle { start, end } => Rect::new(*start, *end),
        Shape::Polyline { points } => points
            .iter()
            .skip(1)
            .fold(Rect::around(points[0]), |box_so_far, &point| {
                box_so_far.union(Rect::around(point))
            }),
        Shape::Circle { center, radius } => Rect::around(*center).inflate(*radius),
        Shape::Arc { start, mid, end } => arc_box(*start, *mid, *end),
    }
}

/// The box of an arc, in library space.
///
/// The ends and every quarter turn the arc crosses enclose it, which is how
/// KiCad measures one (`common/eda_shape.cpp:1897-1945`). The middle point is
/// merged as well: it is on the arc, so it can only make the answer right.
///
/// The centre of the arc is where the three points stop being integers. KiCad
/// computes it in floating point too, so the two agree.
fn arc_box(start: Point, mid: Point, end: Point) -> Rect {
    let ends = Rect::new(start, end).union(Rect::around(mid));
    let Some((centre, radius)) = arc_centre(start, mid, end) else {
        return ends;
    };

    let angle_of = |point: Point| {
        let (dx, dy) = (
            f64::from(point.x.0 - centre.x.0),
            f64::from(point.y.0 - centre.y.0),
        );
        dy.atan2(dx).to_degrees().rem_euclid(360.0)
    };
    // The sweep runs from the first angle to the second, the way the angle
    // grows. The middle point says which of the two ends it starts at, which is
    // the winding KiCad settles the same way
    // (`common/eda_shape.cpp:1194-1206`).
    let swept = |first: f64, second: f64, angle: f64| {
        if second > first {
            angle > first && angle < second
        } else {
            angle > first || angle < second
        }
    };
    let (mut first, mut second) = (angle_of(start), angle_of(end));
    if !swept(first, second, angle_of(mid)) {
        std::mem::swap(&mut first, &mut second);
    }

    let mut rect = ends;
    for (quarter, extreme) in [
        (0.0, Point::new(centre.x.0 + radius.0, centre.y.0)),
        (90.0, Point::new(centre.x.0, centre.y.0 + radius.0)),
        (180.0, Point::new(centre.x.0 - radius.0, centre.y.0)),
        (270.0, Point::new(centre.x.0, centre.y.0 - radius.0)),
    ] {
        if swept(first, second, quarter) {
            rect = rect.union(Rect::around(extreme));
        }
    }
    rect
}

/// The centre and radius of the circle through three points.
///
/// Returns nothing when the three points are in a line, which draws no arc.
fn arc_centre(start: Point, mid: Point, end: Point) -> Option<(Point, Iu)> {
    let (ax, ay) = (f64::from(start.x.0), f64::from(start.y.0));
    let (bx, by) = (f64::from(mid.x.0), f64::from(mid.y.0));
    let (cx, cy) = (f64::from(end.x.0), f64::from(end.y.0));

    let determinant = 2.0 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by));
    if determinant.abs() < f64::EPSILON {
        return None;
    }
    let (aa, bb, cc) = (ax * ax + ay * ay, bx * bx + by * by, cx * cx + cy * cy);
    let x = (aa * (by - cy) + bb * (cy - ay) + cc * (ay - by)) / determinant;
    let y = (aa * (cx - bx) + bb * (ax - cx) + cc * (bx - ax)) / determinant;
    let radius = ((ax - x).powi(2) + (ay - y).powi(2)).sqrt();

    // A library coordinate is a few hundred thousand internal units, so the
    // narrowing is safe. The rounding matches KiCad's own.
    #[allow(clippy::cast_possible_truncation)]
    Some((
        Point::new(x.round() as i32, y.round() as i32),
        Iu(radius.round() as i32),
    ))
}

/// The box of a pin's drawn segment, in library space.
fn pin_segment_box(pin: &LibraryPin) -> Rect {
    let (dx, dy) = pin_direction(pin.angle);
    Rect::new(
        pin.at,
        Point::new(
            pin.at.x.0 + dx * pin.length.0,
            pin.at.y.0 + dy * pin.length.0,
        ),
    )
}

/// Which way a pin's body runs from its connection point, in library space.
///
/// The library reader has already flipped Y, so a pin the file calls 90 degrees
/// runs towards smaller Y. KiCad names the same four cases `PIN_RIGHT`,
/// `PIN_UP`, `PIN_LEFT` and `PIN_DOWN`
/// (`eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1670-1676`).
fn pin_direction(angle: Angle) -> (i32, i32) {
    match angle.0.rem_euclid(360) {
        90 => (0, -1),
        180 => (-1, 0),
        270 => (0, 1),
        _ => (1, 0),
    }
}

/// The boxes of the name and the number a pin draws, in library space.
///
/// The layout is worked out with the pin running to the right and is then
/// turned to the pin's own direction, exactly as KiCad does it
/// (`eeschema/pin_layout_cache.cpp:312-352, 520-593`).
fn pin_text_boxes(
    doc: &Doc,
    definition: &LibrarySymbol,
    pin: &LibraryPin,
) -> Vec<(TextStyle, Rect)> {
    let mut boxes = Vec::new();
    let offset = definition.pin_name_offset;
    let name_shown = !definition.pin_names_hidden && !pin.name.is_empty();
    let number_shown = !definition.pin_numbers_hidden && !pin.number.is_empty();

    if name_shown {
        let style = child_style(doc, pin.node, "name");
        let extents = string_extents(&pin.name, style.size, style.pen_width(DEFAULT_PEN_WIDTH));
        let box_in_pin = if offset.0 > 0 {
            // The name sits inside the body, past the end of the pin.
            centred(Point::new(pin.length.0, 0), extents)
                .offset(Point::new(extents.x.0 / 2 + offset.0, 0))
        } else {
            // The name sits over the pin.
            centred(Point::new(pin.length.0 / 2, 0), extents)
                .offset(Point::new(0, -extents.y.0 / 2 - PIN_TEXT_OFFSET.0))
        };
        boxes.push((style, turn_for_pin(box_in_pin, pin.angle)));
    }

    if number_shown {
        let style = child_style(doc, pin.node, "number");
        let extents = string_extents(&pin.number, style.size, style.pen_width(DEFAULT_PEN_WIDTH));
        // The number sits over the pin, and drops below it when the name is
        // over the pin as well.
        let below = offset.0 == 0 && name_shown;
        let step = extents.y.0 / 2 + PIN_TEXT_OFFSET.0;
        let box_in_pin = centred(Point::new(pin.length.0 / 2, 0), extents)
            .offset(Point::new(0, if below { step } else { -step }));
        boxes.push((style, turn_for_pin(box_in_pin, pin.angle)));
    }

    boxes
}

/// The style of a pin's `name` or `number` list.
fn child_style(doc: &Doc, pin: NodeId, head: &str) -> TextStyle {
    let child = doc
        .children(pin)
        .iter()
        .copied()
        .find(|&id| doc.head_is(id, head));
    match child {
        Some(node) => TextStyle::read(doc, node),
        None => TextStyle::default(),
    }
}

/// A box of this size, centred on a point.
fn centred(centre: Point, size: Size) -> Rect {
    let corner = Point::new(centre.x.0 - size.x.0 / 2, centre.y.0 - size.y.0 / 2);
    Rect::from_origin(corner, size)
}

/// Turn a box worked out for a pin running right so it fits the pin's own
/// direction.
///
/// A pin pointing left mirrors rather than turns, so its text stays the right
/// way up (`eeschema/pin_layout_cache.cpp:312-352`).
fn turn_for_pin(rect: Rect, angle: Angle) -> Rect {
    let (start, end) = (rect.start(), rect.end());
    match angle.0.rem_euclid(360) {
        // Up: a quarter turn.
        90 => Rect::new(
            Point::new(start.y.0, -start.x.0),
            Point::new(end.y.0, -end.x.0),
        ),
        // Left: mirrored across the pin, not turned.
        180 => Rect::new(
            Point::new(-start.x.0, start.y.0),
            Point::new(-end.x.0, end.y.0),
        ),
        // Down: a quarter turn the other way, then mirrored.
        270 => Rect::new(
            Point::new(start.y.0, start.x.0),
            Point::new(end.y.0, end.x.0),
        ),
        _ => rect,
    }
}

#[cfg(test)]
mod tests {
    use super::{arc_box, pin_direction, pin_text_boxes, shape_box};
    use crate::geometry::{Angle, Iu, Point, Rect};
    use crate::model::library::{LibraryPin, LibrarySymbol, Shape};
    use kicli_sexpr::Doc;

    #[test]
    fn a_shape_is_measured_by_its_extremes() {
        let rectangle = Shape::Rectangle {
            start: Point::new(1_000, 2_000),
            end: Point::new(-1_000, -2_000),
        };
        assert_eq!(
            shape_box(&rectangle),
            Rect::new(Point::new(-1_000, -2_000), Point::new(1_000, 2_000))
        );

        let circle = Shape::Circle {
            center: Point::new(0, 0),
            radius: Iu(500),
        };
        assert_eq!(
            shape_box(&circle),
            Rect::new(Point::new(-500, -500), Point::new(500, 500))
        );
    }

    #[test]
    fn an_arc_reaches_past_its_ends() {
        // A half circle from the right, through the top, to the left. In
        // schematic sense the top is the smaller Y.
        let boxed = arc_box(
            Point::new(1_000, 0),
            Point::new(0, -1_000),
            Point::new(-1_000, 0),
        );
        assert_eq!(
            boxed,
            Rect::new(Point::new(-1_000, -1_000), Point::new(1_000, 0))
        );

        // The same half circle the other way round bulges downwards instead.
        let other = arc_box(
            Point::new(1_000, 0),
            Point::new(0, 1_000),
            Point::new(-1_000, 0),
        );
        assert_eq!(
            other,
            Rect::new(Point::new(-1_000, 0), Point::new(1_000, 1_000))
        );

        // A quarter turn that crosses no extreme is measured by its ends.
        let quarter = arc_box(
            Point::new(1_000, 0),
            Point::new(707, -707),
            Point::new(0, -1_000),
        );
        assert_eq!(
            quarter,
            Rect::new(Point::new(0, -1_000), Point::new(1_000, 0))
        );

        // Three points in a line draw nothing, and still have a box.
        let flat = arc_box(Point::new(0, 0), Point::new(500, 0), Point::new(1_000, 0));
        assert_eq!(flat, Rect::new(Point::new(0, 0), Point::new(1_000, 0)));
    }

    /// A pin running right, with a name and a number of the default size.
    const PIN_SOURCE: &str = "((pin\n\t(name \"NAME\" (effects (font (size 1.27 1.27))))\n\t(number \"1\" (effects (font (size 1.27 1.27))))))";

    /// A symbol holding that one pin, with the given name settings.
    fn one_pin(
        offset: Iu,
        names_hidden: bool,
        numbers_hidden: bool,
    ) -> (Doc, LibrarySymbol, LibraryPin) {
        let doc = Doc::parse(PIN_SOURCE).expect("the fragment parses");
        let root = doc.root().expect("the fragment has a root");
        let node = doc.children(root)[0];
        let symbol = LibrarySymbol {
            name: "Test:P".to_owned(),
            is_power: false,
            pin_name_offset: offset,
            pin_numbers_hidden: numbers_hidden,
            pin_names_hidden: names_hidden,
            fields: Vec::new(),
            units: Vec::new(),
            node,
        };
        let pin = LibraryPin {
            number: "1".to_owned(),
            name: "NAME".to_owned(),
            at: Point::new(0, 0),
            angle: Angle(0),
            length: Iu(2_540),
            electrical: "passive".to_owned(),
            hidden: false,
            alternates: Vec::new(),
            node,
        };
        (doc, symbol, pin)
    }

    #[test]
    fn pin_text_follows_the_name_offset_and_the_number_setting() {
        // A positive name offset draws the name inside the body, past the end
        // of the pin, and the number over the pin.
        let (doc, symbol, pin) = one_pin(Iu(508), false, false);
        let boxes = pin_text_boxes(&doc, &symbol, &pin);
        assert_eq!(boxes.len(), 2);
        let name = boxes[0].1;
        assert!(
            name.start().x >= pin.at.x + pin.length,
            "the name is not inside the body: {name}"
        );
        let number = boxes[1].1;
        assert!(number.end().y <= Iu(0), "the number is not over the pin");

        // With no offset the name goes over the pin and the number below it.
        let (doc, symbol, pin) = one_pin(Iu(0), false, false);
        let boxes = pin_text_boxes(&doc, &symbol, &pin);
        assert!(boxes[0].1.end().y <= Iu(0), "the name is not over the pin");
        assert!(
            boxes[1].1.start().y >= Iu(0),
            "the number is not under the pin"
        );

        // A symbol that hides its pin numbers draws only the name, and one that
        // hides both draws nothing.
        let (doc, symbol, pin) = one_pin(Iu(0), false, true);
        assert_eq!(pin_text_boxes(&doc, &symbol, &pin).len(), 1);
        let (doc, symbol, pin) = one_pin(Iu(0), true, true);
        assert!(pin_text_boxes(&doc, &symbol, &pin).is_empty());
    }

    #[test]
    fn a_pin_runs_the_way_the_library_says() {
        // The library reader has flipped Y, so the file's 90 degrees is up the
        // page, which is the smaller Y.
        assert_eq!(pin_direction(Angle(0)), (1, 0));
        assert_eq!(pin_direction(Angle(90)), (0, -1));
        assert_eq!(pin_direction(Angle(180)), (-1, 0));
        assert_eq!(pin_direction(Angle(270)), (0, 1));
    }
}
