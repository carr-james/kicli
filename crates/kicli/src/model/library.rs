//! The library symbols a schematic embeds, and how a placement resolves to one.
//!
//! KiCad draws the embedded copy in `lib_symbols`, not the library file on
//! disk, so this cache is the definition that matters. Its key is the full
//! `lib_id` unless the symbol was edited in place, in which case `(lib_name)`
//! redirects to a uniquified entry. Resolving by `lib_id` alone finds the wrong
//! definition for exactly those symbols.
//!
//! Library coordinates are Y-up. They are negated as they are read here, so
//! every point this module hands out is already in schematic sense.

use crate::geometry::{Angle, Iu, Point};
use crate::model::items::{Field, Symbol, read_fields_of};
use crate::model::version::{FormatVersion, pin_text};
use kicli_sexpr::{Doc, NodeId};

/// One pin of a library symbol.
#[derive(Clone, Debug)]
pub struct LibraryPin {
    /// The pin number, as text.
    pub number: String,
    /// The pin name. An unnamed pin is the empty string.
    pub name: String,
    /// The connection point, relative to the symbol anchor, in schematic sense.
    pub at: Point,
    /// The angle the pin body runs in, in library sense: 0 right, 90 up,
    /// 180 left, 270 down.
    pub angle: Angle,
    /// How long the pin body is.
    pub length: Iu,
    /// The electrical type, such as `passive` or `power_in`.
    pub electrical: String,
    /// Is the pin hidden? A hidden power pin still connects.
    pub hidden: bool,
    /// The alternate functions the pin offers.
    pub alternates: Vec<String>,
    /// The `pin` list this was read from.
    pub node: NodeId,
}

/// A drawn shape of a library symbol, in schematic sense.
#[derive(Clone, Debug)]
pub enum Shape {
    /// A rectangle, by two opposite corners.
    Rectangle {
        /// One corner.
        start: Point,
        /// The opposite corner.
        end: Point,
    },
    /// A run of straight segments.
    Polyline {
        /// The points, in order.
        points: Vec<Point>,
    },
    /// A circle.
    Circle {
        /// The centre.
        center: Point,
        /// The radius.
        radius: Iu,
    },
    /// An arc, by three points.
    Arc {
        /// Where the arc starts.
        start: Point,
        /// A point along it.
        mid: Point,
        /// Where it ends.
        end: Point,
    },
}

/// One unit and body style of a library symbol.
#[derive(Clone, Debug)]
pub struct LibraryUnit {
    /// Which unit this draws. Unit 0 is common to every unit.
    pub unit: u32,
    /// Which body style this draws. Style 0 is common to both.
    pub body_style: u32,
    /// The pins of this unit.
    pub pins: Vec<LibraryPin>,
    /// The graphics of this unit.
    pub shapes: Vec<Shape>,
}

/// A symbol definition, as the file embeds it.
#[derive(Clone, Debug)]
pub struct LibrarySymbol {
    /// The cache key, which carries the library nickname.
    pub name: String,
    /// Is this a power symbol? Its value is then a net name.
    pub is_power: bool,
    /// How far pin names sit from the body.
    pub pin_name_offset: Iu,
    /// Are pin numbers drawn?
    pub pin_numbers_hidden: bool,
    /// Are pin names drawn?
    pub pin_names_hidden: bool,
    /// The default fields, from which a placement's fields are derived.
    pub fields: Vec<Field>,
    /// The units, in file order.
    pub units: Vec<LibraryUnit>,
    /// The `symbol` list this was read from.
    pub node: NodeId,
}

impl LibrarySymbol {
    /// The units a placement draws: its own, plus the common ones.
    ///
    /// A unit of 0 is common to every unit, and a body style of 0 to both
    /// styles. Missing them loses the pins of most multi-unit parts, whose
    /// graphics live in unit 0.
    pub fn units_for(&self, unit: u32, body_style: u32) -> impl Iterator<Item = &LibraryUnit> {
        self.units.iter().filter(move |candidate| {
            (candidate.unit == unit || candidate.unit == 0)
                && (candidate.body_style == body_style || candidate.body_style == 0)
        })
    }

    /// The pins a placement draws.
    pub fn pins_for(&self, unit: u32, body_style: u32) -> impl Iterator<Item = &LibraryPin> {
        self.units_for(unit, body_style)
            .flat_map(|selected| selected.pins.iter())
    }
}

/// Read every entry of a file's `lib_symbols`.
#[must_use]
pub fn read_library(
    doc: &Doc,
    entries: &[(String, NodeId)],
    version: FormatVersion,
) -> Vec<LibrarySymbol> {
    entries
        .iter()
        .map(|(name, node)| read_symbol(doc, name, *node, version))
        .collect()
}

/// Find the definition a placement draws.
///
/// `lib_name` wins when it is present, because that is what KiCad keys the
/// cache by for a symbol edited in place.
#[must_use]
pub fn definition_of<'a>(
    library: &'a [LibrarySymbol],
    symbol: &Symbol,
) -> Option<&'a LibrarySymbol> {
    let key = symbol.lib_name.as_deref().unwrap_or(&symbol.lib_id.0);
    library.iter().find(|candidate| candidate.name == key)
}

fn read_symbol(doc: &Doc, name: &str, node: NodeId, version: FormatVersion) -> LibrarySymbol {
    let mut symbol = LibrarySymbol {
        name: name.to_owned(),
        is_power: doc
            .children(node)
            .iter()
            .any(|&child| doc.head_is(child, "power")),
        pin_name_offset: Iu(0),
        pin_numbers_hidden: false,
        pin_names_hidden: false,
        fields: read_fields_of(doc, node, version),
        units: Vec::new(),
        node,
    };

    for &child in doc.children(node) {
        match doc.head(child) {
            Some("pin_names") => {
                symbol.pin_names_hidden = has_hide(doc, child);
                for &setting in doc.children(child) {
                    if doc.head_is(setting, "offset") {
                        if let Some(offset) = number_at(doc, setting, 1) {
                            symbol.pin_name_offset = offset;
                        }
                    }
                }
            }
            Some("pin_numbers") => symbol.pin_numbers_hidden = has_hide(doc, child),
            Some("symbol") => {
                let child_name = doc
                    .children(child)
                    .get(1)
                    .and_then(|&id| doc.atom_as_str(id))
                    .unwrap_or_default();
                let (unit, body_style) = unit_of(&child_name);
                symbol.units.push(LibraryUnit {
                    unit,
                    body_style,
                    pins: read_pins(doc, child, version),
                    shapes: read_shapes(doc, child),
                });
            }
            _ => {}
        }
    }
    symbol
}

/// Split a child name of the form `NAME_<unit>_<bodyStyle>`.
///
/// The symbol name may itself hold underscores, so the two numbers are taken
/// from the end rather than by splitting on the first separator.
fn unit_of(name: &str) -> (u32, u32) {
    let Some((rest, style)) = name.rsplit_once('_') else {
        return (0, 0);
    };
    let Some((_, unit)) = rest.rsplit_once('_') else {
        return (0, 0);
    };
    (unit.parse().unwrap_or(0), style.parse().unwrap_or(0))
}

fn read_pins(doc: &Doc, unit: NodeId, version: FormatVersion) -> Vec<LibraryPin> {
    let mut pins = Vec::new();
    for &child in doc.children(unit) {
        if !doc.head_is(child, "pin") {
            continue;
        }
        let values = doc.children(child);
        let electrical = values
            .get(1)
            .and_then(|&id| doc.atom_text(id))
            .unwrap_or_default()
            .to_owned();
        let at = doc
            .children(child)
            .iter()
            .copied()
            .find(|&id| doc.head_is(id, "at"));
        // Library space is Y-up. Negate here, once, so that everything
        // downstream works in schematic sense.
        let position = at
            .map(|id| Point {
                x: number_at(doc, id, 1).unwrap_or(Iu(0)),
                y: Iu(-number_at(doc, id, 2).unwrap_or(Iu(0)).0),
            })
            .unwrap_or_default();
        let angle = at
            .and_then(|id| doc.children(id).get(3).copied())
            .and_then(|id| doc.atom_text(id))
            .and_then(Angle::from_text)
            .unwrap_or_default();

        pins.push(LibraryPin {
            number: pin_text(&text_of(doc, child, "number"), version).to_owned(),
            name: pin_text(&text_of(doc, child, "name"), version).to_owned(),
            at: position,
            angle,
            length: child_number(doc, child, "length").unwrap_or(Iu(0)),
            electrical,
            hidden: has_hide(doc, child),
            alternates: doc
                .children(child)
                .iter()
                .filter(|&&id| doc.head_is(id, "alternate"))
                .filter_map(|&id| doc.children(id).get(1).and_then(|&v| doc.atom_as_str(v)))
                .collect(),
            node: child,
        });
    }
    pins
}

fn read_shapes(doc: &Doc, unit: NodeId) -> Vec<Shape> {
    let mut shapes = Vec::new();
    for &child in doc.children(unit) {
        match doc.head(child) {
            Some("rectangle") => {
                if let (Some(start), Some(end)) = (
                    named_point(doc, child, "start"),
                    named_point(doc, child, "end"),
                ) {
                    shapes.push(Shape::Rectangle { start, end });
                }
            }
            Some("polyline" | "bezier") => {
                let points = point_run(doc, child);
                if !points.is_empty() {
                    shapes.push(Shape::Polyline { points });
                }
            }
            Some("circle") => {
                if let (Some(center), Some(radius)) = (
                    named_point(doc, child, "center"),
                    child_number(doc, child, "radius"),
                ) {
                    shapes.push(Shape::Circle { center, radius });
                }
            }
            Some("arc") => {
                if let (Some(start), Some(mid), Some(end)) = (
                    named_point(doc, child, "start"),
                    named_point(doc, child, "mid"),
                    named_point(doc, child, "end"),
                ) {
                    shapes.push(Shape::Arc { start, mid, end });
                }
            }
            _ => {}
        }
    }
    shapes
}

/// The `(pts (xy ...) ...)` of a shape, in schematic sense.
fn point_run(doc: &Doc, node: NodeId) -> Vec<Point> {
    let mut points = Vec::new();
    for &child in doc.children(node) {
        if !doc.head_is(child, "pts") {
            continue;
        }
        for &pair in doc.children(child) {
            if !doc.head_is(pair, "xy") {
                continue;
            }
            points.push(Point {
                x: number_at(doc, pair, 1).unwrap_or(Iu(0)),
                y: Iu(-number_at(doc, pair, 2).unwrap_or(Iu(0)).0),
            });
        }
    }
    points
}

/// A named child list holding a point, in schematic sense.
fn named_point(doc: &Doc, node: NodeId, head: &str) -> Option<Point> {
    let child = doc
        .children(node)
        .iter()
        .copied()
        .find(|&id| doc.head_is(id, head))?;
    Some(Point {
        x: number_at(doc, child, 1)?,
        y: Iu(-number_at(doc, child, 2)?.0),
    })
}

/// The text of a `(name "...")` or `(number "...")` child.
fn text_of(doc: &Doc, node: NodeId, head: &str) -> String {
    doc.children(node)
        .iter()
        .copied()
        .find(|&id| doc.head_is(id, head))
        .and_then(|id| doc.children(id).get(1).and_then(|&v| doc.atom_as_str(v)))
        .unwrap_or_default()
}

/// Does a list carry `hide` or `(hide yes)`?
fn has_hide(doc: &Doc, node: NodeId) -> bool {
    doc.children(node).iter().any(|&child| {
        doc.atom_text(child) == Some("hide")
            || (doc.head_is(child, "hide")
                && doc.children(child).get(1).and_then(|&id| doc.atom_text(id)) != Some("no"))
    })
}

/// A number in a list's nth position, as internal units.
fn number_at(doc: &Doc, node: NodeId, index: usize) -> Option<Iu> {
    doc.children(node)
        .get(index)
        .and_then(|&id| doc.atom_as_iu(id))
        .map(Iu)
}

/// The first value of a named child list, as internal units.
fn child_number(doc: &Doc, node: NodeId, head: &str) -> Option<Iu> {
    let child = doc
        .children(node)
        .iter()
        .copied()
        .find(|&id| doc.head_is(id, head))?;
    number_at(doc, child, 1)
}

#[cfg(test)]
mod tests {
    use super::unit_of;

    #[test]
    fn a_child_name_gives_up_its_unit_and_body_style() {
        assert_eq!(unit_of("R_0_1"), (0, 1));
        assert_eq!(unit_of("R_1_1"), (1, 1));
        assert_eq!(unit_of("LM358_2_1"), (2, 1));
        // A symbol name with underscores of its own still parses, because the
        // numbers are taken from the end.
        assert_eq!(unit_of("74LS00_A_B_3_2"), (3, 2));
        assert_eq!(unit_of("nonsense"), (0, 0));
    }
}
